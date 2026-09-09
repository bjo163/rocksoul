use crate::{LifeProgressionEngine, LifeState, ProgressionReport, VerifiedOutcome};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProgressionPolicy {
    pub max_xp_per_outcome: u64,
    pub max_trust_delta: i16,
    pub max_age_step: u16,
}

impl Default for ProgressionPolicy {
    fn default() -> Self {
        Self {
            max_xp_per_outcome: 10_000,
            max_trust_delta: 10,
            max_age_step: 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustLedgerEntry {
    pub id: Uuid,
    pub outcome_id: Uuid,
    pub verified: bool,
    pub evaluation_passed: bool,
    pub trust_delta: i16,
    pub resulting_trust: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProgressionLedger {
    pub entries: Vec<TrustLedgerEntry>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ProgressionError {
    #[error("outcome is not verified")]
    Unverified,
    #[error("evaluation did not pass")]
    EvaluationFailed,
    #[error("XP exceeds policy bound")]
    XpLimit,
    #[error("trust delta exceeds policy bound")]
    TrustDeltaLimit,
    #[error("cognitive age step exceeds policy bound")]
    AgeStepLimit,
}

impl ProgressionLedger {
    pub fn apply(
        &mut self,
        state: &mut LifeState,
        outcome_id: Uuid,
        outcome: VerifiedOutcome,
        policy: ProgressionPolicy,
    ) -> Result<ProgressionReport, ProgressionError> {
        if !outcome.verified {
            return Err(ProgressionError::Unverified);
        }
        if !outcome.evaluation_passed {
            return Err(ProgressionError::EvaluationFailed);
        }
        if outcome.xp > policy.max_xp_per_outcome {
            return Err(ProgressionError::XpLimit);
        }
        if outcome.trust_delta.unsigned_abs() > policy.max_trust_delta.unsigned_abs() {
            return Err(ProgressionError::TrustDeltaLimit);
        }
        if let Some(age) = outcome.cognitive_age {
            if age.saturating_sub(state.cognitive_age) > policy.max_age_step {
                return Err(ProgressionError::AgeStepLimit);
            }
        }
        let report = LifeProgressionEngine::apply(state, outcome);
        self.entries.push(TrustLedgerEntry {
            id: Uuid::new_v4(),
            outcome_id,
            verified: true,
            evaluation_passed: true,
            trust_delta: outcome.trust_delta,
            resulting_trust: state.trust,
        });
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn outcome() -> VerifiedOutcome {
        VerifiedOutcome {
            verified: true,
            xp: 100,
            evaluation_passed: true,
            cognitive_age: Some(1),
            trust_delta: 5,
        }
    }
    #[test]
    fn policy_applies_bounded_progression_and_appends_trust_ledger() {
        let mut state = LifeState::born_now();
        let mut ledger = ProgressionLedger::default();
        let report = ledger
            .apply(
                &mut state,
                Uuid::new_v4(),
                outcome(),
                ProgressionPolicy::default(),
            )
            .unwrap();
        assert_eq!(
            (report.new_level, report.new_age, report.new_trust),
            (2, 1, 5)
        );
        assert_eq!(ledger.entries.len(), 1);
        assert_eq!(ledger.entries[0].resulting_trust, 5);
    }
    #[test]
    fn failed_or_unverified_outcomes_cannot_reward_progression() {
        let mut state = LifeState::born_now();
        let original = state.clone();
        let mut ledger = ProgressionLedger::default();
        let mut rejected = outcome();
        rejected.verified = false;
        assert_eq!(
            ledger.apply(
                &mut state,
                Uuid::new_v4(),
                rejected,
                ProgressionPolicy::default()
            ),
            Err(ProgressionError::Unverified)
        );
        rejected = outcome();
        rejected.evaluation_passed = false;
        assert_eq!(
            ledger.apply(
                &mut state,
                Uuid::new_v4(),
                rejected,
                ProgressionPolicy::default()
            ),
            Err(ProgressionError::EvaluationFailed)
        );
        assert_eq!(state, original);
        assert!(ledger.entries.is_empty());
    }
    #[test]
    fn policy_rejects_excessive_delta_without_mutating_state_or_ledger() {
        let mut state = LifeState::born_now();
        let original = state.clone();
        let mut ledger = ProgressionLedger::default();
        let policy = ProgressionPolicy {
            max_trust_delta: 1,
            ..Default::default()
        };
        assert_eq!(
            ledger.apply(&mut state, Uuid::new_v4(), outcome(), policy),
            Err(ProgressionError::TrustDeltaLimit)
        );
        assert_eq!(state, original);
        assert!(ledger.entries.is_empty());
    }
}
