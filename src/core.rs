use std::fmt;

use serde::{Deserialize, Serialize};

/// Deterministic lifecycle phases for one cognitive turn.
///
/// This state machine records orchestration state only. It does not invoke a
/// model, execute tools, or persist data.
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
                write!(formatter, "event {event:?} is invalid while turn is {phase:?}")
            }
            Self::RetryBudgetExhausted { max_retries } => {
                write!(formatter, "turn retry budget exhausted (maximum {max_retries})")
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
    use super::{TransitionError, TurnEvent, TurnPhase, TurnState};

    fn reach_verification(state: &mut TurnState) {
        state.apply(TurnEvent::Start).unwrap();
        state.apply(TurnEvent::PlanAccepted).unwrap();
        state.apply(TurnEvent::ActionCompleted).unwrap();
    }

    #[test]
    fn completes_a_valid_turn() {
        let mut state = TurnState::new("turn-1", 1);
        reach_verification(&mut state);
        state.apply(TurnEvent::VerificationPassed).unwrap();

        assert_eq!(state.phase, TurnPhase::Completed);
        assert_eq!(state.retry_count, 0);
    }

    #[test]
    fn retry_returns_to_planning_and_increments_counter() {
        let mut state = TurnState::new("turn-2", 1);
        reach_verification(&mut state);
        state.apply(TurnEvent::RetryRequested).unwrap();

        assert_eq!(state.phase, TurnPhase::Planning);
        assert_eq!(state.retry_count, 1);
    }

    #[test]
    fn exhausted_retry_budget_does_not_change_state() {
        let mut state = TurnState::new("turn-3", 0);
        reach_verification(&mut state);
        let before = state.clone();

        assert_eq!(
            state.apply(TurnEvent::RetryRequested),
            Err(TransitionError::RetryBudgetExhausted { max_retries: 0 })
        );
        assert_eq!(state, before);
    }

    #[test]
    fn rejects_invalid_transition_without_mutation() {
        let mut state = TurnState::new("turn-4", 1);

        assert!(matches!(
            state.apply(TurnEvent::PlanAccepted),
            Err(TransitionError::InvalidTransition {
                phase: TurnPhase::Idle,
                event: TurnEvent::PlanAccepted
            })
        ));
        assert_eq!(state.phase, TurnPhase::Idle);
    }

    #[test]
    fn terminal_states_cannot_be_reopened() {
        let mut state = TurnState::new("turn-5", 0);
        reach_verification(&mut state);
        state.apply(TurnEvent::VerificationPassed).unwrap();

        assert_eq!(
            state.apply(TurnEvent::Start),
            Err(TransitionError::TerminalState(TurnPhase::Completed))
        );
    }

    #[test]
    fn cancellation_is_terminal() {
        let mut state = TurnState::new("turn-6", 0);
        state.apply(TurnEvent::Cancel).unwrap();

        assert_eq!(state.phase, TurnPhase::Cancelled);
        assert!(state.phase.is_terminal());
    }
}
