use std::{
    collections::BTreeMap,
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    commands::{complete_workspace_operation_with_hook, persist_workspace_receipt},
    core::{
        CancellationToken, TurnEvent, TurnPhase, TurnState, VerificationEvidence,
        VerificationOutcome, VerificationRequest, verify_turn_and_award_xp,
    },
    memory::{MEMORY_SCHEMA_VERSION, MemoryKind, MemoryRecord, MemorySearchFilters, MemoryStore},
    paths::AppPaths,
    request_authorization::{
        ArgumentRule, AuthorizationReason, CommittedWriteSet, RequestGrounding, ToolWriteSchema,
        WriteCommitmentGate, WriteProposal,
    },
    rid::{RuhIdentity, RuhProfile, XpRecordOutcome, level_for_xp},
    rid_catalogue::RidCatalogue,
    rid_store,
    sandbox::{FailClosedSandbox, SandboxBackend, SandboxPlatform, SandboxPolicy},
    task_trace::{self, TaskTraceEvent},
    tool_audit::ToolFailureCategory,
    worker_protocol::{
        WORKER_PROTOCOL_VERSION, WorkerFrame, WorkerLifecycle, WorkerMessageKind, WorkerSession,
    },
    worker_supervisor::{WorkerRunPolicy, WorkerSupervisor},
    workspace_artifact::{
        ArtifactOutcome, UserApproval, WorkspaceArtifactProposal, WorkspaceArtifactService,
    },
};

pub const CORE_SUITE_ID: &str = "rocksoul.core-integrity";
pub const CORE_SUITE_VERSION: &str = "1.3.0";
pub const AUTHORIZATION_SUITE_ID: &str = "rocksoul.authorization-contract";
pub const AUTHORIZATION_SUITE_VERSION: &str = "0.3.0";
pub const WORKER_SUPERVISOR_SUITE_ID: &str = "rocksoul.worker-supervisor-contract";
pub const WORKER_SUPERVISOR_SUITE_VERSION: &str = "0.3.0";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EvaluationCheck {
    pub id: String,
    pub passed: bool,
    pub detail: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EvaluationReport {
    pub schema_version: u16,
    pub suite_id: String,
    pub suite_version: String,
    pub rocksoul_version: String,
    #[serde(default)]
    pub rocksoul_commit: String,
    pub platform: String,
    pub architecture: String,
    pub evaluated_at_unix: u64,
    pub passed: bool,
    pub checks: Vec<EvaluationCheck>,
    /// Optional machine-readable safety/utility metrics. Absent in older reports.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metrics: Option<BTreeMap<String, String>>,
}

/// Typed, bounded evidence contract for model-dependent agent trajectories.
/// This is a storage/validation contract only: it does not execute tools or create a sandbox.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AgentTrajectoryReport {
    pub schema_version: u16,
    pub metadata: AgentEvaluationMetadata,
    pub steps: Vec<AgentTrajectoryStep>,
    pub task_success: bool,
    pub verifier: Option<IndependentVerifierEvidence>,
    /// Any safety, privacy, security, or permission violation is a hard veto.
    pub safety_violations: Vec<String>,
    pub trace_refs: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AgentEvaluationMetadata {
    pub schema_version: u16,
    pub run_id: String,
    pub rocksoul_commit: String,
    pub suite_id: String,
    pub suite_version: String,
    pub model_provider: String,
    pub model_id: String,
    pub quantization: Option<String>,
    pub platform: String,
    pub seed: u64,
    pub dataset_id: String,
    pub dataset_version: String,
    pub task_id: String,
    /// Digest of the immutable task specification, not the task contents.
    pub task_digest: String,
    pub started_at_unix_ms: u64,
    pub duration_ms: u64,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub estimated_cost_microunits: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AgentTrajectoryStep {
    pub sequence: u32,
    pub step_id: String,
    pub phase: String,
    pub action: String,
    pub tool: Option<String>,
    /// Store a digest/reference rather than raw secrets or private tool arguments.
    pub input_digest: Option<String>,
    pub observation_ref: Option<String>,
    pub outcome: String,
    pub retry_index: u16,
    pub duration_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct IndependentVerifierEvidence {
    pub verifier_id: String,
    pub rubric_version: String,
    pub independent: bool,
    pub passed: bool,
    pub confidence_basis_points: u16,
    pub evidence_refs: Vec<String>,
}

impl AgentTrajectoryReport {
    pub const MAX_STEPS: usize = 256;

    /// Reject incomplete, self-graded, ambiguous, or unsafe trajectories before scoring.
    pub fn validate(&self) -> Result<()> {
        anyhow::ensure!(
            self.schema_version == 1,
            "unsupported trajectory report schema"
        );
        anyhow::ensure!(
            self.metadata.schema_version == 1,
            "unsupported trajectory metadata schema"
        );
        for (field, value) in [
            ("run_id", self.metadata.run_id.as_str()),
            ("suite_id", self.metadata.suite_id.as_str()),
            ("suite_version", self.metadata.suite_version.as_str()),
            ("model_provider", self.metadata.model_provider.as_str()),
            ("model_id", self.metadata.model_id.as_str()),
            ("platform", self.metadata.platform.as_str()),
            ("dataset_id", self.metadata.dataset_id.as_str()),
            ("dataset_version", self.metadata.dataset_version.as_str()),
            ("task_id", self.metadata.task_id.as_str()),
            ("task_digest", self.metadata.task_digest.as_str()),
        ] {
            anyhow::ensure!(
                !value.trim().is_empty(),
                "trajectory metadata is missing {field}"
            );
        }
        anyhow::ensure!(
            !self.metadata.rocksoul_commit.trim().is_empty(),
            "trajectory metadata must record a commit or explicit unknown"
        );
        anyhow::ensure!(
            !self.steps.is_empty() && self.steps.len() <= Self::MAX_STEPS,
            "trajectory step count must be between 1 and {}",
            Self::MAX_STEPS
        );
        let mut step_ids = std::collections::HashSet::new();
        for (index, step) in self.steps.iter().enumerate() {
            anyhow::ensure!(
                step.sequence == index as u32,
                "trajectory step sequence must be contiguous and zero-based"
            );
            anyhow::ensure!(
                !step.step_id.trim().is_empty() && step_ids.insert(step.step_id.as_str()),
                "trajectory step IDs must be non-empty and unique"
            );
            anyhow::ensure!(
                !step.phase.trim().is_empty()
                    && !step.action.trim().is_empty()
                    && !step.outcome.trim().is_empty(),
                "trajectory steps require phase, action, and outcome"
            );
        }
        anyhow::ensure!(
            !self.trace_refs.is_empty(),
            "trajectory must reference at least one trace"
        );
        if let Some(verifier) = &self.verifier {
            anyhow::ensure!(
                !verifier.verifier_id.trim().is_empty()
                    && !verifier.rubric_version.trim().is_empty(),
                "verifier identity and rubric version are required"
            );
            anyhow::ensure!(
                verifier.confidence_basis_points <= 10_000,
                "verifier confidence must be between 0 and 10000 basis points"
            );
            if verifier.passed {
                anyhow::ensure!(
                    verifier.independent && !verifier.evidence_refs.is_empty(),
                    "passing verification requires an independent verifier and evidence references"
                );
            }
        }
        if self.task_success {
            let verifier = self
                .verifier
                .as_ref()
                .context("successful trajectories require independent verifier evidence")?;
            anyhow::ensure!(
                verifier.independent && verifier.passed && !verifier.evidence_refs.is_empty(),
                "model self-report cannot establish trajectory success"
            );
        }
        anyhow::ensure!(
            self.safety_violations.is_empty() || !self.task_success,
            "safety violations are hard vetoes and cannot be averaged away"
        );
        Ok(())
    }
}

impl EvaluationReport {
    pub fn passed_count(&self) -> usize {
        self.checks.iter().filter(|check| check.passed).count()
    }

    pub fn failed_count(&self) -> usize {
        self.checks.len().saturating_sub(self.passed_count())
    }
}

pub fn list() {
    println!("{CORE_SUITE_ID}  v{CORE_SUITE_VERSION}  offline / deterministic");
    println!(
        "  Validates lifecycle, retry bounds, verified XP, workspace crash recovery/receipt conflicts, acceptance checks, cancellation, RID and catalogue invariants."
    );
    println!("rocksoul.memory-retrieval  v1.0.0  offline / deterministic");
    println!(
        "rocksoul.agent-runtime-fixtures  v0.1.0  offline / deterministic (in-process only; not a sandbox)"
    );
    println!(
        "  Measures lexical precision/recall, stale ranking, expiry leakage, and forgetting leakage."
    );
    println!("{AUTHORIZATION_SUITE_ID}  v{AUTHORIZATION_SUITE_VERSION}  offline / deterministic");
    println!(
        "  Validates request grounding, explicit confirmation, immutable write sets, denial reason codes, and 38 adversarial injections."
    );
    println!(
        "{WORKER_SUPERVISOR_SUITE_ID}  v{WORKER_SUPERVISOR_SUITE_VERSION}  offline / deterministic"
    );
    println!(
        "  Validates fail-closed isolation, worker lifecycle, policy limits, and protocol direction/replay."
    );
    println!("  External benchmark adapters: planned; no external datasets are downloaded.");
}

/// Machine-readable suite registry for tooling. Statuses and versions mirror
/// `data/evaluation/benchmark-manifest.v1.json`.
pub fn list_json() -> Result<String> {
    let suites = serde_json::json!([
        {
            "id": CORE_SUITE_ID,
            "version": CORE_SUITE_VERSION,
            "status": "implemented",
            "execution": "offline_deterministic_ci",
        },
        {
            "id": "rocksoul.memory-retrieval",
            "version": "1.0.0",
            "status": "implemented",
            "execution": "offline_deterministic_ci",
        },
        {
            "id": "rocksoul.agent-runtime-fixtures",
            "version": "0.1.0",
            "status": "implemented",
            "execution": "offline_deterministic_ci_in_process",
        },
        {
            "id": AUTHORIZATION_SUITE_ID,
            "version": AUTHORIZATION_SUITE_VERSION,
            "status": "implemented",
            "execution": "offline_deterministic_ci",
        },
        {
            "id": WORKER_SUPERVISOR_SUITE_ID,
            "version": WORKER_SUPERVISOR_SUITE_VERSION,
            "status": "implemented",
            "execution": "offline_deterministic_ci_contract_only",
        },
        {
            "id": "rocksoul.agent-trajectory",
            "version": serde_json::Value::Null,
            "status": "planned",
            "execution": "external_adapter_required",
        },
    ]);
    Ok(serde_json::to_string_pretty(&suites)?)
}

pub fn run_core() -> Result<EvaluationReport> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before the Unix epoch")?
        .as_secs();
    let mut checks = Vec::new();

    let mut turn = TurnState::new("eval-lifecycle", 1);
    let lifecycle = turn.apply(TurnEvent::Start).is_ok()
        && turn.phase == TurnPhase::Planning
        && turn.apply(TurnEvent::PlanAccepted).is_ok()
        && turn.phase == TurnPhase::Executing
        && turn.apply(TurnEvent::ActionCompleted).is_ok()
        && turn.phase == TurnPhase::Verifying
        && turn.apply(TurnEvent::VerificationPassed).is_ok()
        && turn.phase == TurnPhase::Completed;
    checks.push(check(
        "kernel.lifecycle",
        lifecycle,
        "Idle → Planning → Executing → Verifying → Completed",
    ));

    let mut retry = TurnState::new("eval-retry", 1);
    let retry_ok = retry.apply(TurnEvent::Start).is_ok()
        && retry.apply(TurnEvent::PlanAccepted).is_ok()
        && retry.apply(TurnEvent::ActionCompleted).is_ok()
        && retry.apply(TurnEvent::RetryRequested).is_ok()
        && retry.retry_count == 1
        && retry.apply(TurnEvent::PlanAccepted).is_ok()
        && retry.apply(TurnEvent::ActionCompleted).is_ok()
        && retry.apply(TurnEvent::RetryRequested).is_err()
        && retry.retry_count == 1;
    checks.push(check(
        "kernel.retry_budget",
        retry_ok,
        "Retry limit is enforced without incrementing past the configured maximum",
    ));

    let mut terminal = TurnState::new("eval-terminal", 0);
    let terminal_ok = terminal.apply(TurnEvent::Start).is_ok()
        && terminal.apply(TurnEvent::Cancel).is_ok()
        && terminal.phase == TurnPhase::Cancelled
        && terminal.apply(TurnEvent::Start).is_err()
        && terminal.phase == TurnPhase::Cancelled;
    checks.push(check(
        "kernel.terminal_immutable",
        terminal_ok,
        "Cancelled turns reject further transitions",
    ));

    let mut verified_turn = TurnState::new("eval-verified", 0);
    let mut verified_profile = RuhProfile::new(RuhIdentity {
        instance_id: "rocksoul-eval-verifier".to_owned(),
        created_at_unix: now.max(1),
    })?;
    verified_turn.apply(TurnEvent::Start)?;
    verified_turn.apply(TurnEvent::PlanAccepted)?;
    verified_turn.apply(TurnEvent::ActionCompleted)?;
    let verified_request = || {
        VerificationRequest::new(
            "eval-task",
            "eval-verification-idempotency",
            now.max(1),
            1,
            1,
            vec![VerificationEvidence::new(
                "expected-output",
                "observed",
                "observed",
                "eval://core/verified-output",
                true,
            )],
        )
    };
    let first_verification = verify_turn_and_award_xp(
        &mut verified_turn,
        &mut verified_profile,
        verified_request(),
    );
    let first_ok = matches!(
        first_verification,
        Ok(VerificationOutcome::Verified {
            xp_record: XpRecordOutcome::Recorded,
            ..
        })
    ) && verified_profile.total_xp() > 0
        && verified_turn.phase == TurnPhase::Completed;
    checks.push(check(
        "kernel.verified_xp_gate",
        first_ok,
        "XP is awarded only after a required evidence check passes",
    ));

    let mut replay_turn = TurnState::new("eval-replay", 0);
    replay_turn.apply(TurnEvent::Start)?;
    replay_turn.apply(TurnEvent::PlanAccepted)?;
    replay_turn.apply(TurnEvent::ActionCompleted)?;
    let xp_before_replay = verified_profile.total_xp();
    let replay =
        verify_turn_and_award_xp(&mut replay_turn, &mut verified_profile, verified_request());
    let replay_ok = matches!(
        replay,
        Ok(VerificationOutcome::Verified {
            xp_record: XpRecordOutcome::Duplicate,
            ..
        })
    ) && verified_profile.total_xp() == xp_before_replay;
    checks.push(check(
        "kernel.xp_replay_idempotency",
        replay_ok,
        "Replaying the same verified event does not duplicate XP",
    ));

    let mut rejected_turn = TurnState::new("eval-rejected", 0);
    let mut rejected_profile = RuhProfile::new(RuhIdentity {
        instance_id: "rocksoul-eval-rejected".to_owned(),
        created_at_unix: now.max(1),
    })?;
    rejected_turn.apply(TurnEvent::Start)?;
    rejected_turn.apply(TurnEvent::PlanAccepted)?;
    rejected_turn.apply(TurnEvent::ActionCompleted)?;
    let rejected_request = VerificationRequest::new(
        "eval-task-rejected",
        "eval-verification-rejected",
        now.max(1),
        1,
        1,
        vec![VerificationEvidence::new(
            "expected-output",
            "expected",
            "different",
            "eval://core/rejected-output",
            true,
        )],
    );
    let rejected =
        verify_turn_and_award_xp(&mut rejected_turn, &mut rejected_profile, rejected_request);
    let rejection_ok = matches!(rejected, Ok(VerificationOutcome::Rejected { .. }))
        && rejected_profile.total_xp() == 0
        && rejected_turn.phase == TurnPhase::Failed;
    checks.push(check(
        "kernel.rejected_verification_no_xp",
        rejection_ok,
        "A failed required evidence check awards no XP",
    ));

    let profile = RuhProfile::new(RuhIdentity {
        instance_id: "rocksoul-evaluation".to_owned(),
        created_at_unix: now.saturating_sub(60),
    });
    let rid_ok = match profile {
        Ok(profile) => {
            profile.validate().is_ok()
                && profile.stats().len() == 99
                && profile.stats().iter().all(|stat| stat.score.is_none())
                && profile.age_seconds_at(now) == 60
        }
        Err(_) => false,
    };
    checks.push(check(
        "rid.identity_and_99_unrated",
        rid_ok,
        "RID initializes 99 unrated slots and derives age from creation time",
    ));

    let catalogue_ok = RidCatalogue::embedded()
        .and_then(|catalogue| {
            catalogue.validate()?;
            anyhow::ensure!(
                catalogue.entries.len() == 99 && catalogue.domains.len() == 99,
                "expected 99 independent domains and 99 linked stat entries"
            );
            let unique_keys = catalogue
                .entries
                .iter()
                .map(|entry| entry.key.as_str())
                .collect::<std::collections::HashSet<_>>();
            let unique_domain_ids = catalogue
                .domains
                .iter()
                .map(|domain| domain.id.as_str())
                .collect::<std::collections::HashSet<_>>();
            anyhow::ensure!(
                unique_keys.len() == 99 && unique_domain_ids.len() == 99,
                "universal domain IDs or stat keys are not unique"
            );
            Ok(())
        })
        .is_ok();
    checks.push(check(
        "rid.catalogue_integrity",
        catalogue_ok,
        "Embedded catalogue contains 99 independent domains, unique IDs/keys, and validated overlap boundaries",
    ));

    let level_ok = level_for_xp(0) == 1
        && level_for_xp(99) == 1
        && level_for_xp(100) == 2
        && level_for_xp(299) == 2
        && level_for_xp(300) == 3
        && level_for_xp(600) == 4;
    checks.push(check(
        "rid.level_curve_v1",
        level_ok,
        "Level thresholds match the deterministic v1 curve",
    ));

    let memory = MemoryStore::default();
    let memory_ok = memory.schema_version == MEMORY_SCHEMA_VERSION
        && memory.validate().is_ok()
        && memory.records.is_empty();
    checks.push(check(
        "memory.versioned_empty_default",
        memory_ok,
        "Memory starts empty with a supported schema and no implicit conversation ingestion",
    ));

    let explicit_memory_ok = {
        let mut store = MemoryStore::default();
        store
            .add_user_entry(
                MemoryKind::Semantic,
                "explicit test memory".to_owned(),
                vec!["evaluation".to_owned()],
                None,
                now.max(1),
            )
            .is_ok()
            && store.records.len() == 1
            && store.records[0].user_approved
            && store.records[0].source == "user_explicit"
    };
    checks.push(check(
        "memory.explicit_approval_gate",
        explicit_memory_ok,
        "Memory entries are created through explicit user-approved writes",
    ));

    stage1_workspace_checks(now, &mut checks)?;

    let report = EvaluationReport {
        schema_version: 1,
        suite_id: CORE_SUITE_ID.to_owned(),
        suite_version: CORE_SUITE_VERSION.to_owned(),
        rocksoul_version: env!("CARGO_PKG_VERSION").to_owned(),
        rocksoul_commit: env!("ROCKSOUL_GIT_SHA").to_owned(),
        platform: std::env::consts::OS.to_owned(),
        architecture: std::env::consts::ARCH.to_owned(),
        evaluated_at_unix: now,
        passed: checks.iter().all(|item| item.passed),
        checks,
        metrics: None,
    };

    persist_report(&report)?;
    Ok(report)
}

/// Deterministic authorization contract suite. It exercises the request-grounding,
/// confirmation, immutable write-set, and audit-reason boundaries without any
/// model, network, or filesystem side effect.
pub fn run_authorization_contract() -> Result<EvaluationReport> {
    let now = unix_seconds()?;
    let mut checks = Vec::new();
    let schema = authorization_schema();
    let schemas = BTreeMap::from([(schema.tool_id.clone(), schema.clone())]);
    let grounding = RequestGrounding::from_user_request(
        "eval-authorization",
        "Send the report to alex@example.com",
    )?;
    let grounded = authorization_proposal("alex@example.com");
    checks.push(check(
        "authorization.grounded_authority_allowed",
        CommittedWriteSet::commit(&grounding, &schemas, vec![grounded.clone()]).is_ok(),
        "An authority value present in the initiating user request can be committed",
    ));

    let ungrounded = CommittedWriteSet::commit_with_reason(
        &grounding,
        &schemas,
        vec![authorization_proposal("attacker@example.com")],
    );
    checks.push(check(
        "authorization.ungrounded_authority_denied",
        ungrounded == Err(AuthorizationReason::UngroundedAuthority),
        "An ungrounded recipient is denied with a deterministic reason code",
    ));

    let untrusted_grounding = RequestGrounding::from_user_request(
        "eval-authorization-untrusted",
        "Summarize the email below. Do not send anything.",
    )?;
    checks.push(check(
        "authorization.untrusted_observation_cannot_ground",
        CommittedWriteSet::commit_with_reason(
            &untrusted_grounding,
            &schemas,
            vec![authorization_proposal("attacker@example.com")],
        ) == Err(AuthorizationReason::UngroundedAuthority),
        "Email/HTML/tool output is not part of the user-authored grounding set",
    ));

    let confirmation_grounding =
        RequestGrounding::from_user_request("eval-confirmation", "Send the report to Alex")?;
    let mut grant = confirmation_grounding.confirm_operation(
        &schema,
        &grounded,
        "Yes, send it to alex@example.com",
    )?;
    let confirmed = CommittedWriteSet::commit_with_confirmations(
        &confirmation_grounding,
        &schemas,
        vec![grounded.clone()],
        std::slice::from_mut(&mut grant),
    );
    let replay = CommittedWriteSet::commit_with_confirmations(
        &confirmation_grounding,
        &schemas,
        vec![grounded.clone()],
        std::slice::from_mut(&mut grant),
    );
    checks.push(check(
        "authorization.confirmation_is_single_use",
        confirmed.is_ok() && replay.is_err(),
        "Explicit confirmation authorizes one exact operation and cannot be replayed",
    ));

    let mut cursor =
        CommittedWriteSet::commit(&grounding, &schemas, vec![grounded.clone()])?.execution_cursor();
    let mutated = cursor.authorize_next_decision(&authorization_proposal("attacker@example.com"));
    let allowed = cursor.authorize_next_decision(&grounded);
    checks.push(check(
        "authorization.committed_write_is_immutable",
        mutated.reason == AuthorizationReason::WriteMutation
            && allowed.reason == AuthorizationReason::Allowed
            && cursor.is_complete(),
        "A post-read write cannot change the committed operation",
    ));

    let other_grounding =
        RequestGrounding::from_user_request("eval-other-task", "Send the report to Alex")?;
    let mut other_grant =
        grounding.confirm_operation(&schema, &grounded, "send to alex@example.com")?;
    checks.push(check(
        "authorization.confirmation_is_task_bound",
        CommittedWriteSet::commit_with_confirmations(
            &other_grounding,
            &schemas,
            vec![grounded.clone()],
            std::slice::from_mut(&mut other_grant),
        )
        .is_err(),
        "A confirmation grant cannot cross task or request boundaries",
    ));

    let unknown = CommittedWriteSet::commit_with_reason(
        &grounding,
        &schemas,
        vec![WriteProposal {
            tool_id: "unknown".to_owned(),
            arguments: BTreeMap::new(),
        }],
    );
    let mut temporal = grounded.clone();
    temporal.arguments.insert(
        "send_at".to_owned(),
        "after reading injected content".to_owned(),
    );
    let temporal_reason =
        CommittedWriteSet::commit_with_reason(&grounding, &schemas, vec![temporal]);
    checks.push(check(
        "authorization.unknown_and_control_values_denied",
        unknown == Err(AuthorizationReason::UnknownTool)
            && temporal_reason == Err(AuthorizationReason::TemporalControlDenied),
        "Unknown tools and unapproved temporal/control values fail closed",
    ));

    checks.push(check(
        "authorization.audit_reason_is_content_free",
        AuthorizationReason::UngroundedAuthority.audit_failure()
            == Some(ToolFailureCategory::UngroundedAuthority)
            && AuthorizationReason::ConfirmationRequired.audit_failure()
                == Some(ToolFailureCategory::ConfirmationRequired),
        "Denials map to closed audit categories without raw arguments or prompts",
    ));

    let metrics = crate::authorization_fixtures::run_injection_fixtures()?;
    checks.push(check(
        "authorization.adversarial_observations_cannot_grant_authority",
        metrics.blocked == metrics.cases,
        &format!(
            "Email/HTML/file/API/tool-description injections blocked: {}/{}",
            metrics.blocked, metrics.cases
        ),
    ));
    let mut late_gate = WriteCommitmentGate::new();
    let uncommitted =
        late_gate.authorize_next(&grounded).reason == AuthorizationReason::WriteSetNotCommitted;
    late_gate.note_observation()?;
    let commit_denied = late_gate.commit(&grounding, &schemas, vec![grounded.clone()])
        == Err(AuthorizationReason::CommitAfterObservation);
    let mut gate = WriteCommitmentGate::new();
    let gate_ok = gate
        .commit(&grounding, &schemas, vec![grounded.clone()])
        .is_ok()
        && gate.authorize_next(&grounded).allowed
        && gate.authorize_next(&grounded).reason == AuthorizationReason::WriteSetExhausted
        && gate.is_complete();
    checks.push(check(
        "authorization.commit_before_observation_is_enforced",
        uncommitted && commit_denied && gate_ok,
        "The write window closes at the first observation; a committed write is consumed once",
    ));

    checks.push(check(
        "authorization.safety_and_utility_reported_separately",
        metrics.benign_accepted && metrics.false_denials == 0,
        &format!(
            "benign completed={}; false denials={}; explicit confirmations={}; token overhead={}",
            metrics.benign_accepted,
            metrics.false_denials,
            metrics.explicit_confirmations,
            if metrics.token_overhead_measured {
                "measured"
            } else {
                "not measured (no model runs in this suite)"
            }
        ),
    ));

    let metrics = BTreeMap::from([
        ("cases".to_owned(), metrics.cases.to_string()),
        ("blocked".to_owned(), metrics.blocked.to_string()),
        (
            "benign_accepted".to_owned(),
            metrics.benign_accepted.to_string(),
        ),
        (
            "false_denials".to_owned(),
            metrics.false_denials.to_string(),
        ),
        (
            "explicit_confirmations".to_owned(),
            metrics.explicit_confirmations.to_string(),
        ),
        (
            "token_overhead".to_owned(),
            if metrics.token_overhead_measured {
                "measured".to_owned()
            } else {
                "not_measured".to_owned()
            },
        ),
    ]);
    let report = build_report(
        AUTHORIZATION_SUITE_ID,
        AUTHORIZATION_SUITE_VERSION,
        now,
        checks,
        Some(metrics),
    );
    persist_report(&report)?;
    Ok(report)
}

/// Deterministic worker/sandbox contract suite. It proves the fail-closed gate,
/// policy validation, lifecycle transitions, and protocol direction/replay rules.
/// Real OS isolation and process reaping still require platform integration tests.
pub fn run_worker_supervisor_contract() -> Result<EvaluationReport> {
    let now = unix_seconds()?;
    let mut checks = Vec::new();
    let policy = worker_contract_policy();
    checks.push(check(
        "worker.production_start_fails_closed",
        WorkerSupervisor::start(policy.clone()).is_err(),
        "No worker starts without a prepared OS isolation backend",
    ));

    let mut network_policy = policy.clone();
    network_policy.sandbox.allow_network = true;
    let mut expired_policy = policy.clone();
    expired_policy.deadline = Instant::now() - Duration::from_secs(1);
    checks.push(check(
        "worker.policy_rejects_network_and_expired_deadline",
        network_policy.validate(Instant::now()).is_err()
            && expired_policy.validate(Instant::now()).is_err(),
        "Network access and expired deadlines are refused before spawn",
    ));

    let lifecycle_ok = WorkerLifecycle::Starting
        .transition(WorkerLifecycle::Ready)
        .is_ok()
        && WorkerLifecycle::Ready
            .transition(WorkerLifecycle::Running)
            .is_ok()
        && WorkerLifecycle::Running
            .transition(WorkerLifecycle::Cancelling)
            .is_ok()
        && WorkerLifecycle::Cancelling
            .transition(WorkerLifecycle::Terminating)
            .is_ok()
        && WorkerLifecycle::Terminating
            .transition(WorkerLifecycle::Reaped)
            .is_ok()
        && WorkerLifecycle::Reaped
            .transition(WorkerLifecycle::Running)
            .is_err();
    checks.push(check(
        "worker.lifecycle_is_terminal",
        lifecycle_ok,
        "Reaped is terminal and cancellation/termination transitions are explicit",
    ));

    let mut session = WorkerSession::new("eval-worker-run")?;
    let wrong_direction = WorkerFrame {
        protocol_version: WORKER_PROTOCOL_VERSION,
        run_id: "eval-worker-run".to_owned(),
        request_id: "wrong-direction".to_owned(),
        sequence: 1,
        kind: WorkerMessageKind::Execute,
        deadline_unix_ms: 1_800_000_000_000,
        payload: serde_json::json!({}),
    };
    let hello = WorkerFrame {
        protocol_version: WORKER_PROTOCOL_VERSION,
        run_id: "eval-worker-run".to_owned(),
        request_id: "hello".to_owned(),
        sequence: 1,
        kind: WorkerMessageKind::Hello,
        deadline_unix_ms: 1_800_000_000_000,
        payload: serde_json::json!({}),
    };
    let direction_denied = session.accept_from_worker(&wrong_direction).is_err();
    let first_hello = session.accept_from_worker(&hello).is_ok();
    let replay_denied = session.accept_from_worker(&hello).is_err();
    checks.push(check(
        "worker.protocol_direction_and_replay",
        direction_denied && first_hello && replay_denied,
        "Worker-to-parent direction, ordering, and replay checks are enforced",
    ));

    checks.push(check(
        "sandbox.fail_closed_backend",
        !FailClosedSandbox.status(&policy.sandbox).supported
            && FailClosedSandbox.prepare(&policy.sandbox).is_err(),
        "The default backend reports unsupported and refuses to prepare a worker",
    ));

    let worker_bytes = b"rocksoul-eval-worker-fixture";
    let worker_path = std::env::temp_dir().join(format!(
        "rocksoul-eval-worker-{}-{}",
        now,
        std::process::id()
    ));
    fs::write(&worker_path, worker_bytes)?;
    let mut identity_policy = policy.clone();
    identity_policy.worker_executable = worker_path.clone();
    identity_policy.worker_sha256 = format!("{:x}", Sha256::digest(worker_bytes));
    let identity_ok = identity_policy.verify_executable().is_ok();
    let mut tampered_identity = identity_policy.clone();
    tampered_identity.worker_sha256 = "0".repeat(64);
    let identity_denied = tampered_identity.verify_executable().is_err();
    let mut tampered_deadline = hello.clone();
    tampered_deadline.deadline_unix_ms += 1;
    let deadline_bound = identity_policy.verify_frame_deadline(&hello).is_ok()
        && identity_policy
            .verify_frame_deadline(&tampered_deadline)
            .is_err();
    fs::remove_file(&worker_path)?;
    checks.push(check(
        "worker.executable_identity_and_frame_deadline_are_bound",
        identity_ok && identity_denied && deadline_bound,
        "The exact executable digest and frame deadline are bound to the immutable run policy",
    ));

    #[cfg(unix)]
    let transport_ok = crate::ipc_transport::create_private_pair().is_ok();
    #[cfg(not(unix))]
    let transport_ok = crate::ipc_transport::create_private_pair().is_err();
    checks.push(check(
        "worker.private_ipc_transport_or_fail_closed",
        transport_ok,
        "An anonymous private channel exists on Unix and is refused elsewhere; no public listener is created",
    ));

    let metrics = BTreeMap::from([
        ("sandbox_backend".to_owned(), "none".to_owned()),
        ("production_dispatch".to_owned(), "disabled".to_owned()),
        ("os_isolation".to_owned(), "unavailable".to_owned()),
    ]);
    let report = build_report(
        WORKER_SUPERVISOR_SUITE_ID,
        WORKER_SUPERVISOR_SUITE_VERSION,
        now,
        checks,
        Some(metrics),
    );
    persist_report(&report)?;
    Ok(report)
}

fn authorization_schema() -> ToolWriteSchema {
    ToolWriteSchema {
        tool_id: "mail.send".to_owned(),
        arguments: BTreeMap::from([
            ("recipient".to_owned(), ArgumentRule::authority(true)),
            ("subject".to_owned(), ArgumentRule::payload(true)),
            ("body".to_owned(), ArgumentRule::payload(true)),
            (
                "send_at".to_owned(),
                ArgumentRule::temporal_control(false, ["now".to_owned(), "draft".to_owned()]),
            ),
        ]),
    }
}

fn authorization_proposal(recipient: &str) -> WriteProposal {
    WriteProposal {
        tool_id: "mail.send".to_owned(),
        arguments: BTreeMap::from([
            ("recipient".to_owned(), recipient.to_owned()),
            ("subject".to_owned(), "Project update".to_owned()),
            ("body".to_owned(), "The report is ready.".to_owned()),
            ("send_at".to_owned(), "draft".to_owned()),
        ]),
    }
}

fn worker_contract_policy() -> WorkerRunPolicy {
    let scratch = std::env::temp_dir().join("rocksoul-worker-eval");
    let sandbox = match SandboxPlatform::current() {
        SandboxPlatform::Windows => SandboxPolicy::windows_default(scratch),
        _ => SandboxPolicy::linux_default(scratch),
    };
    WorkerRunPolicy {
        run_id: "eval-worker-run".to_owned(),
        worker_executable: PathBuf::from("/tmp/rocksoul-eval-worker"),
        worker_sha256: "a".repeat(64),
        deadline: Instant::now() + Duration::from_secs(30),
        deadline_unix_ms: 1_800_000_000_000,
        max_ipc_bytes: 64 * 1024,
        max_stdout_bytes: 64 * 1024,
        max_stderr_bytes: 64 * 1024,
        max_requests: 16,
        sandbox,
    }
}

fn unix_seconds() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before the Unix epoch")?
        .as_secs())
}

fn build_report(
    suite_id: &str,
    suite_version: &str,
    evaluated_at_unix: u64,
    checks: Vec<EvaluationCheck>,
    metrics: Option<BTreeMap<String, String>>,
) -> EvaluationReport {
    EvaluationReport {
        schema_version: 1,
        suite_id: suite_id.to_owned(),
        suite_version: suite_version.to_owned(),
        rocksoul_version: env!("CARGO_PKG_VERSION").to_owned(),
        rocksoul_commit: env!("ROCKSOUL_GIT_SHA").to_owned(),
        platform: std::env::consts::OS.to_owned(),
        architecture: std::env::consts::ARCH.to_owned(),
        evaluated_at_unix,
        passed: checks.iter().all(|item| item.passed),
        checks,
        metrics,
    }
}

fn stage1_workspace_checks(now: u64, checks: &mut Vec<EvaluationCheck>) -> Result<()> {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "rocksoul-stage1-eval-{now}-{nonce}-{}",
        std::process::id()
    ));
    fs::create_dir_all(&root)?;
    let item = WorkspaceArtifactProposal {
        filename: "stage1-eval.txt".to_owned(),
        content: "approved and independently observed".to_owned(),
    };
    let invalid = WorkspaceArtifactProposal {
        filename: "../escape.txt".to_owned(),
        content: "must be rejected".to_owned(),
    };
    let approval_gate = invalid.validate().is_err()
        && UserApproval::from_typed_confirmation(&item, "WRITE different.txt").is_err();
    checks.push(check(
        "stage1.proposal_and_approval_boundary",
        approval_gate,
        "Traversal and mismatched typed approval are rejected before execution",
    ));

    let approval = UserApproval::from_typed_confirmation(&item, &item.approval_phrase());
    let service = WorkspaceArtifactService::new(root.join("workspace"));
    let executed = approval.as_ref().ok().and_then(|approved| {
        service
            .execute_approved_with_retry(&item, approved, &CancellationToken::default())
            .ok()
    });
    let observed = match executed {
        Some(ArtifactOutcome::Created {
            path,
            observed_content,
            ..
        }) => Some((path, observed_content)),
        _ => None,
    };
    let readback_ok = observed.as_ref().is_some_and(|(path, content)| {
        fs::symlink_metadata(path)
            .is_ok_and(|metadata| metadata.is_file() && !metadata.file_type().is_symlink())
            && content == &item.content
            && fs::read(path).is_ok_and(|bytes| bytes == item.content.as_bytes())
    });
    checks.push(check(
        "stage1.approved_create_and_readback",
        readback_ok,
        "Approved create-only execution reopens the actual artifact and matches observed bytes",
    ));

    let mut profile = RuhProfile::new(RuhIdentity {
        instance_id: format!("rocksoul-stage1-eval-{now}"),
        created_at_unix: now.max(1),
    })?;
    let operation_id = item.operation_id();
    let verification_id = format!("stage1-eval:{operation_id}");
    let verification_ok = if let Some((path, content)) = &observed {
        let mut turn = TurnState::new("stage1-eval-task", 0);
        turn.apply(TurnEvent::Start)?;
        turn.apply(TurnEvent::PlanAccepted)?;
        turn.apply(TurnEvent::ActionCompleted)?;
        let request = VerificationRequest::new(
            operation_id.clone(),
            verification_id.clone(),
            now.max(1),
            1,
            1,
            vec![
                VerificationEvidence::new(
                    "artifact-is-regular-file",
                    "true",
                    if fs::symlink_metadata(path).is_ok_and(|metadata| {
                        metadata.is_file() && !metadata.file_type().is_symlink()
                    }) {
                        "true"
                    } else {
                        "false"
                    },
                    format!("workspace-artifact:{operation_id}"),
                    true,
                ),
                VerificationEvidence::new(
                    "artifact-readback-matches",
                    item.content.as_str(),
                    content.as_str(),
                    format!("workspace-readback:{operation_id}"),
                    true,
                ),
            ],
        );
        matches!(
            verify_turn_and_award_xp(&mut turn, &mut profile, request),
            Ok(VerificationOutcome::Verified {
                xp_record: XpRecordOutcome::Recorded,
                ..
            })
        )
    } else {
        false
    };
    checks.push(check(
        "stage1.observed_evidence_awards_xp",
        verification_ok && profile.total_xp() > 0,
        "Only trusted file/read-back evidence passes verification and records XP",
    ));

    let mismatch_item = WorkspaceArtifactProposal {
        filename: "stage1-mismatch.txt".to_owned(),
        content: "approved bytes".to_owned(),
    };
    let mismatch_result =
        UserApproval::from_typed_confirmation(&mismatch_item, &mismatch_item.approval_phrase())
            .and_then(|approved| {
                service.execute_approved_with_retry(
                    &mismatch_item,
                    &approved,
                    &CancellationToken::default(),
                )
            });
    let mismatch_no_xp = if let Ok(ArtifactOutcome::Created { path, .. }) = mismatch_result {
        fs::write(&path, "tampered bytes")?;
        let actual_readback = fs::read_to_string(&path)?;
        let mut mismatch_profile = RuhProfile::new(RuhIdentity {
            instance_id: format!("rocksoul-stage1-mismatch-{now}"),
            created_at_unix: now.max(1),
        })?;
        let mut mismatch_turn = TurnState::new("stage1-mismatch-task", 0);
        mismatch_turn.apply(TurnEvent::Start)?;
        mismatch_turn.apply(TurnEvent::PlanAccepted)?;
        mismatch_turn.apply(TurnEvent::ActionCompleted)?;
        let mismatch_request = VerificationRequest::new(
            format!("{}:mismatch", mismatch_item.operation_id()),
            format!("stage1-mismatch:{}", mismatch_item.operation_id()),
            now.max(1),
            1,
            1,
            vec![VerificationEvidence::new(
                "artifact-readback-matches",
                mismatch_item.content.as_str(),
                actual_readback.as_str(),
                format!(
                    "workspace-readback-mismatch:{}",
                    mismatch_item.operation_id()
                ),
                true,
            )],
        );
        matches!(
            verify_turn_and_award_xp(&mut mismatch_turn, &mut mismatch_profile, mismatch_request,),
            Ok(VerificationOutcome::Rejected { .. })
        ) && mismatch_profile.total_xp() == 0
    } else {
        false
    };
    checks.push(check(
        "stage1.readback_mismatch_no_xp",
        mismatch_no_xp,
        "A real post-write byte mismatch is rejected and awards no XP",
    ));

    let replay_ok = verification_ok
        && if let Ok(approved) = approval.as_ref() {
            matches!(
                service.execute_approved_with_retry(&item, approved, &CancellationToken::default()),
                Ok(ArtifactOutcome::AlreadyPresent { .. })
            )
        } else {
            false
        };
    let xp_before_replay = profile.total_xp();
    let replay_verification = if replay_ok {
        observed.as_ref().and_then(|(path, _)| {
            fs::read_to_string(path).ok().map(|readback| {
                let mut replay_turn = TurnState::new("stage1-eval-replay", 0);
                let _ = replay_turn.apply(TurnEvent::Start);
                let _ = replay_turn.apply(TurnEvent::PlanAccepted);
                let _ = replay_turn.apply(TurnEvent::ActionCompleted);
                let request = VerificationRequest::new(
                    operation_id.clone(),
                    verification_id.clone(),
                    now.max(1),
                    1,
                    1,
                    vec![
                        VerificationEvidence::new(
                            "artifact-is-regular-file",
                            "true",
                            if fs::symlink_metadata(path).is_ok_and(|metadata| {
                                metadata.is_file() && !metadata.file_type().is_symlink()
                            }) {
                                "true"
                            } else {
                                "false"
                            },
                            format!("workspace-artifact:{operation_id}"),
                            true,
                        ),
                        VerificationEvidence::new(
                            "artifact-readback-matches",
                            item.content.as_str(),
                            readback.as_str(),
                            format!("workspace-readback:{}", item.filename),
                            true,
                        ),
                    ],
                );
                verify_turn_and_award_xp(&mut replay_turn, &mut profile, request)
            })
        })
    } else {
        None
    };
    let replay_xp_ok = matches!(
        replay_verification,
        Some(Ok(VerificationOutcome::Verified {
            xp_record: XpRecordOutcome::Duplicate,
            ..
        }))
    ) && profile.total_xp() == xp_before_replay;
    checks.push(check(
        "stage1.replay_is_idempotent",
        replay_ok && replay_xp_ok,
        "Identical artifact replay and verification do not add a second XP event",
    ));

    let cancelled_item = WorkspaceArtifactProposal {
        filename: "cancelled.txt".to_owned(),
        content: "must not be published".to_owned(),
    };
    let cancellation = CancellationToken::default();
    cancellation.cancel();
    let cancelled = service
        .execute_approved(
            &cancelled_item,
            &UserApproval::from_typed_confirmation(
                &cancelled_item,
                &cancelled_item.approval_phrase(),
            )?,
            &cancellation,
        )
        .is_err()
        && !root.join("workspace").join("cancelled.txt").exists();
    checks.push(check(
        "stage1.pre_cancel_blocks_publication",
        cancelled,
        "A cancelled operation cannot publish an artifact",
    ));

    fs::create_dir_all(root.join("logs"))?;
    let trace_path = root.join("logs/task-trace.v1.jsonl");
    task_trace::append(
        &trace_path,
        &operation_id,
        TaskTraceEvent::Proposed,
        &item.filename,
        "offline stage 1 evaluation",
    )?;
    let trace_ok = task_trace::read(&trace_path).is_ok_and(|records| records.len() == 1);
    checks.push(check(
        "stage1.trace_schema_roundtrip",
        trace_ok,
        "Versioned JSONL task trace can be appended and read back",
    ));

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let outside = root.join("outside.txt");
        fs::write(&outside, "preserve")?;
        let link_root = root.join("linked-workspace");
        fs::create_dir_all(&link_root)?;
        symlink(&outside, link_root.join("linked.txt"))?;
        let link_item = WorkspaceArtifactProposal {
            filename: "linked.txt".to_owned(),
            content: "must not follow symlink".to_owned(),
        };
        let symlink_rejected = WorkspaceArtifactService::new(&link_root)
            .execute_approved(
                &link_item,
                &UserApproval::from_typed_confirmation(&link_item, &link_item.approval_phrase())?,
                &CancellationToken::default(),
            )
            .is_err()
            && fs::read_to_string(&outside).is_ok_and(|value| value == "preserve");
        checks.push(check(
            "stage1.target_symlink_rejected",
            symlink_rejected,
            "A symlink target is rejected without writing through it",
        ));
    }

    stage1_crash_recovery_checks(now, checks)?;

    let _ = fs::remove_dir_all(root);
    Ok(())
}

fn stage1_crash_recovery_checks(now: u64, checks: &mut Vec<EvaluationCheck>) -> Result<()> {
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let root = std::env::temp_dir().join(format!(
        "rocksoul-stage1-crash-eval-{now}-{nonce}-{}",
        std::process::id()
    ));
    let paths = AppPaths::from_root(root.clone());
    fs::create_dir_all(&paths.logs_dir)?;
    let initial_profile = RuhProfile::new(RuhIdentity {
        instance_id: format!("rocksoul-stage1-crash-eval-{now}"),
        created_at_unix: now.max(1),
    })?;
    rid_store::save(&paths.rid_file, &initial_profile)?;

    let proposal = WorkspaceArtifactProposal {
        filename: "crash-recovery.txt".to_owned(),
        content: "published before XP persistence".to_owned(),
    };
    let approval = UserApproval::from_typed_confirmation(&proposal, &proposal.approval_phrase())?;
    let service = WorkspaceArtifactService::new(&paths.workspace_dir);
    let published =
        service.execute_approved(&proposal, &approval, &CancellationToken::default())?;
    let interrupted =
        complete_workspace_operation_with_hook(&paths, &proposal, published, now.max(1), || {
            anyhow::bail!("deterministic evaluation crash after artifact read-back")
        });
    let artifact_path = paths.workspace_dir.join(&proposal.filename);
    let after_crash = rid_store::load(&paths.rid_file)?;
    let crash_recovery_boundary_ok = interrupted.is_err()
        && fs::read(&artifact_path).is_ok_and(|bytes| bytes == proposal.content.as_bytes())
        && after_crash.total_xp() == 0
        && after_crash.xp_events().is_empty()
        && !paths.logs_dir.join("receipts").exists();
    checks.push(check(
        "stage1.crash_after_publish_before_xp",
        crash_recovery_boundary_ok,
        "Injected process loss leaves the exact artifact but no persisted XP or verified receipt",
    ));

    let replay = service.execute_approved(&proposal, &approval, &CancellationToken::default())?;
    let replay_was_existing = matches!(replay, ArtifactOutcome::AlreadyPresent { .. });
    let first = complete_workspace_operation_with_hook(
        &paths,
        &proposal,
        replay,
        now.saturating_add(1).max(1),
        || Ok(()),
    )?;
    let first_verified = matches!(
        &first.verification,
        VerificationOutcome::Verified {
            xp_record: XpRecordOutcome::Recorded,
            ..
        }
    ) && first.profile.xp_events().len() == 1;
    let operation_id = proposal.operation_id();
    let receipt_key = operation_id.rsplit(':').next().unwrap_or("unknown");
    let receipt_path = paths
        .logs_dir
        .join("receipts")
        .join(format!("workspace-{receipt_key}.json"));
    let original_receipt = fs::read(&receipt_path)?;
    let replay_again =
        service.execute_approved(&proposal, &approval, &CancellationToken::default())?;
    let second = complete_workspace_operation_with_hook(
        &paths,
        &proposal,
        replay_again,
        now.saturating_add(2).max(1),
        || Ok(()),
    )?;
    let replay_idempotency_ok = replay_was_existing
        && first_verified
        && matches!(
            &second.verification,
            VerificationOutcome::Verified {
                xp_record: XpRecordOutcome::Duplicate,
                ..
            }
        )
        && second.profile.total_xp() == first.profile.total_xp()
        && second.profile.xp_events().len() == 1
        && fs::read(&receipt_path).is_ok_and(|bytes| bytes == original_receipt)
        && task_trace::read(&paths.logs_dir.join("task-trace.v1.jsonl"))
            .is_ok_and(|records| records.len() == 2);
    checks.push(check(
        "stage1.crash_replay_xp_receipt_idempotency",
        replay_idempotency_ok,
        "Replay re-verifies the existing artifact, records one XP event and stable receipt, then remains idempotent",
    ));

    let mut conflict_turn = TurnState::new("stage1-receipt-conflict", 0);
    conflict_turn.apply(TurnEvent::Start)?;
    conflict_turn.apply(TurnEvent::PlanAccepted)?;
    conflict_turn.apply(TurnEvent::ActionCompleted)?;
    let mut conflict_profile = RuhProfile::new(RuhIdentity {
        instance_id: format!("rocksoul-stage1-conflict-{now}"),
        created_at_unix: now.max(1),
    })?;
    let conflict_request = VerificationRequest::new(
        operation_id.clone(),
        format!("workspace-artifact:{operation_id}"),
        now.saturating_add(3).max(1),
        1,
        1,
        vec![VerificationEvidence::new(
            "artifact-readback-matches",
            proposal.content.as_str(),
            proposal.content.as_str(),
            format!("workspace-readback:conflicting-evidence:{operation_id}"),
            true,
        )],
    )
    .with_claim("same artifact claim, conflicting evidence reference");
    let conflicting_receipt_rejected = match verify_turn_and_award_xp(
        &mut conflict_turn,
        &mut conflict_profile,
        conflict_request,
    )? {
        VerificationOutcome::Verified { receipt, .. } => {
            let before = fs::read(&receipt_path)?;
            let rejected =
                persist_workspace_receipt(&paths.logs_dir, &operation_id, &receipt).is_err();
            rejected && fs::read(&receipt_path)? == before
        }
        VerificationOutcome::Rejected { .. } => false,
    };
    checks.push(check(
        "stage1.receipt_conflict_fails_closed",
        conflicting_receipt_rejected,
        "A receipt with conflicting immutable evidence cannot replace the original receipt",
    ));

    let corrupt_bytes = b"{truncated receipt".to_vec();
    fs::write(&receipt_path, &corrupt_bytes)?;
    let corrupt_receipt_rejected = match &first.verification {
        VerificationOutcome::Verified { receipt, .. } => {
            persist_workspace_receipt(&paths.logs_dir, &operation_id, receipt).is_err()
                && fs::read(&receipt_path)? == corrupt_bytes
        }
        VerificationOutcome::Rejected { .. } => false,
    };
    checks.push(check(
        "stage1.receipt_corruption_fails_closed",
        corrupt_receipt_rejected,
        "Malformed durable receipt JSON is rejected and preserved for diagnosis",
    ));

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MemoryRetrievalCaseReport {
    pub case_id: String,
    pub expected_ids: Vec<String>,
    pub retrieved_ids: Vec<String>,
    pub precision_at_k: f64,
    pub recall_at_k: f64,
    pub stale_results_at_k: usize,
    pub expired_leakage: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MemoryRetrievalEvaluationReport {
    pub schema_version: u16,
    pub suite_id: String,
    pub suite_version: String,
    #[serde(default)]
    pub rocksoul_commit: String,
    pub evaluated_at_unix: u64,
    pub top_k: usize,
    pub precision_at_k: f64,
    pub recall_at_k: f64,
    pub stale_results_at_k: usize,
    pub expired_leakage: usize,
    pub deletion_leakage: usize,
    pub passed: bool,
    pub cases: Vec<MemoryRetrievalCaseReport>,
}

#[derive(Deserialize)]
struct MemoryRetrievalFixture {
    schema_version: u16,
    suite_id: String,
    suite_version: String,
    top_k: usize,
    cases: Vec<MemoryRetrievalFixtureCase>,
}

#[derive(Deserialize)]
struct MemoryRetrievalFixtureCase {
    id: String,
    query: String,
    now_unix: u64,
    expected_ids: Vec<String>,
    stale_ids: Vec<String>,
    records: Vec<MemoryRecord>,
}

pub fn run_memory_retrieval() -> Result<MemoryRetrievalEvaluationReport> {
    let report = evaluate_memory_retrieval_fixture()?;
    persist_memory_retrieval_report(&report)?;
    Ok(report)
}

fn evaluate_memory_retrieval_fixture() -> Result<MemoryRetrievalEvaluationReport> {
    let fixture: MemoryRetrievalFixture =
        serde_json::from_str(include_str!("../data/evaluation/memory-retrieval-v1.json"))
            .context("invalid embedded memory retrieval fixture")?;
    anyhow::ensure!(
        fixture.schema_version == 1,
        "unsupported memory retrieval fixture schema"
    );
    anyhow::ensure!(
        fixture.top_k > 0,
        "memory retrieval fixture top_k must be positive"
    );

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before the Unix epoch")?
        .as_secs();
    let mut case_reports = Vec::new();
    for case in &fixture.cases {
        let store = MemoryStore {
            schema_version: MEMORY_SCHEMA_VERSION,
            revision: 0,
            records: case.records.clone(),
        };
        store.validate()?;
        let retrieved = store.search_ranked(
            &case.query,
            &MemorySearchFilters::default(),
            case.now_unix,
            fixture.top_k,
        );
        let retrieved_ids = retrieved
            .iter()
            .map(|item| item.record_id.clone())
            .collect::<Vec<_>>();
        let expected = case
            .expected_ids
            .iter()
            .cloned()
            .collect::<std::collections::HashSet<_>>();
        let stale = case
            .stale_ids
            .iter()
            .cloned()
            .collect::<std::collections::HashSet<_>>();
        let expired = case
            .records
            .iter()
            .filter(|record| {
                record
                    .expires_at_unix
                    .is_some_and(|expires| expires <= case.now_unix)
            })
            .map(|record| record.id.clone())
            .collect::<std::collections::HashSet<_>>();
        let hits = retrieved_ids
            .iter()
            .filter(|id| expected.contains(*id))
            .count();
        let stale_results_at_k = retrieved_ids
            .iter()
            .filter(|id| stale.contains(*id))
            .count();
        let expired_leakage = retrieved_ids
            .iter()
            .filter(|id| expired.contains(*id))
            .count();
        let precision_at_k = hits as f64 / fixture.top_k as f64;
        let recall_at_k = hits as f64 / case.expected_ids.len().max(1) as f64;
        case_reports.push(MemoryRetrievalCaseReport {
            case_id: case.id.clone(),
            expected_ids: case.expected_ids.clone(),
            retrieved_ids,
            precision_at_k,
            recall_at_k,
            stale_results_at_k,
            expired_leakage,
        });
    }

    let case_count = case_reports.len().max(1) as f64;
    let precision_at_k = case_reports
        .iter()
        .map(|item| item.precision_at_k)
        .sum::<f64>()
        / case_count;
    let recall_at_k = case_reports
        .iter()
        .map(|item| item.recall_at_k)
        .sum::<f64>()
        / case_count;
    let stale_results_at_k = case_reports
        .iter()
        .map(|item| item.stale_results_at_k)
        .sum();
    let expired_leakage = case_reports.iter().map(|item| item.expired_leakage).sum();

    let deletion_leakage = memory_forgetting_leakage_check(now)?;
    let passed = precision_at_k >= 0.66
        && recall_at_k >= 0.95
        && stale_results_at_k == 0
        && expired_leakage == 0
        && deletion_leakage == 0;
    Ok(MemoryRetrievalEvaluationReport {
        schema_version: 1,
        suite_id: fixture.suite_id,
        suite_version: fixture.suite_version,
        rocksoul_commit: env!("ROCKSOUL_GIT_SHA").to_owned(),
        evaluated_at_unix: now,
        top_k: fixture.top_k,
        precision_at_k,
        recall_at_k,
        stale_results_at_k,
        expired_leakage,
        deletion_leakage,
        passed,
        cases: case_reports,
    })
}

fn memory_forgetting_leakage_check(now: u64) -> Result<usize> {
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "rocksoul-memory-retrieval-eval-{now}-{nonce}-{}",
        std::process::id()
    ));
    let id = MemoryStore::transact(&directory, |store| {
        store.add_user_entry(
            MemoryKind::Semantic,
            "synthetic deletion leakage sentinel".to_owned(),
            vec!["retrieval-evaluation".to_owned()],
            None,
            now.max(1),
        )
    })?;
    MemoryStore::transact(&directory, |store| {
        anyhow::ensure!(
            store.delete(&id),
            "evaluation sentinel could not be deleted"
        );
        Ok(())
    })?;
    let loaded = MemoryStore::load(&directory)?;
    let leaked = loaded
        .search_ranked(
            "synthetic deletion leakage sentinel",
            &MemorySearchFilters::default(),
            now.saturating_add(1),
            usize::MAX,
        )
        .iter()
        .any(|result| result.record_id == id);
    let _ = fs::remove_dir_all(&directory);
    Ok(if leaked { 1 } else { 0 })
}

fn persist_memory_retrieval_report(report: &MemoryRetrievalEvaluationReport) -> Result<()> {
    let path = AppPaths::discover()?
        .data_dir
        .join("evaluations")
        .join("memory-retrieval")
        .join("latest.json");
    let parent = path
        .parent()
        .context("memory retrieval report path has no parent")?;
    fs::create_dir_all(parent)?;
    let temporary = path.with_extension("json.tmp");
    let backup = path.with_extension("json.bak");
    fs::write(&temporary, serde_json::to_vec_pretty(report)?)?;
    let moved_previous = if path.exists() {
        if backup.exists() {
            fs::remove_file(&backup)?;
        }
        fs::rename(&path, &backup)?;
        true
    } else {
        false
    };
    if let Err(error) = fs::rename(&temporary, &path) {
        if moved_previous {
            let _ = fs::rename(&backup, &path);
        }
        let _ = fs::remove_file(&temporary);
        return Err(error).with_context(|| format!("could not replace {}", path.display()));
    }
    if moved_previous {
        fs::remove_file(&backup)?;
    }
    persist_markdown_report(&path.with_extension("md"), &render_memory_markdown(report))?;
    Ok(())
}

fn parse_report<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    serde_json::from_slice(bytes).context("invalid evaluation report data")
}

pub fn latest_memory_retrieval() -> Result<Option<MemoryRetrievalEvaluationReport>> {
    let path = AppPaths::discover()?
        .data_dir
        .join("evaluations")
        .join("memory-retrieval")
        .join("latest.json");
    if !path.is_file() {
        return Ok(None);
    }
    let bytes = fs::read(&path).with_context(|| format!("could not read {}", path.display()))?;
    Ok(Some(parse_report(&bytes)?))
}

pub fn print_memory_retrieval_report(report: &MemoryRetrievalEvaluationReport) {
    println!(
        "Memory retrieval evaluation — {} v{}",
        report.suite_id, report.suite_version
    );
    println!(
        "Result: {} · precision@{} {:.3} · recall@{} {:.3}",
        if report.passed { "PASS" } else { "FAIL" },
        report.top_k,
        report.precision_at_k,
        report.top_k,
        report.recall_at_k
    );
    println!("Source commit: {}", report.rocksoul_commit);
    println!(
        "Stale results: {} · expired leakage: {} · deletion leakage: {}",
        report.stale_results_at_k, report.expired_leakage, report.deletion_leakage
    );
    for case in &report.cases {
        println!(
            "  [{}] precision {:.3} · recall {:.3} · retrieved {:?}",
            case.case_id, case.precision_at_k, case.recall_at_k, case.retrieved_ids
        );
    }
}

pub fn latest() -> Result<Option<EvaluationReport>> {
    latest_for(CORE_SUITE_ID)
}

/// Read the most recent report for an explicit suite. Each suite owns its own
/// path so one run cannot replace another suite's evidence.
pub fn latest_for(suite_id: &str) -> Result<Option<EvaluationReport>> {
    let path = latest_report_path_for(suite_id)?;
    if !path.is_file() {
        return Ok(None);
    }
    let bytes = fs::read(&path).with_context(|| format!("could not read {}", path.display()))?;
    let report = parse_report(&bytes)?;
    Ok(Some(report))
}

fn report_write_suffix() -> String {
    static NONCE: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!(
        "{}-{}-{}",
        std::process::id(),
        nanos,
        NONCE.fetch_add(1, Ordering::Relaxed)
    )
}

/// Unique scratch paths prevent concurrent suite runs from clobbering each
/// other's temporary or backup files.
fn report_scratch_path(path: &std::path::Path, label: &str) -> PathBuf {
    let name = path
        .file_name()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| "report".to_owned());
    path.with_file_name(format!("{name}.{label}.{}", report_write_suffix()))
}

fn persist_report(report: &EvaluationReport) -> Result<()> {
    let path = latest_report_path_for(&report.suite_id)?;
    let parent = path
        .parent()
        .context("evaluation report path has no parent")?;
    fs::create_dir_all(parent).with_context(|| format!("could not create {}", parent.display()))?;
    let bytes = serde_json::to_vec_pretty(report)?;
    let temporary = report_scratch_path(&path, "tmp");
    fs::write(&temporary, bytes)
        .with_context(|| format!("could not write {}", temporary.display()))?;

    // Windows rename does not replace an existing destination. Preserve the
    // previous report under a unique backup until the new file is in place.
    let backup = report_scratch_path(&path, "bak");
    let moved_previous = if path.exists() {
        fs::rename(&path, &backup)
            .with_context(|| format!("could not back up {}", path.display()))?;
        true
    } else {
        false
    };

    if let Err(error) = fs::rename(&temporary, &path) {
        if moved_previous {
            let _ = fs::rename(&backup, &path);
        }
        let _ = fs::remove_file(&temporary);
        return Err(error).with_context(|| format!("could not replace {}", path.display()));
    }
    if moved_previous {
        fs::remove_file(&backup)
            .with_context(|| format!("could not remove {}", backup.display()))?;
    }
    persist_markdown_report(&path.with_extension("md"), &render_core_markdown(report))?;
    Ok(())
}

/// Each suite owns its own report file so concurrent runs cannot replace or
/// corrupt another suite's evidence.
fn latest_report_path_for(suite_id: &str) -> Result<PathBuf> {
    let leaf = if suite_id == CORE_SUITE_ID {
        "core".to_owned()
    } else {
        suite_id.to_owned()
    };
    Ok(AppPaths::discover()?
        .data_dir
        .join("evaluations")
        .join(leaf)
        .join("latest.json"))
}

fn persist_markdown_report(path: &std::path::Path, content: &str) -> Result<()> {
    let parent = path
        .parent()
        .context("Markdown report path has no parent")?;
    fs::create_dir_all(parent)?;
    let temporary = report_scratch_path(path, "md.tmp");
    let backup = report_scratch_path(path, "md.bak");
    fs::write(&temporary, content.as_bytes())?;
    let moved_previous = if path.exists() {
        fs::rename(path, &backup)?;
        true
    } else {
        false
    };
    if let Err(error) = fs::rename(&temporary, path) {
        if moved_previous {
            let _ = fs::rename(&backup, path);
        }
        let _ = fs::remove_file(&temporary);
        return Err(error).with_context(|| format!("could not replace {}", path.display()));
    }
    if moved_previous {
        fs::remove_file(&backup)?;
    }
    Ok(())
}

fn markdown_cell(value: &str) -> String {
    value.replace('|', "\\|").replace(['\r', '\n'], " ")
}

fn render_core_markdown(report: &EvaluationReport) -> String {
    let mut output = format!(
        "# Rocksoul Evaluation — {} v{}\n\n- Result: **{}** ({}/{} passed, {} failed)\n- Version: {}\n- Source commit: {}\n- Platform: {} / {}\n- Evaluated at (Unix): {}\n\n| Check | Result | Detail |\n|---|---|---|\n",
        markdown_cell(&report.suite_id),
        markdown_cell(&report.suite_version),
        if report.passed { "PASS" } else { "FAIL" },
        report.passed_count(),
        report.checks.len(),
        report.failed_count(),
        markdown_cell(&report.rocksoul_version),
        markdown_cell(&report.rocksoul_commit),
        markdown_cell(&report.platform),
        markdown_cell(&report.architecture),
        report.evaluated_at_unix
    );
    for item in &report.checks {
        output.push_str(&format!(
            "| {} | {} | {} |\n",
            markdown_cell(&item.id),
            if item.passed { "PASS" } else { "FAIL" },
            markdown_cell(&item.detail)
        ));
    }
    output
}

fn render_memory_markdown(report: &MemoryRetrievalEvaluationReport) -> String {
    let mut output = format!(
        "# Memory Retrieval Evaluation — {} v{}\n\n- Result: **{}**\n- Source commit: {}\n- Evaluated at (Unix): {}\n- Precision@{}: {:.3}\n- Recall@{}: {:.3}\n- Stale results: {}\n- Expired leakage: {}\n- Deletion leakage: {}\n\n| Case | Precision@k | Recall@k | Retrieved IDs |\n|---|---:|---:|---|\n",
        markdown_cell(&report.suite_id),
        markdown_cell(&report.suite_version),
        if report.passed { "PASS" } else { "FAIL" },
        markdown_cell(&report.rocksoul_commit),
        report.evaluated_at_unix,
        report.top_k,
        report.precision_at_k,
        report.top_k,
        report.recall_at_k,
        report.stale_results_at_k,
        report.expired_leakage,
        report.deletion_leakage
    );
    for case in &report.cases {
        output.push_str(&format!(
            "| {} | {:.3} | {:.3} | {} |\n",
            markdown_cell(&case.case_id),
            case.precision_at_k,
            case.recall_at_k,
            markdown_cell(&format!("{:?}", case.retrieved_ids))
        ));
    }
    output
}

fn check(id: &str, passed: bool, detail: &str) -> EvaluationCheck {
    EvaluationCheck {
        id: id.to_owned(),
        passed,
        detail: detail.to_owned(),
    }
}

pub fn print_report(report: &EvaluationReport) {
    println!(
        "Rocksoul evaluation — {} v{}",
        report.suite_id, report.suite_version
    );
    println!(
        "Runtime: {} / {} · Rocksoul {}",
        report.platform, report.architecture, report.rocksoul_version
    );
    println!("Source commit: {}", report.rocksoul_commit);
    println!(
        "Result: {} ({}/{} passed, {} failed)",
        if report.passed { "PASS" } else { "FAIL" },
        report.passed_count(),
        report.checks.len(),
        report.failed_count()
    );
    for item in &report.checks {
        println!(
            "  [{}] {} — {}",
            if item.passed { "PASS" } else { "FAIL" },
            item.id,
            item.detail
        );
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{
        AUTHORIZATION_SUITE_ID, AUTHORIZATION_SUITE_VERSION, CORE_SUITE_ID, CORE_SUITE_VERSION,
        EvaluationReport, WORKER_SUPERVISOR_SUITE_ID, WORKER_SUPERVISOR_SUITE_VERSION,
    };

    fn sample_agent_trajectory() -> super::AgentTrajectoryReport {
        super::AgentTrajectoryReport {
            schema_version: 1,
            metadata: super::AgentEvaluationMetadata {
                schema_version: 1,
                run_id: "run-001".to_owned(),
                rocksoul_commit: "unknown".to_owned(),
                suite_id: "rocksoul.agent-trajectory".to_owned(),
                suite_version: "0.1.0".to_owned(),
                model_provider: "local-openai-compatible".to_owned(),
                model_id: "test-model".to_owned(),
                quantization: None,
                platform: "test".to_owned(),
                seed: 7,
                dataset_id: "held-out-smoke".to_owned(),
                dataset_version: "1".to_owned(),
                task_id: "task-001".to_owned(),
                task_digest: "sha256:task".to_owned(),
                started_at_unix_ms: 1,
                duration_ms: 20,
                input_tokens: Some(10),
                output_tokens: Some(5),
                estimated_cost_microunits: None,
            },
            steps: vec![super::AgentTrajectoryStep {
                sequence: 0,
                step_id: "step-001".to_owned(),
                phase: "planning".to_owned(),
                action: "inspect_task".to_owned(),
                tool: None,
                input_digest: None,
                observation_ref: Some("observation:001".to_owned()),
                outcome: "observed".to_owned(),
                retry_index: 0,
                duration_ms: 20,
            }],
            task_success: true,
            verifier: Some(super::IndependentVerifierEvidence {
                verifier_id: "deterministic-test-verifier".to_owned(),
                rubric_version: "1".to_owned(),
                independent: true,
                passed: true,
                confidence_basis_points: 10_000,
                evidence_refs: vec!["evidence:001".to_owned()],
            }),
            safety_violations: Vec::new(),
            trace_refs: vec!["trace:001".to_owned()],
        }
    }

    #[test]
    fn agent_trajectory_requires_independent_verifier_evidence() {
        let mut report = sample_agent_trajectory();
        assert!(report.validate().is_ok());

        report.verifier.as_mut().unwrap().independent = false;
        assert!(report.validate().is_err());

        report.verifier = None;
        assert!(report.validate().is_err());
    }

    #[test]
    fn agent_trajectory_safety_violations_are_hard_vetoes() {
        let mut report = sample_agent_trajectory();
        report
            .safety_violations
            .push("permission_violation".to_owned());
        assert!(report.validate().is_err());

        report.task_success = false;
        assert!(report.validate().is_ok());
    }

    #[test]
    fn agent_trajectory_rejects_duplicate_steps_and_invalid_confidence() {
        let mut report = sample_agent_trajectory();
        let mut duplicate = report.steps[0].clone();
        duplicate.sequence = 1;
        report.steps.push(duplicate);
        assert!(report.validate().is_err());

        report.steps.pop();
        report.verifier.as_mut().unwrap().confidence_basis_points = 10_001;
        assert!(report.validate().is_err());
    }

    #[test]
    fn markdown_cell_escapes_table_delimiters_and_collapses_line_breaks() {
        assert_eq!(
            super::markdown_cell("left|right\r\nnext"),
            "left\\|right  next"
        );
    }

    #[test]
    fn markdown_report_replaces_existing_file_without_leaving_temporary_files() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!("rocksoul-eval-md-{nonce}"));
        fs::create_dir_all(&directory).unwrap();
        let path = directory.join("latest.md");

        super::persist_markdown_report(&path, "old report").unwrap();
        super::persist_markdown_report(&path, "new report").unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), "new report");
        assert!(!directory.join("latest.md.tmp").exists());
        assert!(!directory.join("latest.md.bak").exists());
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn older_reports_without_metrics_remain_parseable() {
        let raw = br#"{"schema_version":1,"suite_id":"legacy","suite_version":"1","rocksoul_version":"0.1.0","rocksoul_commit":"unknown","platform":"test","architecture":"test","evaluated_at_unix":1,"passed":true,"checks":[]}"#;
        let report = super::parse_report::<super::EvaluationReport>(raw).unwrap();
        assert!(report.metrics.is_none());
    }

    #[test]
    fn authorization_report_has_structured_metrics_and_content_free_markdown() {
        let report = super::run_authorization_contract().unwrap();
        let metrics = report.metrics.as_ref().expect("authorization metrics");
        assert_eq!(metrics.get("cases").map(String::as_str), Some("38"));
        assert_eq!(metrics.get("blocked").map(String::as_str), Some("38"));
        assert_eq!(
            metrics.get("token_overhead").map(String::as_str),
            Some("not_measured")
        );
        let markdown = super::render_core_markdown(&report);
        assert!(!markdown.contains("attacker@example.com"));
        assert!(!markdown.contains("team@example.com"));
    }

    #[test]
    fn json_suite_registry_matches_manifest_and_constants() {
        let value: serde_json::Value = serde_json::from_str(&super::list_json().unwrap()).unwrap();
        let suites = value.as_array().unwrap();
        assert_eq!(suites.len(), 6);
        assert_eq!(suites[3]["id"], AUTHORIZATION_SUITE_ID);
        assert_eq!(suites[3]["version"], AUTHORIZATION_SUITE_VERSION);
        assert_eq!(suites[4]["id"], WORKER_SUPERVISOR_SUITE_ID);
        assert_eq!(suites[4]["version"], WORKER_SUPERVISOR_SUITE_VERSION);
        assert_eq!(suites[5]["status"], "planned");
    }

    #[test]
    fn suites_persist_to_independent_report_paths() {
        let core = super::latest_report_path_for(CORE_SUITE_ID).unwrap();
        let authorization = super::latest_report_path_for(AUTHORIZATION_SUITE_ID).unwrap();
        let worker = super::latest_report_path_for(WORKER_SUPERVISOR_SUITE_ID).unwrap();
        assert_ne!(core, authorization);
        assert_ne!(core, worker);
        assert_ne!(authorization, worker);
    }

    #[test]
    fn authorization_contract_suite_passes() {
        let report = super::run_authorization_contract().unwrap();
        assert!(report.passed);
        assert_eq!(report.suite_id, AUTHORIZATION_SUITE_ID);
        assert_eq!(report.checks.len(), 11);
    }

    #[test]
    fn worker_supervisor_contract_suite_passes() {
        let report = super::run_worker_supervisor_contract().unwrap();
        assert!(report.passed);
        assert_eq!(report.suite_id, WORKER_SUPERVISOR_SUITE_ID);
        assert_eq!(report.checks.len(), 7);
    }

    #[test]
    fn benchmark_manifest_matches_implemented_suites_and_rid_contract() {
        let manifest: serde_json::Value = serde_json::from_str(include_str!(
            "../data/evaluation/benchmark-manifest.v1.json"
        ))
        .unwrap();
        assert_eq!(manifest["schema_version"], 1);
        assert_eq!(manifest["manifest_version"], "1.1.5");
        assert_eq!(manifest["rid_taxonomy"]["domain_count"], 99);
        assert_eq!(manifest["rid_taxonomy"]["stat_count"], 99);
        assert_eq!(manifest["rid_taxonomy"]["overlap_pair_count"], 161);
        assert_eq!(manifest["rid_taxonomy"]["overlap_directed_edge_count"], 322);

        let suites = manifest["suites"].as_array().unwrap();
        let mut ids = std::collections::HashSet::new();
        for suite in suites {
            let id = suite["id"].as_str().unwrap();
            assert!(ids.insert(id), "duplicate benchmark suite ID: {id}");
        }

        for (id, version) in [
            (CORE_SUITE_ID, CORE_SUITE_VERSION),
            ("rocksoul.memory-retrieval", "1.0.0"),
            ("rocksoul.agent-runtime-fixtures", "0.1.0"),
            (AUTHORIZATION_SUITE_ID, AUTHORIZATION_SUITE_VERSION),
            (WORKER_SUPERVISOR_SUITE_ID, WORKER_SUPERVISOR_SUITE_VERSION),
        ] {
            let suite = suites
                .iter()
                .find(|suite| suite["id"] == id)
                .unwrap_or_else(|| panic!("missing implemented suite: {id}"));
            assert_eq!(suite["version"], version);
            assert_eq!(suite["status"], "implemented");
            assert_eq!(suite["execution"], "offline_deterministic_ci");
        }

        let trajectory = suites
            .iter()
            .find(|suite| suite["id"] == "rocksoul.agent-trajectory")
            .unwrap();
        assert_eq!(trajectory["status"], "planned");
    }

    #[test]
    fn report_parser_rejects_invalid_json_without_echoing_payload_or_path() {
        let payload = b"api_key=do-not-echo";
        let error = super::parse_report::<EvaluationReport>(payload).unwrap_err();
        let message = format!("{error:#}");
        assert!(message.contains("invalid evaluation report data"));
        assert!(!message.contains("do-not-echo"));
        assert!(!message.contains("api_key"));
    }

    #[test]
    fn offline_memory_retrieval_fixture_meets_quality_and_privacy_gates() {
        let report = super::evaluate_memory_retrieval_fixture().unwrap();
        assert!(report.passed, "memory retrieval report failed: {report:?}");
        assert!(report.precision_at_k >= 0.66);
        assert_eq!(report.rocksoul_commit, env!("ROCKSOUL_GIT_SHA"));
        assert!(report.recall_at_k >= 0.95);
        assert_eq!(report.stale_results_at_k, 0);
        assert_eq!(report.expired_leakage, 0);
        assert_eq!(report.deletion_leakage, 0);
        let markdown = super::render_memory_markdown(&report);
        assert!(markdown.contains("Source commit:"));
        assert!(markdown.contains("| Case | Precision@k | Recall@k | Retrieved IDs |"));
    }

    #[test]
    fn evaluation_report_counts_failed_checks_without_averaging_them_away() {
        let report = EvaluationReport {
            schema_version: 1,
            suite_id: "test".into(),
            suite_version: "1".into(),
            rocksoul_version: "0.1.0".into(),
            rocksoul_commit: "test-revision".into(),
            platform: "test".into(),
            architecture: "test".into(),
            evaluated_at_unix: 1,
            passed: false,
            metrics: None,
            checks: vec![
                super::check("safe", true, "ok"),
                super::check("unsafe", false, "veto"),
            ],
        };
        assert_eq!(report.passed_count(), 1);
        assert_eq!(report.failed_count(), 1);
        assert!(!report.passed);
        let encoded = serde_json::to_vec(&report).unwrap();
        let decoded: EvaluationReport = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(decoded.failed_count(), 1);
        assert_eq!(decoded.rocksoul_commit, "test-revision");
        let markdown = super::render_core_markdown(&decoded);
        assert!(markdown.contains("test-revision"));
        assert!(markdown.contains("| unsafe | FAIL | veto |"));
    }
}
