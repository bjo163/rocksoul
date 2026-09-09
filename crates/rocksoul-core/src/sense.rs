use crate::cognition::{EpistemicStatus, Observation, Provenance};
use chrono::Utc;
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum SenseError {
    #[error("connector payload exceeds limit")]
    PayloadTooLarge,
    #[error("connector is offline")]
    Offline,
}

pub trait SenseConnector {
    fn name(&self) -> &str;
    fn observe(&self, subject: &str) -> Result<Observation, SenseError>;
}

#[derive(Debug, Clone)]
pub struct FixtureSense {
    pub name: String,
    pub payload: Value,
    pub max_bytes: usize,
    pub online: bool,
}

impl SenseConnector for FixtureSense {
    fn name(&self) -> &str {
        &self.name
    }
    fn observe(&self, subject: &str) -> Result<Observation, SenseError> {
        if !self.online {
            return Err(SenseError::Offline);
        }
        let encoded = serde_json::to_vec(&self.payload).expect("JSON values are serializable");
        if encoded.len() > self.max_bytes {
            return Err(SenseError::PayloadTooLarge);
        }
        Ok(Observation {
            id: Uuid::new_v4(),
            connector: self.name.clone(),
            subject: subject.to_owned(),
            payload: self.payload.clone(),
            provenance: Provenance {
                source: self.name.clone(),
                observed_at: Utc::now(),
                scope: subject.to_owned(),
                payload_hash: None,
            },
            status: EpistemicStatus::Observed,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixture_is_observation_only() {
        let c = FixtureSense {
            name: "fixture".into(),
            payload: serde_json::json!({"ok":true}),
            max_bytes: 100,
            online: true,
        };
        let o = c.observe("local").unwrap();
        assert_eq!(o.status, EpistemicStatus::Observed);
    }
    #[test]
    fn offline_and_oversize_are_explicit() {
        let c = FixtureSense {
            name: "fixture".into(),
            payload: serde_json::json!({"x":"long"}),
            max_bytes: 2,
            online: false,
        };
        assert!(matches!(c.observe("x"), Err(SenseError::Offline)));
        let c = FixtureSense { online: true, ..c };
        assert!(matches!(c.observe("x"), Err(SenseError::PayloadTooLarge)));
    }
}
