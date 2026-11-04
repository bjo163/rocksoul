use std::fmt;

use serde::{Deserialize, Serialize};

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
#[derive(Clone, Debug, Eq, PartialEq)]
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
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerificationReceipt {
    verification_id: String,
    task_id: String,
    required_checks_passed: usize,
    total_checks: usize,
    evidence_refs: Vec<String>,
    xp_awarded: u32,
    verified_at_unix: u64,
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
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VerificationOutcome {
    Verified {
        receipt: VerificationReceipt,
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

    Ok(VerificationOutcome::Verified {
        receipt: VerificationReceipt {
            verification_id: request.verification_id,
            task_id: request.task_id,
            required_checks_passed: passed_required,
            total_checks: request.checks.len(),
            evidence_refs,
            xp_awarded,
            verified_at_unix: request.verified_at_unix,
        },
        xp_record,
    })
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
        VerificationRequest, verify_turn_and_award_xp,
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
