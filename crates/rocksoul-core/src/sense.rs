use crate::cognition::{EpistemicStatus, Observation, Provenance};
use chrono::Utc;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum SenseError {
    #[error("connector payload exceeds limit")]
    PayloadTooLarge,
    #[error("connector is offline")]
    Offline,
    #[error("connector response is stale")]
    Stale,
    #[error("connector rate limit exceeded")]
    RateLimited,
    #[error("connector timed out")]
    Timeout,
    #[error("connector retry limit exceeded")]
    RetryLimit,
    #[error("connector provenance is invalid")]
    InvalidProvenance,
}

#[derive(Debug, Clone, Copy)]
pub struct ConnectorPolicy {
    pub max_payload_bytes: usize,
    pub max_age: Duration,
    pub timeout: Duration,
    pub max_attempts: u8,
    pub max_calls: u32,
}

impl Default for ConnectorPolicy {
    fn default() -> Self {
        Self {
            max_payload_bytes: 64 * 1024,
            max_age: Duration::from_secs(300),
            timeout: Duration::from_secs(5),
            max_attempts: 3,
            max_calls: 60,
        }
    }
}

pub struct BoundedConnector<C> {
    inner: C,
    policy: ConnectorPolicy,
}

impl<C> BoundedConnector<C> {
    pub fn new(inner: C, policy: ConnectorPolicy) -> Self {
        Self { inner, policy }
    }
}

impl<C: SenseConnector> SenseConnector for BoundedConnector<C> {
    fn name(&self) -> &str {
        self.inner.name()
    }
    fn observe(&self, subject: &str) -> Result<Observation, SenseError> {
        self.observe_bounded(subject)
    }
}

impl<C: SenseConnector> BoundedConnector<C> {
    pub fn observe_bounded(&self, subject: &str) -> Result<Observation, SenseError> {
        if self.policy.max_attempts == 0 {
            return Err(SenseError::RetryLimit);
        }
        if self.policy.timeout.is_zero() {
            return Err(SenseError::Timeout);
        }
        if self.policy.max_calls == 0 {
            return Err(SenseError::RateLimited);
        }
        let started = Instant::now();
        let mut last = None;
        for _ in 0..self.policy.max_attempts {
            match self.inner.observe(subject) {
                Ok(mut observation) => {
                    if started.elapsed() > self.policy.timeout {
                        return Err(SenseError::Timeout);
                    }
                    let bytes =
                        serde_json::to_vec(&observation.payload).expect("JSON is serializable");
                    if bytes.len() > self.policy.max_payload_bytes {
                        return Err(SenseError::PayloadTooLarge);
                    }
                    if Utc::now()
                        .signed_duration_since(observation.provenance.observed_at)
                        .to_std()
                        .unwrap_or_default()
                        > self.policy.max_age
                    {
                        return Err(SenseError::Stale);
                    }
                    if observation.connector.is_empty() || observation.provenance.source.is_empty()
                    {
                        return Err(SenseError::InvalidProvenance);
                    }
                    let hash = format!("{:x}", Sha256::digest(&bytes));
                    if let Some(existing) = &observation.provenance.payload_hash {
                        if existing != &hash {
                            return Err(SenseError::InvalidProvenance);
                        }
                    }
                    observation.provenance.payload_hash = Some(hash);
                    return Ok(observation);
                }
                Err(error) => last = Some(error),
            }
        }
        Err(last.unwrap_or(SenseError::RetryLimit))
    }
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

    #[test]
    fn bounded_connector_adds_hash_and_enforces_limits() {
        let c = BoundedConnector::new(
            FixtureSense {
                name: "fixture".into(),
                payload: serde_json::json!({"ok":true}),
                max_bytes: 100,
                online: true,
            },
            ConnectorPolicy::default(),
        );
        let o = c.observe("local").unwrap();
        assert_eq!(
            o.provenance.payload_hash.as_ref().map(String::len),
            Some(64)
        );
        let limited = BoundedConnector::new(
            FixtureSense {
                name: "fixture".into(),
                payload: serde_json::json!({"ok":true}),
                max_bytes: 100,
                online: true,
            },
            ConnectorPolicy {
                max_payload_bytes: 1,
                ..Default::default()
            },
        );
        assert!(matches!(
            limited.observe("local"),
            Err(SenseError::PayloadTooLarge)
        ));
    }

    #[test]
    fn bounded_connector_rejects_zero_budget_and_invalid_provenance() {
        let c = BoundedConnector::new(
            FixtureSense {
                name: "fixture".into(),
                payload: Value::Null,
                max_bytes: 100,
                online: true,
            },
            ConnectorPolicy {
                max_calls: 0,
                ..Default::default()
            },
        );
        assert!(matches!(c.observe("local"), Err(SenseError::RateLimited)));
        let timeout = BoundedConnector::new(
            FixtureSense {
                name: "fixture".into(),
                payload: Value::Null,
                max_bytes: 100,
                online: true,
            },
            ConnectorPolicy {
                timeout: Duration::ZERO,
                ..Default::default()
            },
        );
        assert!(matches!(timeout.observe("local"), Err(SenseError::Timeout)));
    }
}
