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

    /// Invalid transitions never mutate the state.
    pub fn apply(&mut self, event: TurnEvent) -> Result<(), TransitionError> {
        if self.phase.is_terminal() {
            return Err(TransitionError::TerminalState(self.phase));
        }
        if event == TurnEvent::Fail {
            self.phase = TurnPhase::Failed;
            return Ok(());
        }
        if event == TurnEvent::Cancel {
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTransition { phase, event } => {
                write!(f, "event {event:?} is invalid while turn is {phase:?}")
            }
            Self::RetryBudgetExhausted { max_retries } => {
                write!(f, "turn retry budget exhausted (maximum {max_retries})")
            }
            Self::TerminalState(phase) => {
                write!(f, "terminal turn state {phase:?} cannot transition")
            }
        }
    }
}

impl std::error::Error for TransitionError {}

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
        if self.max_model_calls == 0 || self.max_output_tokens == 0 || self.timeout_ms == 0 {
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
    pub fn check_time_budget(
        &self,
        cancellation: &CancellationToken,
    ) -> Result<(), CoreError> {
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
mod tests {
    use super::*;

    fn valid_plan() -> TurnPlan {
        TurnPlan {
            objective: "Complete a bounded task".into(),
            steps: vec![PlanStep {
                id: "step-1".into(),
                description: "Inspect inputs".into(),
            }],
            acceptance_checks: vec![AcceptanceCheck {
                id: "check-1".into(),
                description: "Expected result is present".into(),
                required: true,
            }],
        }
    }

    #[test]
    fn valid_turn_completes_and_terminal_state_is_immutable() {
        let mut state = TurnState::new("turn-1", 1);
        state.apply(TurnEvent::Start).unwrap();
        state.apply(TurnEvent::PlanAccepted).unwrap();
        state.apply(TurnEvent::ActionCompleted).unwrap();
        state.apply(TurnEvent::VerificationPassed).unwrap();
        assert_eq!(state.phase, TurnPhase::Completed);
        assert_eq!(
            state.apply(TurnEvent::Start),
            Err(TransitionError::TerminalState(TurnPhase::Completed))
        );
    }

    #[test]
    fn retry_budget_is_enforced_without_state_mutation() {
        let mut state = TurnState::new("turn-2", 0);
        state.apply(TurnEvent::Start).unwrap();
        state.apply(TurnEvent::PlanAccepted).unwrap();
        state.apply(TurnEvent::ActionCompleted).unwrap();
        let before = state.clone();
        assert_eq!(
            state.apply(TurnEvent::RetryRequested),
            Err(TransitionError::RetryBudgetExhausted { max_retries: 0 })
        );
        assert_eq!(state, before);
    }

    #[test]
    fn plan_requires_unique_steps_and_required_acceptance_check() {
        assert!(valid_plan().validate().is_ok());
        let mut invalid = valid_plan();
        invalid.steps.push(invalid.steps[0].clone());
        assert_eq!(invalid.validate(), Err(CoreError::InvalidPlan));
    }

    #[test]
    fn reflection_rejects_success_with_unresolved_issues() {
        let reflection = TurnReflection {
            outcome: ReflectionOutcome::Succeeded,
            summary: "Done".into(),
            issues: vec!["A required check failed".into()],
        };
        assert_eq!(reflection.validate(), Err(CoreError::InvalidReflection));
    }

    #[test]
    fn cancellation_is_shared_across_clones() {
        let token = CancellationToken::default();
        let worker = token.clone();
        token.cancel();
        assert!(worker.is_cancelled());
        assert_eq!(worker.check(), Err(CoreError::Cancelled));
    }

    #[test]
    fn meter_enforces_model_call_and_token_budgets() {
        let budget = TurnBudget {
            max_model_calls: 1,
            max_output_tokens: 4,
            timeout_ms: 10_000,
        };
        let token = CancellationToken::default();
        let mut meter = TurnMeter::new(budget).unwrap();
        meter.record_model_call(&token).unwrap();
        assert_eq!(
            meter.record_model_call(&token),
            Err(CoreError::ModelCallBudgetExceeded)
        );
        meter.record_output_tokens(4).unwrap();
        assert_eq!(
            meter.record_output_tokens(1),
            Err(CoreError::OutputTokenBudgetExceeded)
        );
    }

    #[test]
    fn elapsed_time_check_does_not_consume_a_model_call_slot() {
        let budget = TurnBudget {
            max_model_calls: 1,
            max_output_tokens: 10,
            timeout_ms: 10_000,
        };
        let token = CancellationToken::default();
        let mut meter = TurnMeter::new(budget).unwrap();
        meter.record_model_call(&token).unwrap();
        assert!(meter.check_time_budget(&token).is_ok());
        assert_eq!(meter.model_calls(), 1);
    }

    #[test]
    fn request_rejects_empty_input() {
        let request = TurnRequest {
            id: "turn-3".into(),
            input: " ".into(),
            budget: TurnBudget::default(),
        };
        assert_eq!(request.validate(), Err(CoreError::InvalidRequest));
    }
}
