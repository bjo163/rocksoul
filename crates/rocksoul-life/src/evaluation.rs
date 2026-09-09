use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// A versioned rubric keeps progression decisions reproducible across releases.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluationRubric {
    pub version: u16,
    pub pass_score: f32,
    pub max_age: Duration,
    pub required_capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilityScore {
    pub capability: String,
    pub score: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluationEvidence {
    pub id: Uuid,
    pub rubric_version: u16,
    pub subject: String,
    pub scores: Vec<CapabilityScore>,
    pub observed_at: DateTime<Utc>,
    pub source: String,
    pub verified: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluationResult {
    pub evidence_id: Uuid,
    pub rubric_version: u16,
    pub score: f32,
    pub passed: bool,
    pub fresh: bool,
}

#[derive(Debug, Error, PartialEq)]
pub enum EvaluationError {
    #[error("evaluation rubric version must be non-zero")]
    InvalidRubric,
    #[error("evaluation score must be between 0 and 1")]
    InvalidScore,
    #[error("evaluation has no scores")]
    EmptyScores,
    #[error("required capability is missing")]
    MissingCapability,
    #[error("evaluation evidence is not verified")]
    Unverified,
    #[error("evaluation evidence has no source")]
    MissingSource,
    #[error("evaluation evidence is stale")]
    Stale,
}

impl EvaluationRubric {
    pub fn evaluate(
        &self,
        evidence: &EvaluationEvidence,
        now: DateTime<Utc>,
    ) -> Result<EvaluationResult, EvaluationError> {
        if self.version == 0
            || !(0.0..=1.0).contains(&self.pass_score)
            || self.max_age < Duration::zero()
        {
            return Err(EvaluationError::InvalidRubric);
        }
        if evidence.rubric_version != self.version {
            return Err(EvaluationError::InvalidRubric);
        }
        if !evidence.verified {
            return Err(EvaluationError::Unverified);
        }
        if evidence.source.trim().is_empty() {
            return Err(EvaluationError::MissingSource);
        }
        let age = now.signed_duration_since(evidence.observed_at);
        if age < Duration::zero() || age > self.max_age {
            return Err(EvaluationError::Stale);
        }
        if evidence.scores.is_empty() {
            return Err(EvaluationError::EmptyScores);
        }
        for score in &evidence.scores {
            if !(0.0..=1.0).contains(&score.score) {
                return Err(EvaluationError::InvalidScore);
            }
        }
        if self.required_capabilities.iter().any(|required| {
            !evidence
                .scores
                .iter()
                .any(|score| &score.capability == required)
        }) {
            return Err(EvaluationError::MissingCapability);
        }
        let score = evidence.scores.iter().map(|item| item.score).sum::<f32>()
            / evidence.scores.len() as f32;
        Ok(EvaluationResult {
            evidence_id: evidence.id,
            rubric_version: self.version,
            score,
            passed: score >= self.pass_score,
            fresh: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evidence(verified: bool, observed_at: DateTime<Utc>) -> EvaluationEvidence {
        EvaluationEvidence {
            id: Uuid::new_v4(),
            rubric_version: 1,
            subject: "runtime".into(),
            scores: vec![
                CapabilityScore {
                    capability: "memory".into(),
                    score: 0.9,
                },
                CapabilityScore {
                    capability: "policy".into(),
                    score: 0.7,
                },
            ],
            observed_at,
            source: "evaluation-fixture".into(),
            verified,
        }
    }

    #[test]
    fn versioned_rubric_produces_deterministic_pass_result() {
        let now = Utc::now();
        let rubric = EvaluationRubric {
            version: 1,
            pass_score: 0.8,
            max_age: Duration::hours(1),
            required_capabilities: vec!["memory".into(), "policy".into()],
        };
        let result = rubric.evaluate(&evidence(true, now), now).unwrap();
        assert_eq!(result.rubric_version, 1);
        assert!(!result.passed);
        assert!((result.score - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn stale_or_unverified_evidence_cannot_pass() {
        let now = Utc::now();
        let rubric = EvaluationRubric {
            version: 1,
            pass_score: 0.5,
            max_age: Duration::minutes(5),
            required_capabilities: vec![],
        };
        assert_eq!(
            rubric.evaluate(&evidence(false, now), now),
            Err(EvaluationError::Unverified)
        );
        assert_eq!(
            rubric.evaluate(&evidence(true, now - Duration::hours(1)), now),
            Err(EvaluationError::Stale)
        );
    }

    #[test]
    fn rubric_rejects_missing_required_capability_and_regression() {
        let now = Utc::now();
        let rubric = EvaluationRubric {
            version: 2,
            pass_score: 0.5,
            max_age: Duration::hours(1),
            required_capabilities: vec!["memory".into()],
        };
        let mut item = evidence(true, now);
        item.rubric_version = 1;
        assert_eq!(
            rubric.evaluate(&item, now),
            Err(EvaluationError::InvalidRubric)
        );
        item.rubric_version = 2;
        item.scores.retain(|score| score.capability == "policy");
        assert_eq!(
            rubric.evaluate(&item, now),
            Err(EvaluationError::MissingCapability)
        );
    }
}
