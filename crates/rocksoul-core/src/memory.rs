use crate::cognition::{CognitiveClaim, EpistemicStatus};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryKind {
    Episodic,
    Semantic,
    Procedural,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryRecord {
    pub id: Uuid,
    pub kind: MemoryKind,
    pub claim: CognitiveClaim,
    pub validated: bool,
}
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MemoryStore {
    pub records: Vec<MemoryRecord>,
}
impl MemoryStore {
    pub fn remember(&mut self, kind: MemoryKind, claim: CognitiveClaim) -> Option<Uuid> {
        if claim.status == EpistemicStatus::Unknown {
            return None;
        }
        let id = claim.id;
        self.records.push(MemoryRecord {
            id,
            kind,
            claim,
            validated: false,
        });
        Some(id)
    }
    pub fn validate(&mut self, id: Uuid) -> bool {
        if let Some(r) = self.records.iter_mut().find(|r| r.id == id) {
            r.validated = true;
            return true;
        }
        false
    }
    pub fn recall(&self, query: &str) -> Vec<&MemoryRecord> {
        self.records
            .iter()
            .filter(|r| r.claim.statement.contains(query))
            .collect()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::cognition::CognitiveClaim;
    #[test]
    fn unknown_is_not_remembered() {
        let mut s = MemoryStore::default();
        assert!(
            s.remember(
                MemoryKind::Semantic,
                CognitiveClaim::new("x", EpistemicStatus::Unknown, 0.1)
            )
            .is_none()
        );
    }
}
