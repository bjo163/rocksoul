use crate::policy::{ActionMode, Guardian, PolicyError};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanStep {
    pub id: Uuid,
    pub intent: String,
    pub mode: ActionMode,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plan {
    pub id: Uuid,
    pub steps: Vec<PlanStep>,
    pub approval_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SimulationState {
    Planned,
    Simulated,
    Cancelled,
    Recovered,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimulationReport {
    pub plan_id: Uuid,
    pub state: SimulationState,
    pub completed_steps: usize,
    pub decisions: Vec<String>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PlanningError {
    #[error("plan is empty")]
    Empty,
    #[error("plan contains duplicate idempotency key")]
    DuplicateIdempotency,
    #[error("irreversible action is denied by default")]
    IrreversibleDenied,
    #[error("simulation cancelled")]
    Cancelled,
    #[error("policy failure: {0}")]
    Policy(#[from] PolicyError),
}

impl Plan {
    pub fn validate(&self) -> Result<(), PlanningError> {
        if self.steps.is_empty() {
            return Err(PlanningError::Empty);
        }
        for (index, step) in self.steps.iter().enumerate() {
            if step.idempotency_key.trim().is_empty()
                || self.steps[..index]
                    .iter()
                    .any(|previous| previous.idempotency_key == step.idempotency_key)
            {
                return Err(PlanningError::DuplicateIdempotency);
            }
            if matches!(step.mode, ActionMode::Privileged) && !self.approval_required {
                return Err(PlanningError::IrreversibleDenied);
            }
        }
        Ok(())
    }
    pub fn simulate(
        &self,
        guardian: &mut Guardian,
        cancel_after: Option<usize>,
    ) -> Result<SimulationReport, PlanningError> {
        self.validate()?;
        let mut decisions = Vec::new();
        for (index, step) in self.steps.iter().enumerate() {
            if cancel_after == Some(index) {
                return Ok(SimulationReport {
                    plan_id: self.id,
                    state: SimulationState::Cancelled,
                    completed_steps: index,
                    decisions,
                });
            }
            guardian.consume_step()?;
            guardian.decide(step.mode, false)?;
            decisions.push(format!("SIMULATED:{}", step.intent));
        }
        Ok(SimulationReport {
            plan_id: self.id,
            state: SimulationState::Simulated,
            completed_steps: self.steps.len(),
            decisions,
        })
    }
    pub fn recover(report: &SimulationReport) -> SimulationReport {
        SimulationReport {
            plan_id: report.plan_id,
            state: SimulationState::Recovered,
            completed_steps: report.completed_steps,
            decisions: report.decisions.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn plan(mode: ActionMode) -> Plan {
        Plan {
            id: Uuid::new_v4(),
            approval_required: false,
            steps: vec![PlanStep {
                id: Uuid::new_v4(),
                intent: "inspect".into(),
                mode,
                idempotency_key: "inspect-1".into(),
            }],
        }
    }
    #[test]
    fn simulation_is_dry_run_and_budgeted() {
        let p = plan(ActionMode::Simulated);
        let mut guardian = Guardian::default();
        let report = p.simulate(&mut guardian, None).unwrap();
        assert_eq!(report.state, SimulationState::Simulated);
        assert_eq!(report.completed_steps, 1);
        assert_eq!(report.decisions[0], "SIMULATED:inspect");
    }
    #[test]
    fn duplicate_and_irreversible_plans_are_rejected() {
        let mut p = plan(ActionMode::Simulated);
        p.steps.push(PlanStep {
            id: Uuid::new_v4(),
            intent: "again".into(),
            mode: ActionMode::Simulated,
            idempotency_key: "inspect-1".into(),
        });
        assert_eq!(p.validate(), Err(PlanningError::DuplicateIdempotency));
        assert_eq!(
            plan(ActionMode::Privileged).validate(),
            Err(PlanningError::IrreversibleDenied)
        );
    }
    #[test]
    fn cancellation_is_recoverable_without_executing_actions() {
        let p = plan(ActionMode::Simulated);
        let mut guardian = Guardian::default();
        let report = p.simulate(&mut guardian, Some(0)).unwrap();
        assert_eq!(report.state, SimulationState::Cancelled);
        assert_eq!(Plan::recover(&report).state, SimulationState::Recovered);
    }
}
