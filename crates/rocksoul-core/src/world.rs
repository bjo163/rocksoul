use crate::cognition::{EpistemicStatus, Observation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Discovery {
    Unknown,
    Discovered,
    Observed,
    Visited,
    Studied,
    Verified,
    Trusted,
}
impl Discovery {
    pub fn can_promote(self, next: Self) -> bool {
        (self as u8 + 1 == next as u8) && !matches!(next, Self::Verified | Self::Trusted)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldNode {
    pub id: String,
    pub label: String,
    pub discovery: Discovery,
    pub evidence: Vec<Uuid>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldConflict {
    pub id: Uuid,
    pub subject: String,
    pub assertions: Vec<String>,
    pub evidence: Vec<Uuid>,
    pub resolved: bool,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorldGraph {
    pub nodes: BTreeMap<String, WorldNode>,
    pub edges: Vec<(String, String, String)>,
    #[serde(default)]
    pub assertions: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub conflicts: Vec<WorldConflict>,
}
impl WorldGraph {
    pub fn observe(&mut self, observation: &Observation) {
        let assertion = format!(
            "{:x}",
            Sha256::digest(serde_json::to_vec(&observation.payload).expect("JSON is serializable"))
        );
        let assertions = self
            .assertions
            .entry(observation.subject.clone())
            .or_default();
        if !assertions.contains(&assertion) {
            if !assertions.is_empty() {
                self.conflicts.push(WorldConflict {
                    id: Uuid::new_v4(),
                    subject: observation.subject.clone(),
                    assertions: assertions.clone(),
                    evidence: Vec::new(),
                    resolved: false,
                });
            }
            assertions.push(assertion);
        }
        self.nodes
            .entry(observation.subject.clone())
            .or_insert_with(|| WorldNode {
                id: observation.subject.clone(),
                label: observation.subject.clone(),
                discovery: Discovery::Unknown,
                evidence: Vec::new(),
            })
            .evidence
            .push(observation.id);
    }
    pub fn unresolved_conflicts(&self) -> impl Iterator<Item = &WorldConflict> {
        self.conflicts.iter().filter(|conflict| !conflict.resolved)
    }
    pub fn resolve_conflict(&mut self, id: Uuid, review_evidence: Uuid) -> bool {
        let Some(conflict) = self
            .conflicts
            .iter_mut()
            .find(|conflict| conflict.id == id && !conflict.resolved)
        else {
            return false;
        };
        conflict.evidence.push(review_evidence);
        conflict.resolved = true;
        true
    }
    pub fn promote(&mut self, id: &str, target: Discovery, evidence: Uuid) -> bool {
        let Some(node) = self.nodes.get_mut(id) else {
            return false;
        };
        let valid = match target {
            Discovery::Verified | Discovery::Trusted => {
                matches!(node.discovery, Discovery::Studied | Discovery::Verified)
                    && !matches!(target, Discovery::Trusted)
                    || (node.discovery == Discovery::Verified && target == Discovery::Trusted)
            }
            _ => node.discovery.can_promote(target),
        };
        if !valid {
            return false;
        }
        node.discovery = target;
        node.evidence.push(evidence);
        true
    }
}
pub fn status_is_non_authoritative(status: EpistemicStatus) -> bool {
    !matches!(status, EpistemicStatus::Trusted)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::cognition::{Observation, Provenance};
    use chrono::Utc;
    #[test]
    fn observation_creates_unknown_node() {
        let o = Observation {
            id: Uuid::new_v4(),
            connector: "fixture".into(),
            subject: "local".into(),
            payload: serde_json::json!({}),
            provenance: Provenance {
                source: "test".into(),
                observed_at: Utc::now(),
                scope: "test".into(),
                payload_hash: None,
            },
            status: EpistemicStatus::Observed,
        };
        let mut g = WorldGraph::default();
        g.observe(&o);
        assert_eq!(g.nodes["local"].discovery, Discovery::Unknown);
    }
    #[test]
    fn conflicting_observations_are_retained_until_review() {
        let make = |payload| Observation {
            id: Uuid::new_v4(),
            connector: "fixture".into(),
            subject: "same".into(),
            payload,
            provenance: Provenance {
                source: "test".into(),
                observed_at: Utc::now(),
                scope: "test".into(),
                payload_hash: None,
            },
            status: EpistemicStatus::Observed,
        };
        let mut g = WorldGraph::default();
        g.observe(&make(serde_json::json!({"value":1})));
        g.observe(&make(serde_json::json!({"value":2})));
        let id = g.unresolved_conflicts().next().unwrap().id;
        assert!(!g.unresolved_conflicts().next().unwrap().resolved);
        assert!(g.resolve_conflict(id, Uuid::new_v4()));
        assert_eq!(g.unresolved_conflicts().count(), 0);
    }
}
