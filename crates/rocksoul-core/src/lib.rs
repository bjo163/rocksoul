use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
};
use thiserror::Error;
use uuid::Uuid;

pub mod brain;
pub mod brain_adapter;
pub mod cognition;
pub mod event_log;
pub mod inference;
pub mod memory;
pub mod policy;
pub mod sense;
pub mod sense_registry;
pub mod skills;
pub mod workspace;
pub mod world;

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

#[derive(Debug, Error)]
pub enum JournalError {
    #[error("journal I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("journal record {line} is invalid: {source}")]
    InvalidRecord {
        line: usize,
        source: serde_json::Error,
    },
    #[error("journal record {line} has schema version {version}, expected 1")]
    UnsupportedSchema { line: usize, version: u16 },
    #[error("journal sequence must be strictly increasing: previous {previous}, current {current}")]
    InvalidSequence { previous: u64, current: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalRecord {
    pub schema_version: u16,
    pub sequence: u64,
    pub runtime_id: Uuid,
    pub event: Event,
}

#[derive(Debug, Clone)]
pub struct LifecycleJournal {
    path: PathBuf,
    runtime_id: Uuid,
    next_sequence: u64,
}

impl LifecycleJournal {
    pub fn open(path: impl AsRef<Path>, runtime_id: Uuid) -> Result<Self, JournalError> {
        let path = path.as_ref().to_path_buf();
        let mut next_sequence = 1;
        if path.exists() {
            let records = Self::read_records(&path)?;
            next_sequence = records.last().map_or(1, |record| record.sequence + 1);
        }
        Ok(Self {
            path,
            runtime_id,
            next_sequence,
        })
    }

    pub fn append(&mut self, event: Event) -> Result<JournalRecord, JournalError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let record = JournalRecord {
            schema_version: 1,
            sequence: self.next_sequence,
            runtime_id: self.runtime_id,
            event,
        };
        let encoded = serde_json::to_string(&record).expect("journal record is serializable");
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        writeln!(file, "{encoded}")?;
        file.flush()?;
        self.next_sequence += 1;
        Ok(record)
    }

    pub fn replay(&self) -> Result<Vec<JournalRecord>, JournalError> {
        Self::read_records(&self.path)
    }

    fn read_records(path: &Path) -> Result<Vec<JournalRecord>, JournalError> {
        let file = File::open(path)?;
        let mut records = Vec::new();
        let mut previous = 0;
        for (index, line) in BufReader::new(file).lines().enumerate() {
            let line_number = index + 1;
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let record: JournalRecord =
                serde_json::from_str(&line).map_err(|source| JournalError::InvalidRecord {
                    line: line_number,
                    source,
                })?;
            if record.schema_version != 1 {
                return Err(JournalError::UnsupportedSchema {
                    line: line_number,
                    version: record.schema_version,
                });
            }
            if record.sequence <= previous {
                return Err(JournalError::InvalidSequence {
                    previous,
                    current: record.sequence,
                });
            }
            previous = record.sequence;
            records.push(record);
        }
        Ok(records)
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

    #[test]
    fn journal_appends_and_replays_in_order() {
        let path = std::env::temp_dir().join(format!("rocksoul-journal-{}.jsonl", Uuid::new_v4()));
        let identity = Identity::born_now();
        let mut journal = LifecycleJournal::open(&path, identity.id).unwrap();
        journal.append(Event::now(EventKind::Born)).unwrap();
        journal
            .append(Event::now(EventKind::StatusChanged {
                from: RockSoulStatus::Idle,
                to: RockSoulStatus::Listening,
            }))
            .unwrap();
        let replayed = journal.replay().unwrap();
        assert_eq!(replayed.len(), 2);
        assert_eq!(replayed[0].sequence, 1);
        assert_eq!(replayed[1].sequence, 2);
        assert!(
            replayed
                .iter()
                .all(|record| record.runtime_id == identity.id)
        );
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn journal_reopens_with_next_sequence() {
        let path = std::env::temp_dir().join(format!("rocksoul-journal-{}.jsonl", Uuid::new_v4()));
        let runtime_id = Uuid::new_v4();
        let mut first = LifecycleJournal::open(&path, runtime_id).unwrap();
        first.append(Event::now(EventKind::Born)).unwrap();
        drop(first);
        let mut second = LifecycleJournal::open(&path, runtime_id).unwrap();
        let record = second
            .append(Event::now(EventKind::StatusChanged {
                from: RockSoulStatus::Idle,
                to: RockSoulStatus::Listening,
            }))
            .unwrap();
        assert_eq!(record.sequence, 2);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn cognitive_vertical_slice_is_bounded_and_auditable() {
        use brain::{BrainManifest, DeterministicBrain};
        use cognition::{CognitiveClaim, EpistemicStatus};
        use memory::{MemoryKind, MemoryStore};
        use policy::Guardian;
        use sense::{FixtureSense, SenseConnector};
        use skills::{SkillManifest, execute_skill};
        use workspace::{CognitiveWorkspace, bounded_think};
        use world::WorldGraph;

        let connector = FixtureSense {
            name: "fixture-sense".into(),
            payload: serde_json::json!({"state":"ready"}),
            max_bytes: 256,
            online: true,
        };
        let observation = connector.observe("local-system").unwrap();
        assert_eq!(observation.status, EpistemicStatus::Observed);
        let mut graph = WorldGraph::default();
        graph.observe(&observation);

        let mut memory = MemoryStore::default();
        let claim = CognitiveClaim::new("local-system observed", EpistemicStatus::Observed, 0.5);
        let claim_id = memory.remember(MemoryKind::Episodic, claim).unwrap();
        assert!(memory.validate(claim_id));

        let brain = DeterministicBrain {
            manifest: BrainManifest {
                name: "fallback-nano".into(),
                role: "routing".into(),
                schema_version: 1,
                artifact_sha256: "fixture".into(),
            },
        };
        let mut workspace = CognitiveWorkspace {
            goal: "inspect local-system".into(),
            ..Default::default()
        };
        let mut guardian = Guardian::default();
        bounded_think(&mut workspace, &brain, &mut guardian).unwrap();
        assert_eq!(workspace.confidence, 0);
        let skill = SkillManifest {
            name: "fixture-simulation".into(),
            mode: policy::ActionMode::Simulated,
            side_effects: true,
            reversible: true,
        };
        assert_eq!(
            execute_skill(&skill, &mut guardian, false).unwrap(),
            "SIMULATED"
        );
        assert_eq!(
            graph.nodes["local-system"].discovery,
            world::Discovery::Unknown
        );
    }
}
