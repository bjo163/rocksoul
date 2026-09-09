use crate::cognition::{EpistemicStatus, Observation};
use serde::{Deserialize, Serialize};
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorldGraph {
    pub nodes: BTreeMap<String, WorldNode>,
    pub edges: Vec<(String, String, String)>,
}
impl WorldGraph {
    pub fn observe(&mut self, observation: &Observation) {
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
}
