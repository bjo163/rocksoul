use crate::cognition::{EpistemicStatus, Observation, Provenance};
use chrono::Utc;
use serde_json::Value;
use std::path::{Path, PathBuf};
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

#[derive(Debug, Clone)]
pub struct RepositorySense {
    pub root: PathBuf,
}

impl RepositorySense {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }
}

impl SenseConnector for RepositorySense {
    fn name(&self) -> &str {
        "local-repository"
    }
    fn observe(&self, subject: &str) -> Result<Observation, SenseError> {
        let head = std::fs::read_to_string(self.root.join(".git").join("HEAD"))
            .map_err(|_| SenseError::Offline)?;
        let payload = serde_json::json!({ "repository": subject, "head": head.trim() });
        Ok(Observation {
            id: Uuid::new_v4(),
            connector: self.name().into(),
            subject: subject.into(),
            payload,
            provenance: Provenance {
                source: self.root.display().to_string(),
                observed_at: Utc::now(),
                scope: subject.into(),
                payload_hash: None,
            },
            status: EpistemicStatus::Observed,
        })
    }
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

    #[test]
    fn repository_connector_is_read_only_observation() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let observation = RepositorySense::new(root)
            .observe("rocksoul-cognitive-runtime")
            .unwrap();
        assert_eq!(observation.status, EpistemicStatus::Observed);
        assert!(
            observation.payload["head"]
                .as_str()
                .is_some_and(|head| !head.is_empty())
        );
    }
}
