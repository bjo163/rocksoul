//! Deterministic, in-process fixture executor for agent-runtime development.
//!
//! This is NOT an OS sandbox. It has no shell, network, filesystem, process,
//! environment-variable, or arbitrary-code capability. It mutates only its
//! private in-memory map and exists to test authorization and evidence flow.
use std::{
    collections::BTreeMap,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};

use crate::{
    request_authorization::{
        ArgumentRule, AuthorizationReason, RequestGrounding, ToolWriteSchema, WriteCommitmentGate,
        WriteProposal,
    },
    tool_audit::{
        InvocationControl, InvocationStopReason, ToolAuditEvent, ToolAuditKind, ToolAuditStore,
        ToolAuthorization, ToolEffectClass, ToolFailureCategory,
    },
    tool_registry::{ToolCapability, ToolDescriptor, ToolEffect, ToolPolicy, ToolRegistry},
};

pub const FIXTURE_WRITE_TOOL: &str = "fixture.state.write";
pub const FIXTURE_READ_TOOL: &str = "fixture.state.read";
const MAX_KEY_BYTES: usize = 64;
const MAX_VALUE_BYTES: usize = 4_096;
const MAX_STATE_ENTRIES: usize = 32;
const MAX_EXECUTION_STEPS: u16 = 16;
const MAX_TOTAL_INPUT_BYTES: usize = 16_384;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FixtureToolCall {
    pub tool_id: String,
    /// Fixture-only input. Production traces should store a digest, not raw input.
    pub input: String,
}

#[derive(Clone, Debug)]
pub struct FixtureAuditContext {
    pub audit_path: std::path::PathBuf,
    pub run_id: String,
    pub first_sequence: u64,
    pub control: InvocationControl,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FixtureToolObservation {
    pub tool_id: String,
    pub result: String,
    pub evidence_ref: String,
    pub result_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FixtureVerification {
    pub verifier_id: String,
    pub rubric_version: String,
    pub independent: bool,
    pub passed: bool,
    pub confidence_basis_points: u16,
    pub evidence_refs: Vec<String>,
}

/// A deliberately tiny executor with no ambient host capabilities. It is not
/// `Clone`: cloning would duplicate a single-use write commitment.
#[derive(Debug)]
pub struct FixtureExecutor {
    state: BTreeMap<String, String>,
    observed_effects: BTreeMap<String, String>,
    remaining_steps: u16,
    remaining_input_bytes: usize,
    write_gate: WriteCommitmentGate,
}

impl Default for FixtureExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl FixtureExecutor {
    pub fn new() -> Self {
        Self {
            state: BTreeMap::new(),
            observed_effects: BTreeMap::new(),
            remaining_steps: MAX_EXECUTION_STEPS,
            remaining_input_bytes: MAX_TOTAL_INPUT_BYTES,
            write_gate: WriteCommitmentGate::new(),
        }
    }

    pub fn state_len(&self) -> usize {
        self.state.len()
    }

    pub fn remaining_steps(&self) -> u16 {
        self.remaining_steps
    }

    pub fn remaining_input_bytes(&self) -> usize {
        self.remaining_input_bytes
    }

    pub fn execute(
        &mut self,
        registry: &ToolRegistry,
        policy: &ToolPolicy,
        call: &FixtureToolCall,
    ) -> Result<FixtureToolObservation> {
        let descriptor = registry.authorize(&call.tool_id, call.input.len(), policy)?;
        ensure!(
            self.remaining_steps > 0,
            "fixture execution step budget exhausted"
        );
        ensure!(
            call.input.len() <= self.remaining_input_bytes,
            "fixture aggregate input-byte budget exhausted"
        );
        self.remaining_steps -= 1;
        self.remaining_input_bytes -= call.input.len();
        match call.tool_id.as_str() {
            FIXTURE_WRITE_TOOL => {
                ensure!(
                    descriptor.capability == ToolCapability::WorkspaceWrite
                        && descriptor.effect == ToolEffect::Mutating,
                    "fixture write tool descriptor has an invalid capability/effect"
                );
                let (key, value) = call
                    .input
                    .split_once('=')
                    .ok_or_else(|| anyhow::anyhow!("fixture write input must be key=value"))?;
                validate_key(key)?;
                ensure!(
                    !value.is_empty() && value.len() <= MAX_VALUE_BYTES,
                    "fixture value must be non-empty and at most 4096 bytes"
                );
                ensure!(
                    self.state.contains_key(key) || self.state.len() < MAX_STATE_ENTRIES,
                    "fixture state entry limit reached"
                );
                self.state.insert(key.to_owned(), value.to_owned());
                let evidence_ref = format!("fixture://state/{key}");
                self.observed_effects
                    .insert(evidence_ref.clone(), key.to_owned());
                self.write_gate.note_observation()?;
                Ok(observation(
                    FIXTURE_WRITE_TOOL,
                    "state updated",
                    evidence_ref,
                    value,
                ))
            }
            FIXTURE_READ_TOOL => {
                ensure!(
                    descriptor.capability == ToolCapability::WorkspaceRead
                        && descriptor.effect == ToolEffect::ReadOnly,
                    "fixture read tool descriptor has an invalid capability/effect"
                );
                validate_key(&call.input)?;
                let value = self
                    .state
                    .get(&call.input)
                    .ok_or_else(|| anyhow::anyhow!("fixture state key does not exist"))?;
                let evidence_ref = format!("fixture://state/{}", call.input);
                self.observed_effects
                    .insert(evidence_ref.clone(), call.input.clone());
                self.write_gate.note_observation()?;
                Ok(observation(FIXTURE_READ_TOOL, value, evidence_ref, value))
            }
            _ => anyhow::bail!("fixture executor does not implement tool: {}", call.tool_id),
        }
    }

    pub fn observations_seen(&self) -> u64 {
        self.write_gate.observations_seen()
    }

    pub fn is_write_committed(&self) -> bool {
        self.write_gate.is_committed()
    }

    /// Freeze the complete write set before any untrusted observation enters.
    pub fn commit_writes(
        &mut self,
        grounding: &RequestGrounding,
        schemas: &BTreeMap<String, ToolWriteSchema>,
        proposals: Vec<WriteProposal>,
    ) -> std::result::Result<(), AuthorizationReason> {
        self.write_gate.commit(grounding, schemas, proposals)
    }

    pub fn commit_writes_with_confirmations(
        &mut self,
        grounding: &RequestGrounding,
        schemas: &BTreeMap<String, ToolWriteSchema>,
        proposals: Vec<WriteProposal>,
        confirmations: &mut [crate::request_authorization::ConfirmationGrant],
    ) -> Result<()> {
        self.write_gate
            .commit_with_confirmations(grounding, schemas, proposals, confirmations)
    }

    /// Resolve a fixture call to its committed write proposal and consume exactly
    /// one slot of the frozen set. This is the only path audited dispatch uses for
    /// mutating tools.
    fn authorize_committed_write(
        &mut self,
        registry: &ToolRegistry,
        call: &FixtureToolCall,
    ) -> std::result::Result<(), AuthorizationReason> {
        let descriptor = registry
            .get(&call.tool_id)
            .ok_or(AuthorizationReason::UnknownTool)?;
        if descriptor.effect != ToolEffect::Mutating {
            return Ok(());
        }
        let schema = descriptor
            .write_schema
            .as_ref()
            .ok_or(AuthorizationReason::UnclassifiedArgument)?;
        let (key, value) = call
            .input
            .split_once('=')
            .ok_or(AuthorizationReason::InvalidProposal)?;
        let mut arguments = BTreeMap::new();
        for name in schema.arguments.keys() {
            match name.as_str() {
                "key" => {
                    arguments.insert(name.clone(), key.to_owned());
                }
                "value" => {
                    arguments.insert(name.clone(), value.to_owned());
                }
                _ => return Err(AuthorizationReason::UnclassifiedArgument),
            }
        }
        if !arguments.contains_key("key") || !arguments.contains_key("value") {
            return Err(AuthorizationReason::InvalidProposal);
        }
        let proposal = WriteProposal {
            tool_id: schema.tool_id.clone(),
            arguments,
        };
        let decision = self.write_gate.authorize_next(&proposal);
        if decision.allowed {
            Ok(())
        } else {
            Err(decision.reason)
        }
    }

    /// Execute a fixture tool while persisting content-free audit events.
    ///
    /// This is test infrastructure only. It is not a production dispatcher, sandbox,
    /// independent verifier, or human-approval workflow.
    pub fn execute_audited(
        &mut self,
        registry: &ToolRegistry,
        policy: &ToolPolicy,
        call: &FixtureToolCall,
        audit: &FixtureAuditContext,
    ) -> Result<FixtureToolObservation> {
        let descriptor = registry
            .get(&call.tool_id)
            .ok_or_else(|| anyhow::anyhow!("unknown fixture tool: {}", call.tool_id))?;
        let mut next_sequence = audit.first_sequence;
        let mut append_event = |kind: ToolAuditKind,
                                authorization: Option<ToolAuthorization>,
                                failure: Option<ToolFailureCategory>,
                                evidence_refs: Vec<String>|
         -> Result<()> {
            let event = fixture_audit_event(
                &audit.run_id,
                next_sequence,
                kind,
                descriptor,
                authorization,
                failure,
                evidence_refs,
            )?;
            ToolAuditStore::append(&audit.audit_path, &event)?;
            next_sequence = next_sequence
                .checked_add(1)
                .context("fixture audit sequence exhausted")?;
            Ok(())
        };

        append_event(ToolAuditKind::Proposed, None, None, Vec::new())?;
        if let Err(error) = registry.authorize(&call.tool_id, call.input.len(), policy) {
            append_event(
                ToolAuditKind::AuthorizationDenied,
                Some(ToolAuthorization::Denied),
                Some(ToolFailureCategory::PolicyDenied),
                Vec::new(),
            )?;
            return Err(error);
        }

        if let Err(reason) = self.authorize_committed_write(registry, call) {
            append_event(
                ToolAuditKind::AuthorizationDenied,
                Some(ToolAuthorization::Denied),
                reason.audit_failure(),
                Vec::new(),
            )?;
            anyhow::bail!("write authorization denied: {}", reason.as_str());
        }

        append_event(
            ToolAuditKind::AuthorizationAllowed,
            Some(ToolAuthorization::Allowed),
            None,
            Vec::new(),
        )?;
        if let Err(reason) = audit.control.check_and_consume() {
            let (kind, failure) = match reason {
                InvocationStopReason::TimedOut => (
                    ToolAuditKind::TimedOut,
                    ToolFailureCategory::DeadlineExceeded,
                ),
                InvocationStopReason::Cancelled => {
                    (ToolAuditKind::Cancelled, ToolFailureCategory::Cancelled)
                }
                InvocationStopReason::CallBudgetExhausted => (
                    ToolAuditKind::Failed,
                    ToolFailureCategory::ResourceExhausted,
                ),
            };
            append_event(kind, None, Some(failure), Vec::new())?;
            anyhow::bail!("fixture invocation stopped before dispatch: {reason:?}");
        }
        append_event(ToolAuditKind::ExecutionStarted, None, None, Vec::new())?;

        match self.execute(registry, policy, call) {
            Ok(observation) => {
                append_event(
                    ToolAuditKind::ExecutionFinished,
                    None,
                    None,
                    vec![observation.evidence_ref.clone()],
                )?;
                Ok(observation)
            }
            Err(error) => {
                append_event(
                    ToolAuditKind::Failed,
                    None,
                    Some(ToolFailureCategory::WorkerFailure),
                    Vec::new(),
                )?;
                Err(error)
            }
        }
    }

    /// The verifier reads executor-observed state, never a model-provided result string.
    pub fn verify_state_equals(
        &self,
        evidence_ref: &str,
        expected_value: &str,
    ) -> Result<FixtureVerification> {
        let key = self
            .observed_effects
            .get(evidence_ref)
            .ok_or_else(|| anyhow::anyhow!("unknown fixture evidence reference"))?;
        let observed = self
            .state
            .get(key)
            .ok_or_else(|| anyhow::anyhow!("observed fixture state is missing"))?;
        Ok(FixtureVerification {
            verifier_id: "rocksoul.fixture-state-verifier".to_owned(),
            rubric_version: "1.0.0".to_owned(),
            independent: false,
            passed: observed == expected_value,
            confidence_basis_points: 10_000,
            evidence_refs: vec![evidence_ref.to_owned()],
        })
    }
}

fn fixture_audit_event(
    run_id: &str,
    sequence: u64,
    kind: ToolAuditKind,
    descriptor: &ToolDescriptor,
    authorization: Option<ToolAuthorization>,
    failure: Option<ToolFailureCategory>,
    evidence_refs: Vec<String>,
) -> Result<ToolAuditEvent> {
    let timestamp_unix_ms = u64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("system clock is before Unix epoch")?
            .as_millis(),
    )
    .context("system timestamp exceeds supported range")?;
    let run_digest = Sha256::digest(run_id.as_bytes());
    let run_digest = format!("{run_digest:x}");
    let capability = match descriptor.capability {
        ToolCapability::WorkspaceRead => "workspace_read",
        ToolCapability::WorkspaceWrite => "workspace_write",
        ToolCapability::MemoryRead => "memory_read",
        ToolCapability::MemoryWrite => "memory_write",
    };
    let effect = match descriptor.effect {
        ToolEffect::ReadOnly => ToolEffectClass::ReadOnly,
        ToolEffect::Mutating => ToolEffectClass::Mutating,
    };
    let event = ToolAuditEvent {
        schema_version: 1,
        run_id: run_id.to_owned(),
        sequence,
        event_id: format!("audit-{}-{sequence}", &run_digest[..64]),
        kind,
        tool_id: descriptor.id.clone(),
        tool_version: descriptor.version.clone(),
        capability: capability.to_owned(),
        effect,
        policy_version: "1.0.0".to_owned(),
        timestamp_unix_ms,
        authorization,
        approval: None,
        failure,
        evidence_refs,
    };
    event.validate()?;
    Ok(event)
}

pub fn register_fixture_tools(registry: &mut ToolRegistry) -> Result<()> {
    registry.register(ToolDescriptor {
        id: FIXTURE_WRITE_TOOL.to_owned(),
        version: "1.0.0".to_owned(),
        capability: ToolCapability::WorkspaceWrite,
        effect: ToolEffect::Mutating,
        max_input_bytes: MAX_KEY_BYTES + 1 + MAX_VALUE_BYTES,
        write_schema: Some(ToolWriteSchema {
            tool_id: FIXTURE_WRITE_TOOL.to_owned(),
            arguments: BTreeMap::from([
                ("key".to_owned(), ArgumentRule::authority(true)),
                ("value".to_owned(), ArgumentRule::payload(true)),
            ]),
        }),
    })?;
    registry.register(ToolDescriptor {
        id: FIXTURE_READ_TOOL.to_owned(),
        version: "1.0.0".to_owned(),
        capability: ToolCapability::WorkspaceRead,
        effect: ToolEffect::ReadOnly,
        max_input_bytes: MAX_KEY_BYTES,
        write_schema: None,
    })?;
    Ok(())
}

fn validate_key(key: &str) -> Result<()> {
    ensure!(
        !key.is_empty()
            && key.len() <= MAX_KEY_BYTES
            && key.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
            }),
        "fixture key must be 1-64 lowercase ASCII letters, digits, '_' or '-'"
    );
    Ok(())
}

fn observation(
    tool_id: &str,
    result: &str,
    evidence_ref: String,
    digest_source: &str,
) -> FixtureToolObservation {
    let digest = Sha256::digest(digest_source.as_bytes());
    FixtureToolObservation {
        tool_id: tool_id.to_owned(),
        result: result.to_owned(),
        evidence_ref,
        result_sha256: format!("{digest:x}"),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{
        FIXTURE_READ_TOOL, FIXTURE_WRITE_TOOL, FixtureAuditContext, FixtureExecutor,
        FixtureToolCall, register_fixture_tools,
    };
    use crate::{
        request_authorization::{RequestGrounding, WriteProposal},
        tool_audit::InvocationControl,
        tool_registry::{ToolCapability, ToolPolicy, ToolRegistry},
    };

    /// Commit the exact `key=value` write before any audited dispatch.
    fn commit_fixture_write(
        executor: &mut FixtureExecutor,
        registry: &ToolRegistry,
        call: &FixtureToolCall,
    ) {
        let (key, value) = call.input.split_once('=').unwrap();
        let grounding =
            RequestGrounding::from_user_request("fixture-commit", format!("Set {key} to {value}"))
                .unwrap();
        let proposal = WriteProposal {
            tool_id: FIXTURE_WRITE_TOOL.to_owned(),
            arguments: BTreeMap::from([
                ("key".to_owned(), key.to_owned()),
                ("value".to_owned(), value.to_owned()),
            ]),
        };
        executor
            .commit_writes(&grounding, &registry.write_schemas(), vec![proposal])
            .unwrap();
    }

    fn configured() -> (ToolRegistry, ToolPolicy) {
        let mut registry = ToolRegistry::default();
        register_fixture_tools(&mut registry).unwrap();
        let mut policy = ToolPolicy {
            max_input_bytes: 4_161,
            ..ToolPolicy::default()
        };
        policy.allow_tool(FIXTURE_WRITE_TOOL);
        policy.allow_tool(FIXTURE_READ_TOOL);
        policy.grant(ToolCapability::WorkspaceWrite);
        policy.grant(ToolCapability::WorkspaceRead);
        (registry, policy)
    }

    #[test]
    fn fixture_executor_is_deny_by_default() {
        let mut registry = ToolRegistry::default();
        register_fixture_tools(&mut registry).unwrap();
        let mut executor = FixtureExecutor::new();
        assert!(
            executor
                .execute(
                    &registry,
                    &ToolPolicy::default(),
                    &FixtureToolCall {
                        tool_id: FIXTURE_WRITE_TOOL.to_owned(),
                        input: "goal=done".to_owned(),
                    },
                )
                .is_err()
        );
        assert_eq!(executor.state_len(), 0);
    }

    #[test]
    fn audited_fixture_execution_records_content_free_ordered_events() {
        use std::time::{SystemTime, UNIX_EPOCH};

        use crate::tool_audit::{ToolAuditKind, ToolAuditStore};

        let (registry, policy) = configured();
        let mut executor = FixtureExecutor::new();
        let call = FixtureToolCall {
            tool_id: FIXTURE_WRITE_TOOL.to_owned(),
            input: "goal=done".to_owned(),
        };
        commit_fixture_write(&mut executor, &registry, &call);
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("rocksoul-fixture-audit-{nonce}.jsonl"));
        let observation = executor
            .execute_audited(
                &registry,
                &policy,
                &call,
                &FixtureAuditContext {
                    audit_path: path.clone(),
                    run_id: "fixture-run-1".to_owned(),
                    first_sequence: 1,
                    control: InvocationControl::new(std::time::Duration::from_secs(5), 1),
                },
            )
            .unwrap();
        let events = ToolAuditStore::read(&path).unwrap();
        assert_eq!(
            events.iter().map(|event| event.kind).collect::<Vec<_>>(),
            vec![
                ToolAuditKind::Proposed,
                ToolAuditKind::AuthorizationAllowed,
                ToolAuditKind::ExecutionStarted,
                ToolAuditKind::ExecutionFinished,
            ]
        );
        assert_eq!(
            events.last().unwrap().evidence_refs,
            vec![observation.evidence_ref]
        );
        let persisted = std::fs::read_to_string(&path).unwrap();
        assert!(!persisted.contains("goal=done"));
        assert!(!persisted.contains("input"));
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn audited_fixture_denial_is_recorded_before_any_mutation() {
        use std::time::{SystemTime, UNIX_EPOCH};

        use crate::tool_audit::{ToolAuditKind, ToolAuditStore};

        let (registry, mut policy) = configured();
        policy
            .granted_capabilities
            .remove(&ToolCapability::WorkspaceWrite);
        let mut executor = FixtureExecutor::new();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("rocksoul-fixture-denial-{nonce}.jsonl"));
        assert!(
            executor
                .execute_audited(
                    &registry,
                    &policy,
                    &FixtureToolCall {
                        tool_id: FIXTURE_WRITE_TOOL.to_owned(),
                        input: "goal=done".to_owned(),
                    },
                    &FixtureAuditContext {
                        audit_path: path.clone(),
                        run_id: "fixture-denied-1".to_owned(),
                        first_sequence: 1,
                        control: InvocationControl::new(std::time::Duration::from_secs(5), 1),
                    },
                )
                .is_err()
        );
        assert_eq!(executor.state_len(), 0);
        let events = ToolAuditStore::read(&path).unwrap();
        assert_eq!(
            events.iter().map(|event| event.kind).collect::<Vec<_>>(),
            vec![ToolAuditKind::Proposed, ToolAuditKind::AuthorizationDenied,]
        );
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn audited_fixture_deadline_and_cancellation_stop_before_mutation() {
        use std::time::{Duration, SystemTime, UNIX_EPOCH};

        use crate::tool_audit::{ToolAuditKind, ToolAuditStore};

        let (registry, policy) = configured();
        let call = FixtureToolCall {
            tool_id: FIXTURE_WRITE_TOOL.to_owned(),
            input: "goal=done".to_owned(),
        };
        let mut executor = FixtureExecutor::new();
        commit_fixture_write(&mut executor, &registry, &call);
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let timeout_path =
            std::env::temp_dir().join(format!("rocksoul-fixture-timeout-{nonce}.jsonl"));
        assert!(
            executor
                .execute_audited(
                    &registry,
                    &policy,
                    &call,
                    &FixtureAuditContext {
                        audit_path: timeout_path.clone(),
                        run_id: "fixture-timeout".to_owned(),
                        first_sequence: 1,
                        control: InvocationControl::new(Duration::ZERO, 1),
                    },
                )
                .is_err()
        );
        assert_eq!(executor.state_len(), 0);
        let timeout_events = ToolAuditStore::read(&timeout_path).unwrap();
        assert_eq!(
            timeout_events
                .iter()
                .map(|event| event.kind)
                .collect::<Vec<_>>(),
            vec![
                ToolAuditKind::Proposed,
                ToolAuditKind::AuthorizationAllowed,
                ToolAuditKind::TimedOut,
            ]
        );
        std::fs::remove_file(timeout_path).unwrap();

        let cancel_path =
            std::env::temp_dir().join(format!("rocksoul-fixture-cancel-{nonce}.jsonl"));
        let control = InvocationControl::new(Duration::from_secs(5), 1);
        control.cancel();
        let mut cancel_executor = FixtureExecutor::new();
        commit_fixture_write(&mut cancel_executor, &registry, &call);
        assert!(
            cancel_executor
                .execute_audited(
                    &registry,
                    &policy,
                    &call,
                    &FixtureAuditContext {
                        audit_path: cancel_path.clone(),
                        run_id: "fixture-cancel".to_owned(),
                        first_sequence: 1,
                        control: control.clone(),
                    },
                )
                .is_err()
        );
        assert_eq!(cancel_executor.state_len(), 0);
        let cancel_events = ToolAuditStore::read(&cancel_path).unwrap();
        assert_eq!(
            cancel_events
                .iter()
                .map(|event| event.kind)
                .collect::<Vec<_>>(),
            vec![
                ToolAuditKind::Proposed,
                ToolAuditKind::AuthorizationAllowed,
                ToolAuditKind::Cancelled,
            ]
        );
        std::fs::remove_file(cancel_path).unwrap();
    }

    #[test]
    fn audited_fixture_call_budget_is_enforced_before_dispatch() {
        use std::time::{Duration, SystemTime, UNIX_EPOCH};

        use crate::tool_audit::{ToolAuditKind, ToolAuditStore};

        let (registry, policy) = configured();
        let mut executor = FixtureExecutor::new();
        let call = FixtureToolCall {
            tool_id: FIXTURE_WRITE_TOOL.to_owned(),
            input: "goal=done".to_owned(),
        };
        commit_fixture_write(&mut executor, &registry, &call);
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("rocksoul-fixture-budget-{nonce}.jsonl"));
        assert!(
            executor
                .execute_audited(
                    &registry,
                    &policy,
                    &call,
                    &FixtureAuditContext {
                        audit_path: path.clone(),
                        run_id: "fixture-budget".to_owned(),
                        first_sequence: 1,
                        control: InvocationControl::new(Duration::from_secs(5), 0),
                    },
                )
                .is_err()
        );
        assert_eq!(executor.state_len(), 0);
        let events = ToolAuditStore::read(&path).unwrap();
        assert_eq!(events.last().unwrap().kind, ToolAuditKind::Failed);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn verifier_reads_observed_state_not_claimed_tool_output() {
        let (registry, policy) = configured();
        let mut executor = FixtureExecutor::new();
        let write = executor
            .execute(
                &registry,
                &policy,
                &FixtureToolCall {
                    tool_id: FIXTURE_WRITE_TOOL.to_owned(),
                    input: "goal=done".to_owned(),
                },
            )
            .unwrap();
        let verified = executor
            .verify_state_equals(&write.evidence_ref, "done")
            .unwrap();
        assert!(!verified.independent);
        assert!(verified.passed);
        let rejected = executor
            .verify_state_equals(&write.evidence_ref, "model-says-done")
            .unwrap();
        assert!(!rejected.passed);
    }

    #[test]
    fn read_tool_observes_persisted_fixture_state() {
        let (registry, policy) = configured();
        let mut executor = FixtureExecutor::new();
        executor
            .execute(
                &registry,
                &policy,
                &FixtureToolCall {
                    tool_id: FIXTURE_WRITE_TOOL.to_owned(),
                    input: "answer=42".to_owned(),
                },
            )
            .unwrap();
        let read = executor
            .execute(
                &registry,
                &policy,
                &FixtureToolCall {
                    tool_id: FIXTURE_READ_TOOL.to_owned(),
                    input: "answer".to_owned(),
                },
            )
            .unwrap();
        assert_eq!(read.result, "42");
        assert_eq!(read.tool_id, FIXTURE_READ_TOOL);
    }

    #[test]
    fn missing_capability_is_rejected_before_mutation() {
        let (registry, mut policy) = configured();
        policy
            .granted_capabilities
            .remove(&ToolCapability::WorkspaceWrite);
        let mut executor = FixtureExecutor::new();
        assert!(
            executor
                .execute(
                    &registry,
                    &policy,
                    &FixtureToolCall {
                        tool_id: FIXTURE_WRITE_TOOL.to_owned(),
                        input: "goal=done".to_owned(),
                    },
                )
                .is_err()
        );
        assert_eq!(executor.state_len(), 0);
    }

    #[test]
    fn unknown_evidence_and_unknown_tools_fail_closed() {
        let (registry, policy) = configured();
        let mut executor = FixtureExecutor::new();
        assert!(
            executor
                .verify_state_equals("fixture://state/forged", "done")
                .is_err()
        );
        assert!(
            executor
                .execute(
                    &registry,
                    &policy,
                    &FixtureToolCall {
                        tool_id: "process.shell".to_owned(),
                        input: "echo unsafe".to_owned(),
                    },
                )
                .is_err()
        );
    }

    #[test]
    fn fixture_runtime_enforces_step_and_aggregate_input_budgets() {
        let (registry, policy) = configured();
        let mut executor = FixtureExecutor::new();
        for index in 0..16 {
            executor
                .execute(
                    &registry,
                    &policy,
                    &FixtureToolCall {
                        tool_id: FIXTURE_WRITE_TOOL.to_owned(),
                        input: format!("key{index}=v"),
                    },
                )
                .unwrap();
        }
        assert_eq!(executor.remaining_steps(), 0);
        assert!(
            executor
                .execute(
                    &registry,
                    &policy,
                    &FixtureToolCall {
                        tool_id: FIXTURE_WRITE_TOOL.to_owned(),
                        input: "overflow=v".to_owned(),
                    },
                )
                .is_err()
        );

        let mut byte_limited = FixtureExecutor::new();
        let large_value = "x".repeat(4_096);
        for key in ["a", "b", "c"] {
            byte_limited
                .execute(
                    &registry,
                    &policy,
                    &FixtureToolCall {
                        tool_id: FIXTURE_WRITE_TOOL.to_owned(),
                        input: format!("{key}={large_value}"),
                    },
                )
                .unwrap();
        }
        assert!(byte_limited.remaining_input_bytes() < 4_096);
        assert!(
            byte_limited
                .execute(
                    &registry,
                    &policy,
                    &FixtureToolCall {
                        tool_id: FIXTURE_WRITE_TOOL.to_owned(),
                        input: format!("d={large_value}"),
                    },
                )
                .is_err()
        );
    }

    #[test]
    fn fixture_key_validation_rejects_path_like_input() {
        let (registry, policy) = configured();
        let mut executor = FixtureExecutor::new();
        assert!(
            executor
                .execute(
                    &registry,
                    &policy,
                    &FixtureToolCall {
                        tool_id: FIXTURE_WRITE_TOOL.to_owned(),
                        input: "../escape=value".to_owned(),
                    },
                )
                .is_err()
        );
        assert_eq!(executor.state_len(), 0);
    }

    #[test]
    fn audited_write_without_commitment_is_denied_with_content_free_reason() {
        use std::time::{SystemTime, UNIX_EPOCH};

        use crate::tool_audit::{ToolAuditKind, ToolAuditStore};

        let (registry, policy) = configured();
        let mut executor = FixtureExecutor::new();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("rocksoul-fixture-gate-{nonce}.jsonl"));
        let error = executor
            .execute_audited(
                &registry,
                &policy,
                &FixtureToolCall {
                    tool_id: FIXTURE_WRITE_TOOL.to_owned(),
                    input: "goal=done".to_owned(),
                },
                &FixtureAuditContext {
                    audit_path: path.clone(),
                    run_id: "fixture-no-commit".to_owned(),
                    first_sequence: 1,
                    control: InvocationControl::new(std::time::Duration::from_secs(5), 1),
                },
            )
            .unwrap_err();
        assert!(format!("{error:#}").contains("write_set_not_committed"));
        let events = ToolAuditStore::read(&path).unwrap();
        assert_eq!(events[1].kind, ToolAuditKind::AuthorizationDenied);
        assert_eq!(
            events[1].failure,
            Some(crate::tool_audit::ToolFailureCategory::PolicyDenied)
        );
        assert_eq!(executor.state_len(), 0);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn untrusted_observation_closes_the_write_commitment_window() {
        let (registry, policy) = configured();
        let mut executor = FixtureExecutor::new();
        // The raw fixture primitive produces an untrusted observation.
        executor
            .execute(
                &registry,
                &policy,
                &FixtureToolCall {
                    tool_id: FIXTURE_WRITE_TOOL.to_owned(),
                    input: "goal=done".to_owned(),
                },
            )
            .unwrap();
        assert_eq!(executor.observations_seen(), 1);
        let grounding =
            RequestGrounding::from_user_request("fixture-later", "Set other to value").unwrap();
        let proposal = WriteProposal {
            tool_id: FIXTURE_WRITE_TOOL.to_owned(),
            arguments: BTreeMap::from([
                ("key".to_owned(), "other".to_owned()),
                ("value".to_owned(), "value".to_owned()),
            ]),
        };
        assert!(
            executor
                .commit_writes(&grounding, &registry.write_schemas(), vec![proposal])
                .is_err()
        );
        assert!(!executor.is_write_committed());
    }
}
