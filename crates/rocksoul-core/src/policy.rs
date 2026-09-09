use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionMode {
    ReadOnly,
    Simulated,
    Reversible,
    Privileged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyDecision {
    Allow,
    Deny,
    Defer,
    ApprovalRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Budget {
    pub steps: u32,
    pub tools: u32,
    pub replans: u32,
    pub risk: u8,
}

impl Default for Budget {
    fn default() -> Self {
        Self {
            steps: 32,
            tools: 8,
            replans: 3,
            risk: 0,
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PolicyError {
    #[error("action requires explicit approval")]
    ApprovalRequired,
    #[error("budget exhausted: {0}")]
    BudgetExhausted(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Guardian {
    pub default_mode: ActionMode,
    pub budget: Budget,
    pub decisions: Vec<PolicyDecision>,
}

impl Default for Guardian {
    fn default() -> Self {
        Self {
            default_mode: ActionMode::ReadOnly,
            budget: Budget::default(),
            decisions: Vec::new(),
        }
    }
}

impl Guardian {
    pub fn decide(
        &mut self,
        mode: ActionMode,
        authorized: bool,
    ) -> Result<PolicyDecision, PolicyError> {
        let decision = match mode {
            ActionMode::ReadOnly | ActionMode::Simulated => PolicyDecision::Allow,
            ActionMode::Reversible if authorized => PolicyDecision::Allow,
            ActionMode::Reversible => PolicyDecision::ApprovalRequired,
            ActionMode::Privileged if authorized => PolicyDecision::Allow,
            ActionMode::Privileged => PolicyDecision::ApprovalRequired,
        };
        self.decisions.push(decision);
        if decision == PolicyDecision::ApprovalRequired {
            return Err(PolicyError::ApprovalRequired);
        }
        Ok(decision)
    }
    pub fn consume_step(&mut self) -> Result<(), PolicyError> {
        if self.budget.steps == 0 {
            Err(PolicyError::BudgetExhausted("steps"))
        } else {
            self.budget.steps -= 1;
            Ok(())
        }
    }
    pub fn consume_tool(&mut self) -> Result<(), PolicyError> {
        if self.budget.tools == 0 {
            Err(PolicyError::BudgetExhausted("tools"))
        } else {
            self.budget.tools -= 1;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn guardian_is_deny_by_default_for_privilege() {
        let mut g = Guardian::default();
        assert_eq!(
            g.decide(ActionMode::Privileged, false),
            Err(PolicyError::ApprovalRequired)
        );
    }
    #[test]
    fn budgets_stop_execution() {
        let mut g = Guardian {
            budget: Budget {
                steps: 1,
                ..Budget::default()
            },
            ..Guardian::default()
        };
        g.consume_step().unwrap();
        assert_eq!(g.consume_step(), Err(PolicyError::BudgetExhausted("steps")));
    }
}
