use rocksoul_core::{
    Identity, RockSoulStatus, brain::BrainManifest, cognition::CognitiveClaim, memory::MemoryStore,
    policy::Guardian, workspace::CognitiveWorkspace, world::WorldGraph,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
use thiserror::Error;

pub mod evaluation;
pub use evaluation::{
    CapabilityScore, EvaluationError, EvaluationEvidence, EvaluationResult, EvaluationRubric,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifeState {
    pub identity: Identity,
    pub cognitive_age: u16,
    pub level: u32,
    pub xp: u64,
    pub trust: u8,
    pub status: RockSoulStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VerifiedOutcome {
    pub verified: bool,
    pub xp: u64,
    pub evaluation_passed: bool,
    pub cognitive_age: Option<u16>,
    pub trust_delta: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgressionReport {
    pub xp_awarded: u64,
    pub old_level: u32,
    pub new_level: u32,
    pub old_age: u16,
    pub new_age: u16,
    pub old_trust: u8,
    pub new_trust: u8,
}

pub struct LifeProgressionEngine;

impl LifeProgressionEngine {
    pub fn apply(state: &mut LifeState, outcome: VerifiedOutcome) -> ProgressionReport {
        let before = (state.level, state.cognitive_age, state.trust);
        if outcome.verified {
            state.grant_validated_xp(outcome.xp, true);
            if let Some(age) = outcome.cognitive_age {
                state.promote_cognitive_age(age, outcome.evaluation_passed);
            }
            state.trust = (i16::from(state.trust) + outcome.trust_delta).clamp(0, 100) as u8;
            state.level = (1 + state.xp / 100).min(u64::from(u32::MAX)) as u32;
        }
        ProgressionReport {
            xp_awarded: if outcome.verified { outcome.xp } else { 0 },
            old_level: before.0,
            new_level: state.level,
            old_age: before.1,
            new_age: state.cognitive_age,
            old_trust: before.2,
            new_trust: state.trust,
        }
    }
}

impl LifeState {
    #[must_use]
    pub fn born_now() -> Self {
        Self {
            identity: Identity::born_now(),
            cognitive_age: 0,
            level: 1,
            xp: 0,
            trust: 0,
            status: RockSoulStatus::Idle,
        }
    }

    pub fn set_status(&mut self, status: RockSoulStatus) {
        self.status = status;
    }

    pub fn grant_validated_xp(&mut self, amount: u64, verified: bool) -> bool {
        if !verified || amount == 0 {
            return false;
        }

        self.xp = self.xp.saturating_add(amount);
        true
    }

    pub fn promote_cognitive_age(&mut self, new_age: u16, evaluation_passed: bool) -> bool {
        if !evaluation_passed || new_age <= self.cognitive_age {
            return false;
        }

        self.cognitive_age = new_age;
        true
    }
}

#[derive(Debug, Error)]
pub enum RuntimeStoreError {
    #[error("runtime state I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("runtime snapshot is invalid: {0}")]
    InvalidSnapshot(#[from] serde_json::Error),
    #[error("unsupported runtime snapshot schema {0}")]
    UnsupportedSchema(u16),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeSnapshot {
    pub schema_version: u16,
    pub life: LifeState,
    pub world: WorldGraph,
    pub memory: MemoryStore,
    pub guardian: Guardian,
    pub workspace: CognitiveWorkspace,
    pub model: Option<BrainManifest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorSnapshot {
    pub schema_version: u16,
    pub runtime_id: uuid::Uuid,
    pub life_status: RockSoulStatus,
    pub cognitive_age: u16,
    pub level: u32,
    pub world_nodes: usize,
    pub memory_records: usize,
    pub policy_decisions: usize,
    pub remaining_steps: u32,
    pub remaining_tools: u32,
    pub model_available: bool,
}

impl RuntimeSnapshot {
    #[must_use]
    pub fn born() -> Self {
        Self {
            schema_version: 1,
            life: LifeState::born_now(),
            world: WorldGraph::default(),
            memory: MemoryStore::default(),
            guardian: Guardian::default(),
            workspace: CognitiveWorkspace::default(),
            model: None,
        }
    }

    #[must_use]
    pub fn operator_snapshot(&self) -> OperatorSnapshot {
        OperatorSnapshot {
            schema_version: self.schema_version,
            runtime_id: self.life.identity.id,
            life_status: self.life.status,
            cognitive_age: self.life.cognitive_age,
            level: self.life.level,
            world_nodes: self.world.nodes.len(),
            memory_records: self.memory.records.len(),
            policy_decisions: self.guardian.decisions.len(),
            remaining_steps: self.guardian.budget.steps,
            remaining_tools: self.guardian.budget.tools,
            model_available: self.model.is_some(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeStore {
    path: PathBuf,
    snapshot: RuntimeSnapshot,
}

impl RuntimeStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, RuntimeStoreError> {
        let path = path.as_ref().to_path_buf();
        let snapshot = if path.exists() {
            serde_json::from_slice(&fs::read(&path)?)?
        } else {
            RuntimeSnapshot::born()
        };
        if snapshot.schema_version != 1 {
            return Err(RuntimeStoreError::UnsupportedSchema(
                snapshot.schema_version,
            ));
        }
        Ok(Self { path, snapshot })
    }
    #[must_use]
    pub fn snapshot(&self) -> &RuntimeSnapshot {
        &self.snapshot
    }
    pub fn persist(&self) -> Result<(), RuntimeStoreError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let tmp = self.path.with_extension("tmp");
        fs::write(&tmp, serde_json::to_vec_pretty(&self.snapshot)?)?;
        fs::rename(tmp, self.path.clone())?;
        Ok(())
    }
    pub fn update_claim(&mut self, claim: CognitiveClaim) {
        let _ = self
            .snapshot
            .memory
            .remember(rocksoul_core::memory::MemoryKind::Episodic, claim);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn birth_state_is_level_one_without_experience() {
        let life = LifeState::born_now();

        assert_eq!(life.cognitive_age, 0);
        assert_eq!(life.level, 1);
        assert_eq!(life.xp, 0);
        assert_eq!(life.trust, 0);
        assert_eq!(life.status, RockSoulStatus::Idle);
    }

    #[test]
    fn unverified_work_does_not_earn_xp() {
        let mut life = LifeState::born_now();

        assert!(!life.grant_validated_xp(100, false));
        assert_eq!(life.xp, 0);
    }

    #[test]
    fn cognitive_age_requires_evaluation_and_forward_progress() {
        let mut life = LifeState::born_now();

        assert!(!life.promote_cognitive_age(5, false));
        assert!(life.promote_cognitive_age(5, true));
        assert!(!life.promote_cognitive_age(4, true));
        assert_eq!(life.cognitive_age, 5);
    }

    #[test]
    fn runtime_store_round_trips_identity_and_cognitive_state() {
        let path =
            std::env::temp_dir().join(format!("rocksoul-state-{}.json", uuid::Uuid::new_v4()));
        let mut store = RuntimeStore::open(&path).unwrap();
        let id = store.snapshot().life.identity.id;
        store.snapshot.life.set_status(RockSoulStatus::Listening);
        store.persist().unwrap();
        let restored = RuntimeStore::open(&path).unwrap();
        assert_eq!(restored.snapshot().life.identity.id, id);
        assert_eq!(restored.snapshot().life.status, RockSoulStatus::Listening);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn operator_snapshot_exposes_bounded_observability() {
        let snapshot = RuntimeSnapshot::born();
        let operator = snapshot.operator_snapshot();
        assert_eq!(operator.schema_version, 1);
        assert_eq!(operator.life_status, RockSoulStatus::Idle);
        assert_eq!(operator.remaining_steps, 32);
        assert!(!operator.model_available);
    }

    #[test]
    fn verified_outcome_drives_xp_level_age_and_trust() {
        let mut life = LifeState::born_now();
        let report = LifeProgressionEngine::apply(
            &mut life,
            VerifiedOutcome {
                verified: true,
                xp: 250,
                evaluation_passed: true,
                cognitive_age: Some(2),
                trust_delta: 10,
            },
        );
        assert_eq!(
            (report.new_level, report.new_age, report.new_trust),
            (3, 2, 10)
        );
        LifeProgressionEngine::apply(
            &mut life,
            VerifiedOutcome {
                verified: false,
                xp: 1000,
                evaluation_passed: true,
                cognitive_age: Some(9),
                trust_delta: 50,
            },
        );
        assert_eq!((life.xp, life.cognitive_age, life.trust), (250, 2, 10));
    }
}
