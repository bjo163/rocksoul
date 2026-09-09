use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Identity {
    pub id: Uuid,
    pub name: String,
    pub generation: u32,
    pub born_at: DateTime<Utc>,
}

impl Identity {
    #[must_use]
    pub fn born_now() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "RockSoul".to_owned(),
            generation: 1,
            born_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RockSoulStatus {
    Idle,
    Listening,
    Exploring,
    Reading,
    Thinking,
    Planning,
    Executing,
    Verifying,
    Learning,
}

impl RockSoulStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "IDLE",
            Self::Listening => "LISTENING",
            Self::Exploring => "EXPLORING",
            Self::Reading => "READING",
            Self::Thinking => "THINKING",
            Self::Planning => "PLANNING",
            Self::Executing => "EXECUTING",
            Self::Verifying => "VERIFYING",
            Self::Learning => "LEARNING",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventKind {
    Born,
    StatusChanged {
        from: RockSoulStatus,
        to: RockSoulStatus,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub kind: EventKind,
}

impl Event {
    #[must_use]
    pub fn now(kind: EventKind) -> Self {
        Self {
            id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            kind,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newborn_identity_has_expected_defaults() {
        let identity = Identity::born_now();

        assert_eq!(identity.name, "RockSoul");
        assert_eq!(identity.generation, 1);
    }

    #[test]
    fn status_has_stable_machine_label() {
        assert_eq!(RockSoulStatus::Verifying.as_str(), "VERIFYING");
    }
}
