//! Deterministic, in-process fixture executor for agent-runtime development.
//!
//! This is NOT an OS sandbox. It has no shell, network, filesystem, process,
//! environment-variable, or arbitrary-code capability. It mutates only its
//! private in-memory map and exists to test authorization and evidence flow.
use std::collections::BTreeMap;

use anyhow::{Result, ensure};
use sha2::{Digest, Sha256};

use crate::tool_registry::{ToolCapability, ToolDescriptor, ToolEffect, ToolPolicy, ToolRegistry};

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

/// A deliberately tiny executor with no ambient host capabilities.
#[derive(Clone, Debug)]
pub struct FixtureExecutor {
    state: BTreeMap<String, String>,
    observed_effects: BTreeMap<String, String>,
    remaining_steps: u16,
    remaining_input_bytes: usize,
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
                Ok(observation(FIXTURE_READ_TOOL, value, evidence_ref, value))
            }
            _ => anyhow::bail!("fixture executor does not implement tool: {}", call.tool_id),
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

pub fn register_fixture_tools(registry: &mut ToolRegistry) -> Result<()> {
    registry.register(ToolDescriptor {
        id: FIXTURE_WRITE_TOOL.to_owned(),
        version: "1.0.0".to_owned(),
        capability: ToolCapability::WorkspaceWrite,
        effect: ToolEffect::Mutating,
        max_input_bytes: MAX_KEY_BYTES + 1 + MAX_VALUE_BYTES,
    })?;
    registry.register(ToolDescriptor {
        id: FIXTURE_READ_TOOL.to_owned(),
        version: "1.0.0".to_owned(),
        capability: ToolCapability::WorkspaceRead,
        effect: ToolEffect::ReadOnly,
        max_input_bytes: MAX_KEY_BYTES,
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
    use super::{
        FIXTURE_READ_TOOL, FIXTURE_WRITE_TOOL, FixtureExecutor, FixtureToolCall,
        register_fixture_tools,
    };
    use crate::tool_registry::{ToolCapability, ToolPolicy, ToolRegistry};

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
}
