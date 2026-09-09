use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CognitiveEventKind {
    Observation,
    Evaluation,
    Inference,
    Policy,
    Progression,
    Recovery,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CognitiveEvent {
    pub id: Uuid,
    pub kind: CognitiveEventKind,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CognitiveRecord {
    pub schema_version: u16,
    pub sequence: u64,
    pub event: CognitiveEvent,
    pub previous_hash: String,
    pub hash: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum EventLogError {
    #[error("unsupported event schema {0}")]
    UnsupportedSchema(u16),
    #[error("event sequence expected {expected}, got {actual}")]
    Sequence { expected: u64, actual: u64 },
    #[error("event hash chain is invalid")]
    Tampered,
    #[error("duplicate event id")]
    Duplicate,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CognitiveEventLog {
    records: Vec<CognitiveRecord>,
}

impl CognitiveEventLog {
    pub fn append(&mut self, event: CognitiveEvent) -> CognitiveRecord {
        let sequence = self.records.len() as u64 + 1;
        let previous_hash = self
            .records
            .last()
            .map(|r| r.hash.clone())
            .unwrap_or_default();
        let hash = Self::hash(sequence, &event, &previous_hash);
        let record = CognitiveRecord {
            schema_version: 1,
            sequence,
            event,
            previous_hash,
            hash,
        };
        self.records.push(record.clone());
        record
    }
    pub fn records(&self) -> &[CognitiveRecord] {
        &self.records
    }
    pub fn replay(records: &[CognitiveRecord]) -> Result<Self, EventLogError> {
        let mut log = Self::default();
        for record in records {
            if record.schema_version != 1 {
                return Err(EventLogError::UnsupportedSchema(record.schema_version));
            }
            let expected_sequence = log.records.len() as u64 + 1;
            if record.sequence != expected_sequence {
                return Err(EventLogError::Sequence {
                    expected: expected_sequence,
                    actual: record.sequence,
                });
            }
            if log
                .records
                .iter()
                .any(|item| item.event.id == record.event.id)
            {
                return Err(EventLogError::Duplicate);
            }
            let previous = log
                .records
                .last()
                .map(|item| item.hash.as_str())
                .unwrap_or("");
            if record.previous_hash != previous
                || record.hash != Self::hash(record.sequence, &record.event, &record.previous_hash)
            {
                return Err(EventLogError::Tampered);
            }
            log.records.push(record.clone());
        }
        Ok(log)
    }
    fn hash(sequence: u64, event: &CognitiveEvent, previous_hash: &str) -> String {
        let bytes =
            serde_json::to_vec(&(sequence, event, previous_hash)).expect("event is serializable");
        format!("{:x}", Sha256::digest(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn event(kind: CognitiveEventKind, value: &str) -> CognitiveEvent {
        CognitiveEvent {
            id: Uuid::new_v4(),
            kind,
            payload: serde_json::json!({"value":value}),
        }
    }
    #[test]
    fn event_log_replays_all_cognitive_domains() {
        let mut log = CognitiveEventLog::default();
        for kind in [
            CognitiveEventKind::Observation,
            CognitiveEventKind::Evaluation,
            CognitiveEventKind::Inference,
            CognitiveEventKind::Policy,
            CognitiveEventKind::Progression,
            CognitiveEventKind::Recovery,
        ] {
            log.append(event(kind, "ok"));
        }
        let replayed = CognitiveEventLog::replay(log.records()).unwrap();
        assert_eq!(replayed.records().len(), 6);
    }
    #[test]
    fn replay_rejects_gap_duplicate_tamper_and_unknown_schema() {
        let mut log = CognitiveEventLog::default();
        log.append(event(CognitiveEventKind::Observation, "one"));
        log.append(event(CognitiveEventKind::Policy, "two"));
        let mut gap = log.records().to_vec();
        gap[1].sequence = 3;
        assert!(matches!(
            CognitiveEventLog::replay(&gap),
            Err(EventLogError::Sequence { .. })
        ));
        let mut tampered = log.records().to_vec();
        tampered[0].event.payload = serde_json::json!({"changed":true});
        assert_eq!(
            CognitiveEventLog::replay(&tampered),
            Err(EventLogError::Tampered)
        );
        let mut unknown = log.records().to_vec();
        unknown[0].schema_version = 2;
        assert_eq!(
            CognitiveEventLog::replay(&unknown),
            Err(EventLogError::UnsupportedSchema(2))
        );
        let duplicate = vec![log.records()[0].clone(), log.records()[0].clone()];
        assert_eq!(
            CognitiveEventLog::replay(&duplicate),
            Err(EventLogError::Sequence {
                expected: 2,
                actual: 1
            })
        );
    }
}
