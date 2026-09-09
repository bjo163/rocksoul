use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub id: Uuid,
    pub model: String,
    pub model_version: String,
    pub input: serde_json::Value,
    pub requested_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SafetyStatus {
    NotEvaluated,
    Safe,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InferenceProvenance {
    pub provider: String,
    pub model: String,
    pub model_version: String,
    pub input_hash: String,
    pub output_hash: String,
    pub generated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdvisoryInference {
    pub id: Uuid,
    pub output: serde_json::Value,
    pub confidence: f32,
    pub safety: SafetyStatus,
    pub provenance: InferenceProvenance,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum InferenceError {
    #[error("model identity is incomplete")]
    InvalidModel,
    #[error("provider identity is incomplete")]
    InvalidProvider,
    #[error("confidence must be between 0 and 1")]
    InvalidConfidence,
    #[error("inference expiry must be after generation")]
    InvalidExpiry,
    #[error("inference hash is invalid")]
    InvalidHash,
    #[error("inference output is expired")]
    Expired,
}

pub fn payload_hash(value: &serde_json::Value) -> String {
    format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(value).expect("JSON is serializable"))
    )
}

impl InferenceRequest {
    pub fn input_hash(&self) -> String {
        payload_hash(&self.input)
    }
}

impl AdvisoryInference {
    pub fn validate(
        &self,
        request: &InferenceRequest,
        now: DateTime<Utc>,
    ) -> Result<(), InferenceError> {
        if request.model.trim().is_empty() || request.model_version.trim().is_empty() {
            return Err(InferenceError::InvalidModel);
        }
        if self.provenance.provider.trim().is_empty() {
            return Err(InferenceError::InvalidProvider);
        }
        if !(0.0..=1.0).contains(&self.confidence) {
            return Err(InferenceError::InvalidConfidence);
        }
        if self.provenance.expires_at <= self.provenance.generated_at {
            return Err(InferenceError::InvalidExpiry);
        }
        if self.provenance.input_hash != request.input_hash()
            || self.provenance.output_hash != payload_hash(&self.output)
        {
            return Err(InferenceError::InvalidHash);
        }
        if now >= self.provenance.expires_at {
            return Err(InferenceError::Expired);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn request() -> InferenceRequest {
        InferenceRequest {
            id: Uuid::new_v4(),
            model: "minimind".into(),
            model_version: "v1".into(),
            input: serde_json::json!({"task":"classify"}),
            requested_at: Utc::now(),
        }
    }
    fn advisory(request: &InferenceRequest, now: DateTime<Utc>) -> AdvisoryInference {
        let output = serde_json::json!({"label":"advisory"});
        AdvisoryInference {
            id: Uuid::new_v4(),
            output: output.clone(),
            confidence: 0.75,
            safety: SafetyStatus::Safe,
            provenance: InferenceProvenance {
                provider: "fixture".into(),
                model: request.model.clone(),
                model_version: request.model_version.clone(),
                input_hash: request.input_hash(),
                output_hash: payload_hash(&output),
                generated_at: now,
                expires_at: now + chrono::Duration::minutes(5),
            },
        }
    }
    #[test]
    fn advisory_contract_validates_provenance_and_hashes() {
        let now = Utc::now();
        let request = request();
        let result = advisory(&request, now);
        assert!(result.validate(&request, now).is_ok());
        assert_eq!(result.provenance.output_hash.len(), 64);
    }
    #[test]
    fn invalid_hash_and_expiry_are_rejected() {
        let now = Utc::now();
        let request = request();
        let mut result = advisory(&request, now);
        result.provenance.output_hash = "bad".into();
        assert_eq!(
            result.validate(&request, now),
            Err(InferenceError::InvalidHash)
        );
        let mut result = advisory(&request, now);
        result.provenance.expires_at = now;
        assert_eq!(
            result.validate(&request, now),
            Err(InferenceError::InvalidExpiry)
        );
    }
    #[test]
    fn expired_advisory_is_not_usable() {
        let now = Utc::now();
        let request = request();
        let mut result = advisory(&request, now);
        result.provenance.expires_at = now + chrono::Duration::seconds(1);
        assert_eq!(
            result.validate(&request, now + chrono::Duration::seconds(1)),
            Err(InferenceError::Expired)
        );
    }
}
