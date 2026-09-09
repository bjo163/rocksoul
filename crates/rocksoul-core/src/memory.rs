use crate::cognition::{CognitiveClaim, EpistemicStatus, Provenance};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{fs, io, path::Path};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryKind {
    Working,
    Episodic,
    Semantic,
    Procedural,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryRecord {
    pub id: Uuid,
    pub kind: MemoryKind,
    pub claim: CognitiveClaim,
    pub provenance: Vec<Provenance>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub supersedes: Option<Uuid>,
    pub validated: bool,
    pub redacted: bool,
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
        let provenance = claim.provenance.clone();
        let id = claim.id;
        self.records.push(MemoryRecord {
            id,
            kind,
            claim,
            provenance,
            created_at: Utc::now(),
            expires_at: None,
            supersedes: None,
            validated: false,
            redacted: false,
        });
        Some(id)
    }
    pub fn remember_until(
        &mut self,
        kind: MemoryKind,
        claim: CognitiveClaim,
        expires_at: DateTime<Utc>,
        supersedes: Option<Uuid>,
    ) -> Option<Uuid> {
        let id = self.remember(kind, claim)?;
        let record = self
            .records
            .iter_mut()
            .find(|r| r.id == id)
            .expect("just inserted");
        record.expires_at = Some(expires_at);
        record.supersedes = supersedes;
        Some(id)
    }
    pub fn validate(&mut self, id: Uuid) -> bool {
        self.records
            .iter_mut()
            .find(|r| r.id == id)
            .map(|r| {
                r.validated = true;
                true
            })
            .unwrap_or(false)
    }
    pub fn redact(&mut self, id: Uuid) -> bool {
        self.records
            .iter_mut()
            .find(|r| r.id == id)
            .map(|r| {
                r.redacted = true;
                true
            })
            .unwrap_or(false)
    }
    pub fn retain_fresh(&mut self, now: DateTime<Utc>) {
        self.records
            .retain(|r| r.expires_at.is_none_or(|expiry| expiry > now));
    }
    pub fn recall(&self, query: &str) -> Vec<MemoryRecord> {
        let now = Utc::now();
        let mut results: Vec<_> = self
            .records
            .iter()
            .filter(|r| {
                r.expires_at.is_none_or(|expiry| expiry > now) && r.claim.statement.contains(query)
            })
            .cloned()
            .collect();
        results.sort_by(|a, b| {
            b.claim
                .confidence
                .partial_cmp(&a.claim.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.id.cmp(&b.id))
        });
        results.iter_mut().for_each(|r| {
            if r.redacted {
                r.claim.statement = "[REDACTED]".into();
            }
        });
        results
    }
    pub fn save(&self, path: &Path) -> io::Result<()> {
        let bytes = serde_json::to_vec_pretty(self).map_err(io::Error::other)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let temp = path.with_extension("tmp");
        fs::write(&temp, bytes)?;
        fs::rename(temp, path)
    }
    pub fn load(path: &Path) -> io::Result<Self> {
        serde_json::from_slice(&fs::read(path)?).map_err(io::Error::other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cognition::CognitiveClaim;
    use chrono::Duration;
    use std::time::{SystemTime, UNIX_EPOCH};
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
    #[test]
    fn four_kinds_persist_and_recall_deterministically() {
        let root = std::env::temp_dir().join(format!(
            "rocksoul-memory-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let path = root.join("memory.json");
        let mut s = MemoryStore::default();
        for kind in [
            MemoryKind::Working,
            MemoryKind::Episodic,
            MemoryKind::Semantic,
            MemoryKind::Procedural,
        ] {
            s.remember(
                kind,
                CognitiveClaim::new("same", EpistemicStatus::Observed, 0.8),
            );
        }
        s.save(&path).unwrap();
        let loaded = MemoryStore::load(&path).unwrap();
        assert_eq!(loaded.records.len(), 4);
        assert_eq!(loaded.recall("same").len(), 4);
        let _ = fs::remove_dir_all(root);
    }
    #[test]
    fn expiry_supersession_and_redaction_are_explicit() {
        let mut s = MemoryStore::default();
        let old = s
            .remember(
                MemoryKind::Semantic,
                CognitiveClaim::new("secret", EpistemicStatus::Observed, 0.5),
            )
            .unwrap();
        let id = s
            .remember_until(
                MemoryKind::Semantic,
                CognitiveClaim::new("new", EpistemicStatus::Verified, 0.9),
                Utc::now() + Duration::hours(1),
                Some(old),
            )
            .unwrap();
        assert_eq!(
            s.records.iter().find(|r| r.id == id).unwrap().supersedes,
            Some(old)
        );
        assert!(s.redact(id));
        assert_eq!(s.recall("new")[0].claim.statement, "[REDACTED]");
        s.retain_fresh(Utc::now() + Duration::hours(2));
        assert!(s.recall("new").is_empty());
    }
}
