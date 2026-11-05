use std::{
    collections::HashSet,
    fmt,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::rid::{RuhProfile, XpEvent, XpRecordOutcome, XpSource};

/// Deterministic lifecycle phases for one cognitive turn.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TurnPhase {
    Idle,
    Planning,
    Executing,
    Verifying,
    Completed,
    Failed,
    Cancelled,
}

impl TurnPhase {
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }
}

/// Events accepted by the cognitive turn state machine.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TurnEvent {
    Start,
    PlanAccepted,
    ActionCompleted,
    VerificationPassed,
    RetryRequested,
    VerificationRejected,
    Fail,
    Cancel,
}

/// Serializable state for one turn.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TurnState {
    pub id: String,
    pub phase: TurnPhase,
    pub retry_count: u32,
    pub max_retries: u32,
}

impl TurnState {
    pub fn new(id: impl Into<String>, max_retries: u32) -> Self {
        Self {
            id: id.into(),
            phase: TurnPhase::Idle,
            retry_count: 0,
            max_retries,
        }
    }

    /// Apply one explicit event. Invalid transitions leave the state unchanged.
    pub fn apply(&mut self, event: TurnEvent) -> Result<(), TransitionError> {
        if self.phase.is_terminal() {
            return Err(TransitionError::TerminalState(self.phase));
        }

        if matches!(event, TurnEvent::Fail) {
            self.phase = TurnPhase::Failed;
            return Ok(());
        }

        if matches!(event, TurnEvent::Cancel) {
            self.phase = TurnPhase::Cancelled;
            return Ok(());
        }

        let next = match (self.phase, event) {
            (TurnPhase::Idle, TurnEvent::Start) => TurnPhase::Planning,
            (TurnPhase::Planning, TurnEvent::PlanAccepted) => TurnPhase::Executing,
            (TurnPhase::Executing, TurnEvent::ActionCompleted) => TurnPhase::Verifying,
            (TurnPhase::Verifying, TurnEvent::VerificationPassed) => TurnPhase::Completed,
            (TurnPhase::Verifying, TurnEvent::RetryRequested) => {
                if self.retry_count >= self.max_retries {
                    return Err(TransitionError::RetryBudgetExhausted {
                        max_retries: self.max_retries,
                    });
                }
                self.retry_count += 1;
                TurnPhase::Planning
            }
            (TurnPhase::Verifying, TurnEvent::VerificationRejected) => TurnPhase::Failed,
            (phase, event) => return Err(TransitionError::InvalidTransition { phase, event }),
        };

        self.phase = next;
        Ok(())
    }
}

/// Evidence supplied by a trusted executor or evaluator adapter.
///
/// Fields are deliberately private. Model output is not an evidence object;
/// an application adapter must create this value from an observed result.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct VerificationEvidence {
    check_id: String,
    expected: String,
    observed: String,
    evidence_ref: String,
    required: bool,
}

impl VerificationEvidence {
    /// Construct evidence from an independently observed result. Never pass raw model claims here.
    pub fn new(
        check_id: impl Into<String>,
        expected: impl Into<String>,
        observed: impl Into<String>,
        evidence_ref: impl Into<String>,
        required: bool,
    ) -> Self {
        Self {
            check_id: check_id.into(),
            expected: expected.into(),
            observed: observed.into(),
            evidence_ref: evidence_ref.into(),
            required,
        }
    }
}

/// Request to verify a completed turn. Only crate-owned executor adapters
/// should construct it from observed, attributable evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerificationRequest {
    task_id: String,
    verification_id: String,
    verified_at_unix: u64,
    difficulty: u8,
    policy_version: u16,
    checks: Vec<VerificationEvidence>,
    normalized_claim: Option<String>,
}

impl VerificationRequest {
    /// Build a verification request inside a trusted application adapter.
    /// The request must be based on observed tool/runtime results, not model self-report.
    pub fn new(
        task_id: impl Into<String>,
        verification_id: impl Into<String>,
        verified_at_unix: u64,
        difficulty: u8,
        policy_version: u16,
        checks: Vec<VerificationEvidence>,
    ) -> Self {
        Self {
            task_id: task_id.into(),
            verification_id: verification_id.into(),
            verified_at_unix,
            difficulty,
            policy_version,
            checks,
            normalized_claim: None,
        }
    }

    /// Bind a trusted verification to the exact normalized task-outcome claim.
    pub fn with_claim(mut self, claim: impl AsRef<str>) -> Self {
        self.normalized_claim = Some(normalize_task_claim(claim.as_ref()));
        self
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct VerificationReceipt {
    verification_id: String,
    task_id: String,
    required_checks_passed: usize,
    total_checks: usize,
    evidence_refs: Vec<String>,
    xp_awarded: u32,
    verified_at_unix: u64,
    verifier_id: String,
    verifier_version: u16,
    normalized_claim: Option<String>,
    claim_hash: Option<String>,
    evidence_hash: String,
    receipt_hash: String,
    outcome: String,
}

impl VerificationReceipt {
    pub fn verification_id(&self) -> &str {
        &self.verification_id
    }

    pub fn task_id(&self) -> &str {
        &self.task_id
    }

    pub fn required_checks_passed(&self) -> usize {
        self.required_checks_passed
    }

    pub fn total_checks(&self) -> usize {
        self.total_checks
    }

    pub fn evidence_refs(&self) -> &[String] {
        &self.evidence_refs
    }

    pub fn xp_awarded(&self) -> u32 {
        self.xp_awarded
    }

    pub fn verified_at_unix(&self) -> u64 {
        self.verified_at_unix
    }

    pub fn verifier_id(&self) -> &str {
        &self.verifier_id
    }
    pub fn verifier_version(&self) -> u16 {
        self.verifier_version
    }
    pub fn normalized_claim(&self) -> Option<&str> {
        self.normalized_claim.as_deref()
    }
    pub fn claim_hash(&self) -> Option<&str> {
        self.claim_hash.as_deref()
    }
    pub fn evidence_hash(&self) -> &str {
        &self.evidence_hash
    }
    pub fn receipt_hash(&self) -> &str {
        &self.receipt_hash
    }
    pub fn outcome(&self) -> &str {
        &self.outcome
    }

    /// Detect accidental or partial mutation; this digest is not a digital signature.
    pub fn integrity_valid(&self) -> bool {
        let payload = serde_json::to_vec(&(
            &self.verification_id,
            &self.task_id,
            self.required_checks_passed,
            self.total_checks,
            &self.evidence_refs,
            self.xp_awarded,
            self.verified_at_unix,
            &self.verifier_id,
            self.verifier_version,
            &self.normalized_claim,
            &self.claim_hash,
            &self.evidence_hash,
            &self.outcome,
        ));
        payload.is_ok_and(|bytes| format!("{:x}", Sha256::digest(bytes)) == self.receipt_hash)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VerificationOutcome {
    Verified {
        receipt: Box<VerificationReceipt>,
        xp_record: XpRecordOutcome,
    },
    Rejected {
        failed_required_checks: Vec<String>,
    },
}

/// Validate an active turn and award XP only when every required acceptance
/// check matches independently observed evidence. The verification ID makes
/// replay idempotent; reusing it with different evidence is rejected.
/// Verify a turn using trusted, independently observed evidence and award XP
/// only after all required checks pass. Do not expose this as a model-callable tool.
pub fn verify_turn_and_award_xp(
    turn: &mut TurnState,
    profile: &mut RuhProfile,
    request: VerificationRequest,
) -> Result<VerificationOutcome, VerificationError> {
    if turn.phase != TurnPhase::Verifying {
        return Err(VerificationError::NotVerifying(turn.phase));
    }
    if request.task_id.trim().is_empty()
        || request.verification_id.trim().is_empty()
        || request.verified_at_unix == 0
        || request.policy_version == 0
        || !(1..=10).contains(&request.difficulty)
        || request.checks.is_empty()
    {
        return Err(VerificationError::InvalidRequest);
    }

    let mut check_ids = std::collections::HashSet::with_capacity(request.checks.len());
    let mut failed_required_checks = Vec::new();
    let mut passed_required = 0_usize;
    let mut evidence_refs = Vec::with_capacity(request.checks.len());

    for check in &request.checks {
        if check.check_id.trim().is_empty()
            || check.evidence_ref.trim().is_empty()
            || !check_ids.insert(check.check_id.as_str())
        {
            return Err(VerificationError::InvalidEvidence);
        }

        evidence_refs.push(check.evidence_ref.clone());
        if check.expected == check.observed {
            if check.required {
                passed_required += 1;
            }
        } else if check.required {
            failed_required_checks.push(check.check_id.clone());
        }
    }

    if passed_required == 0 || !failed_required_checks.is_empty() {
        turn.apply(TurnEvent::VerificationRejected)
            .map_err(VerificationError::Transition)?;
        return Ok(VerificationOutcome::Rejected {
            failed_required_checks,
        });
    }

    let xp_awarded =
        (5_u32 + u32::from(request.difficulty) + (passed_required as u32 * 2)).min(100);
    let evidence_bytes = serde_json::to_vec(&request.checks)
        .expect("verification evidence serialization cannot fail");
    let evidence_hash = format!("{:x}", Sha256::digest(evidence_bytes));
    let claim_hash = request
        .normalized_claim
        .as_ref()
        .map(|claim| format!("{:x}", Sha256::digest(claim.as_bytes())));
    let event = XpEvent {
        event_id: format!("verification:{}", request.verification_id),
        amount: xp_awarded,
        source: XpSource::VerifiedTask,
        occurred_at_unix: request.verified_at_unix,
        evidence_ref: format!("verification:{}", request.verification_id),
        policy_version: request.policy_version,
    };

    let xp_record = profile
        .record_verified_xp(event)
        .map_err(|error| VerificationError::Rid(error.to_string()))?;
    turn.apply(TurnEvent::VerificationPassed)
        .map_err(VerificationError::Transition)?;

    let verifier_id = "rocksoul.core.verifier".to_owned();
    let verifier_version = 1_u16;
    let outcome = "verified".to_owned();
    let receipt_hash_payload = serde_json::to_vec(&(
        &request.verification_id,
        &request.task_id,
        passed_required,
        request.checks.len(),
        &evidence_refs,
        xp_awarded,
        request.verified_at_unix,
        &verifier_id,
        verifier_version,
        &request.normalized_claim,
        &claim_hash,
        &evidence_hash,
        &outcome,
    ))
    .expect("verification receipt serialization cannot fail");
    let receipt_hash = format!("{:x}", Sha256::digest(receipt_hash_payload));
    Ok(VerificationOutcome::Verified {
        receipt: Box::new(VerificationReceipt {
            verification_id: request.verification_id,
            task_id: request.task_id,
            required_checks_passed: passed_required,
            total_checks: request.checks.len(),
            evidence_refs,
            xp_awarded,
            verified_at_unix: request.verified_at_unix,
            verifier_id,
            verifier_version,
            normalized_claim: request.normalized_claim,
            claim_hash,
            evidence_hash,
            receipt_hash,
            outcome,
        }),
        xp_record,
    })
}

pub fn normalize_task_claim(claim: &str) -> String {
    claim
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VerificationError {
    NotVerifying(TurnPhase),
    InvalidRequest,
    InvalidEvidence,
    Transition(TransitionError),
    Rid(String),
}

impl fmt::Display for VerificationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotVerifying(phase) => {
                write!(formatter, "XP cannot be awarded while turn is {phase:?}")
            }
            Self::InvalidRequest => write!(formatter, "verification request is invalid"),
            Self::InvalidEvidence => {
                write!(
                    formatter,
                    "verification evidence is incomplete or duplicated"
                )
            }
            Self::Transition(error) => write!(formatter, "{error}"),
            Self::Rid(error) => write!(formatter, "RID rejected verified XP event: {error}"),
        }
    }
}

impl std::error::Error for VerificationError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransitionError {
    InvalidTransition { phase: TurnPhase, event: TurnEvent },
    RetryBudgetExhausted { max_retries: u32 },
    TerminalState(TurnPhase),
}

impl fmt::Display for TransitionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTransition { phase, event } => {
                write!(
                    formatter,
                    "event {event:?} is invalid while turn is {phase:?}"
                )
            }
            Self::RetryBudgetExhausted { max_retries } => {
                write!(
                    formatter,
                    "turn retry budget exhausted (maximum {max_retries})"
                )
            }
            Self::TerminalState(phase) => {
                write!(formatter, "terminal turn state {phase:?} cannot transition")
            }
        }
    }
}

impl std::error::Error for TransitionError {}

#[cfg(test)]
mod tests {
    use crate::rid::{RuhIdentity, RuhProfile, XpRecordOutcome};

    use super::{
        TurnEvent, TurnPhase, TurnState, VerificationEvidence, VerificationOutcome,
        VerificationRequest, normalize_task_claim, verify_turn_and_award_xp,
    };

    fn reach_verification(state: &mut TurnState) {
        state.apply(TurnEvent::Start).unwrap();
        state.apply(TurnEvent::PlanAccepted).unwrap();
        state.apply(TurnEvent::ActionCompleted).unwrap();
    }

    fn profile() -> RuhProfile {
        RuhProfile::new(RuhIdentity {
            instance_id: "kernel-test".into(),
            created_at_unix: 1_800_000_000,
        })
        .unwrap()
    }

    fn verified_request(id: &str) -> VerificationRequest {
        VerificationRequest::new(
            "task-1",
            id,
            1_800_000_100,
            3,
            1,
            vec![
                VerificationEvidence::new(
                    "output-exists",
                    "present",
                    "present",
                    "test://artifact/1",
                    true,
                ),
                VerificationEvidence::new("format", "valid", "valid", "test://format/1", true),
            ],
        )
    }

    #[test]
    fn valid_verification_completes_turn_and_awards_xp_once() {
        let mut turn = TurnState::new("turn-verified", 1);
        let mut profile = profile();
        reach_verification(&mut turn);

        let outcome =
            verify_turn_and_award_xp(&mut turn, &mut profile, verified_request("verify-1"))
                .unwrap();

        let VerificationOutcome::Verified { receipt, xp_record } = outcome else {
            panic!("expected verification to pass");
        };
        assert_eq!(turn.phase, TurnPhase::Completed);
        assert_eq!(receipt.required_checks_passed(), 2);
        assert_eq!(receipt.xp_awarded(), 12);
        assert_eq!(xp_record, XpRecordOutcome::Recorded);
        assert_eq!(profile.total_xp(), 12);
    }

    #[test]
    fn receipt_binds_normalized_claim_and_detects_mutation() {
        let mut turn = TurnState::new("claim-bound-task", 0);
        let mut profile = profile();
        reach_verification(&mut turn);
        let request = VerificationRequest::new(
            "claim-bound-task",
            "claim-verification-1",
            1_800_000_101,
            1,
            1,
            vec![VerificationEvidence::new(
                "readback",
                "present",
                "present",
                "artifact:sha256:abc",
                true,
            )],
        )
        .with_claim("  Deployment   completed successfully  ");
        let outcome = verify_turn_and_award_xp(&mut turn, &mut profile, request).unwrap();
        let VerificationOutcome::Verified { receipt, .. } = outcome else {
            panic!("expected verified claim receipt");
        };
        assert_eq!(
            receipt.normalized_claim(),
            Some("deployment completed successfully")
        );
        assert_eq!(receipt.verifier_id(), "rocksoul.core.verifier");
        assert_eq!(receipt.verifier_version(), 1);
        assert!(receipt.integrity_valid());
        let mut tampered = receipt;
        tampered.normalized_claim = Some("deployment failed".to_owned());
        assert!(!tampered.integrity_valid());
        assert_eq!(normalize_task_claim("  A  B\nC "), "a b c");
    }

    #[test]
    fn failed_required_check_rejects_turn_without_xp() {
        let mut turn = TurnState::new("turn-rejected", 1);
        let mut profile = profile();
        reach_verification(&mut turn);

        let request = VerificationRequest::new(
            "task-2",
            "verify-2",
            1_800_000_100,
            3,
            1,
            vec![VerificationEvidence::new(
                "output-match",
                "expected",
                "different",
                "test://artifact/2",
                true,
            )],
        );
        let outcome = verify_turn_and_award_xp(&mut turn, &mut profile, request).unwrap();

        assert_eq!(
            outcome,
            VerificationOutcome::Rejected {
                failed_required_checks: vec!["output-match".into()]
            }
        );
        assert_eq!(turn.phase, TurnPhase::Failed);
        assert_eq!(profile.total_xp(), 0);
    }

    #[test]
    fn turn_outside_verification_cannot_award_xp() {
        let mut turn = TurnState::new("turn-early", 1);
        let mut profile = profile();

        assert!(
            verify_turn_and_award_xp(&mut turn, &mut profile, verified_request("verify-early"))
                .is_err()
        );
        assert_eq!(profile.total_xp(), 0);
        assert_eq!(turn.phase, TurnPhase::Idle);
    }

    #[test]
    fn duplicate_verification_event_is_idempotent() {
        let mut first_turn = TurnState::new("turn-first", 1);
        let mut profile = profile();
        reach_verification(&mut first_turn);
        let first = verify_turn_and_award_xp(
            &mut first_turn,
            &mut profile,
            verified_request("verify-replay"),
        )
        .unwrap();
        assert!(matches!(
            first,
            VerificationOutcome::Verified {
                xp_record: XpRecordOutcome::Recorded,
                ..
            }
        ));

        let mut replay_turn = TurnState::new("turn-replay", 1);
        reach_verification(&mut replay_turn);
        let replay = verify_turn_and_award_xp(
            &mut replay_turn,
            &mut profile,
            verified_request("verify-replay"),
        )
        .unwrap();
        assert!(matches!(
            replay,
            VerificationOutcome::Verified {
                xp_record: XpRecordOutcome::Duplicate,
                ..
            }
        ));
        assert_eq!(profile.total_xp(), 12);
    }

    #[test]
    fn terminal_states_cannot_be_reopened() {
        let mut state = TurnState::new("turn-terminal", 0);
        reach_verification(&mut state);
        state.apply(TurnEvent::VerificationPassed).unwrap();
        assert_eq!(
            state.apply(TurnEvent::Start),
            Err(super::TransitionError::TerminalState(TurnPhase::Completed))
        );
    }
}

/// Explicit upper bounds for one turn. These are enforced by TurnMeter.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TurnBudget {
    pub max_model_calls: u32,
    pub max_output_tokens: u32,
    pub timeout_ms: u64,
}

impl Default for TurnBudget {
    fn default() -> Self {
        Self {
            max_model_calls: 8,
            max_output_tokens: 16_000,
            timeout_ms: 120_000,
        }
    }
}

impl TurnBudget {
    pub fn validate(self) -> Result<Self, CoreError> {
        if self.max_model_calls == 0
            || self.max_output_tokens < self.max_model_calls
            || self.timeout_ms == 0
        {
            return Err(CoreError::InvalidBudget);
        }
        Ok(self)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TurnRequest {
    pub id: String,
    pub input: String,
    pub budget: TurnBudget,
}

impl TurnRequest {
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.id.trim().is_empty() || self.input.trim().is_empty() {
            return Err(CoreError::InvalidRequest);
        }
        self.budget.validate()?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlanStep {
    pub id: String,
    pub description: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AcceptanceCheck {
    pub id: String,
    pub description: String,
    pub required: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TurnPlan {
    pub objective: String,
    pub steps: Vec<PlanStep>,
    pub acceptance_checks: Vec<AcceptanceCheck>,
}

impl TurnPlan {
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.objective.trim().is_empty() || self.steps.is_empty() {
            return Err(CoreError::InvalidPlan);
        }
        let mut ids = HashSet::new();
        for step in &self.steps {
            if step.id.trim().is_empty()
                || step.description.trim().is_empty()
                || !ids.insert(step.id.as_str())
            {
                return Err(CoreError::InvalidPlan);
            }
        }
        let mut check_ids = HashSet::new();
        let mut has_required_check = false;
        for check in &self.acceptance_checks {
            if check.id.trim().is_empty()
                || check.description.trim().is_empty()
                || !check_ids.insert(check.id.as_str())
            {
                return Err(CoreError::InvalidPlan);
            }
            has_required_check |= check.required;
        }
        if self.acceptance_checks.is_empty() || !has_required_check {
            return Err(CoreError::InvalidPlan);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReflectionOutcome {
    Succeeded,
    NeedsRetry,
    Failed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TurnReflection {
    pub outcome: ReflectionOutcome,
    pub summary: String,
    pub issues: Vec<String>,
}

impl TurnReflection {
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.summary.trim().is_empty() {
            return Err(CoreError::InvalidReflection);
        }
        if self.issues.iter().any(|issue| issue.trim().is_empty()) {
            return Err(CoreError::InvalidReflection);
        }
        if self.outcome == ReflectionOutcome::Succeeded && !self.issues.is_empty() {
            return Err(CoreError::InvalidReflection);
        }
        Ok(())
    }
}

/// Cloneable cooperative cancellation signal. Executors must check it between
/// model calls and before performing any side effect.
#[derive(Clone, Debug, Default)]
pub struct CancellationToken(Arc<AtomicBool>);

impl CancellationToken {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }

    pub fn check(&self) -> Result<(), CoreError> {
        if self.is_cancelled() {
            Err(CoreError::Cancelled)
        } else {
            Ok(())
        }
    }
}

/// Per-turn accounting for model calls, generated tokens, and wall-clock time.
#[derive(Debug)]
pub struct TurnMeter {
    budget: TurnBudget,
    model_calls: u32,
    output_tokens: u32,
    started_at: Instant,
}

impl TurnMeter {
    pub fn new(budget: TurnBudget) -> Result<Self, CoreError> {
        Ok(Self {
            budget: budget.validate()?,
            model_calls: 0,
            output_tokens: 0,
            started_at: Instant::now(),
        })
    }

    pub fn before_model_call(&self, cancellation: &CancellationToken) -> Result<(), CoreError> {
        self.check_time_budget(cancellation)?;
        if self.model_calls >= self.budget.max_model_calls {
            return Err(CoreError::ModelCallBudgetExceeded);
        }
        Ok(())
    }

    /// Check cancellation and elapsed time without consuming a model-call slot.
    pub fn check_time_budget(&self, cancellation: &CancellationToken) -> Result<(), CoreError> {
        cancellation.check()?;
        self.check_deadline()
    }

    pub fn record_model_call(&mut self, cancellation: &CancellationToken) -> Result<(), CoreError> {
        self.before_model_call(cancellation)?;
        self.model_calls += 1;
        Ok(())
    }

    pub fn record_output_tokens(&mut self, tokens: u32) -> Result<(), CoreError> {
        self.check_deadline()?;
        let total = self.output_tokens.saturating_add(tokens);
        if total > self.budget.max_output_tokens {
            return Err(CoreError::OutputTokenBudgetExceeded);
        }
        self.output_tokens = total;
        Ok(())
    }

    pub fn model_calls(&self) -> u32 {
        self.model_calls
    }

    pub fn output_tokens(&self) -> u32 {
        self.output_tokens
    }

    /// A conservative per-request cap that keeps the sum of all allowed
    /// requests within the turn budget even when a provider omits usage data.
    pub fn max_tokens_per_call(&self) -> u32 {
        self.budget.max_output_tokens / self.budget.max_model_calls
    }

    pub fn elapsed(&self) -> Duration {
        self.started_at.elapsed()
    }

    fn check_deadline(&self) -> Result<(), CoreError> {
        if self.started_at.elapsed() > Duration::from_millis(self.budget.timeout_ms) {
            Err(CoreError::TimeBudgetExceeded)
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoreError {
    InvalidBudget,
    InvalidRequest,
    InvalidPlan,
    InvalidReflection,
    Cancelled,
    ModelCallBudgetExceeded,
    OutputTokenBudgetExceeded,
    TimeBudgetExceeded,
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidBudget => "turn budget values must be greater than zero",
            Self::InvalidRequest => "turn request requires a non-empty ID and input",
            Self::InvalidPlan => {
                "plan must contain unique, non-empty steps and required acceptance checks"
            }
            Self::InvalidReflection => "reflection is incomplete or internally inconsistent",
            Self::Cancelled => "turn was cancelled",
            Self::ModelCallBudgetExceeded => "model-call budget exceeded",
            Self::OutputTokenBudgetExceeded => "output-token budget exceeded",
            Self::TimeBudgetExceeded => "turn time budget exceeded",
        };
        f.write_str(message)
    }
}

impl std::error::Error for CoreError {}

#[cfg(test)]
mod bounded_contract_tests {
    use super::*;

    #[test]
    fn turn_request_requires_non_empty_input_and_positive_budgets() {
        let request = TurnRequest {
            id: "turn-1".into(),
            input: "hello".into(),
            budget: TurnBudget::default(),
        };
        assert!(request.validate().is_ok());
        let invalid = TurnRequest {
            input: " ".into(),
            ..request
        };
        assert_eq!(invalid.validate(), Err(CoreError::InvalidRequest));
    }

    #[test]
    fn plans_require_unique_steps_and_required_acceptance_checks() {
        let valid = TurnPlan {
            objective: "Complete bounded work".into(),
            steps: vec![PlanStep {
                id: "step-1".into(),
                description: "Inspect".into(),
            }],
            acceptance_checks: vec![AcceptanceCheck {
                id: "check-1".into(),
                description: "Output exists".into(),
                required: true,
            }],
        };
        assert!(valid.validate().is_ok());
        let mut invalid = valid;
        invalid.steps.push(invalid.steps[0].clone());
        assert_eq!(invalid.validate(), Err(CoreError::InvalidPlan));
    }

    #[test]
    fn cancellation_and_meter_enforce_turn_limits() {
        let cancellation = CancellationToken::default();
        let mut meter = TurnMeter::new(TurnBudget {
            max_model_calls: 1,
            max_output_tokens: 4,
            timeout_ms: 10_000,
        })
        .unwrap();
        meter.record_model_call(&cancellation).unwrap();
        assert_eq!(
            meter.record_model_call(&cancellation),
            Err(CoreError::ModelCallBudgetExceeded)
        );
        meter.record_output_tokens(4).unwrap();
        assert_eq!(
            meter.record_output_tokens(1),
            Err(CoreError::OutputTokenBudgetExceeded)
        );
        cancellation.cancel();
        assert_eq!(cancellation.check(), Err(CoreError::Cancelled));
    }

    #[test]
    fn successful_reflection_cannot_contain_unresolved_issues() {
        let reflection = TurnReflection {
            outcome: ReflectionOutcome::Succeeded,
            summary: "Completed".into(),
            issues: vec!["A required check failed".into()],
        };
        assert_eq!(reflection.validate(), Err(CoreError::InvalidReflection));
    }
}
