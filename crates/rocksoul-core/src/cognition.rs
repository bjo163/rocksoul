use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EpistemicStatus {
    Unknown,
    Observed,
    Inferred,
    Verified,
    Trusted,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Provenance {
    pub source: String,
    pub observed_at: DateTime<Utc>,
    pub scope: String,
    pub payload_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Observation {
    pub id: Uuid,
    pub connector: String,
    pub subject: String,
    pub payload: serde_json::Value,
    pub provenance: Provenance,
    pub status: EpistemicStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CognitiveClaim {
    pub id: Uuid,
    pub statement: String,
    pub status: EpistemicStatus,
    pub confidence: f32,
    pub evidence: Vec<Uuid>,
    pub provenance: Vec<Provenance>,
}

impl CognitiveClaim {
    pub fn new(statement: impl Into<String>, status: EpistemicStatus, confidence: f32) -> Self {
        Self {
            id: Uuid::new_v4(),
            statement: statement.into(),
            status,
            confidence: confidence.clamp(0.0, 1.0),
            evidence: Vec::new(),
            provenance: Vec::new(),
        }
    }
    pub fn can_be_trusted(&self) -> bool {
        matches!(self.status, EpistemicStatus::Trusted) && !self.evidence.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaimTransitionError {
    MissingEvidence,
    InvalidPromotion,
    InvalidConfidence,
}

pub fn promote_claim(
    claim: &mut CognitiveClaim,
    target: EpistemicStatus,
    evidence: &[Uuid],
) -> Result<(), ClaimTransitionError> {
    if !(0.0..=1.0).contains(&claim.confidence) {
        return Err(ClaimTransitionError::InvalidConfidence);
    }
    let allowed = matches!(
        (claim.status, target),
        (EpistemicStatus::Unknown, EpistemicStatus::Observed)
            | (EpistemicStatus::Observed, EpistemicStatus::Inferred)
            | (EpistemicStatus::Inferred, EpistemicStatus::Verified)
            | (EpistemicStatus::Verified, EpistemicStatus::Trusted)
    );
    if !allowed {
        return Err(ClaimTransitionError::InvalidPromotion);
    }
    if matches!(target, EpistemicStatus::Verified | EpistemicStatus::Trusted) && evidence.is_empty()
    {
        return Err(ClaimTransitionError::MissingEvidence);
    }
    claim.status = target;
    claim.evidence.extend(evidence.iter().copied());
    claim.evidence.sort_unstable();
    claim.evidence.dedup();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn consequential_promotion_requires_evidence() {
        let mut claim = CognitiveClaim::new("a", EpistemicStatus::Inferred, 0.8);
        assert_eq!(
            promote_claim(&mut claim, EpistemicStatus::Verified, &[]),
            Err(ClaimTransitionError::MissingEvidence)
        );
        assert_eq!(claim.status, EpistemicStatus::Inferred);
    }
    #[test]
    fn claim_progression_is_explicit_and_deduplicated() {
        let evidence = Uuid::new_v4();
        let mut claim = CognitiveClaim::new("a", EpistemicStatus::Observed, 0.8);
        promote_claim(&mut claim, EpistemicStatus::Inferred, &[]).unwrap();
        promote_claim(&mut claim, EpistemicStatus::Verified, &[evidence, evidence]).unwrap();
        assert_eq!(claim.evidence, vec![evidence]);
    }
}
