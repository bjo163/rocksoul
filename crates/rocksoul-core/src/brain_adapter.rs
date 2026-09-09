use crate::brain::{
    Brain, BrainManifest, BrainRequest, BrainResponse, BrainValidationError, DeterministicBrain,
    validate_manifest,
};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct NanoMiniMindAdapter {
    manifest: BrainManifest,
    artifact: Option<Vec<u8>>,
    max_tokens: u32,
    fallback: DeterministicBrain,
}

impl NanoMiniMindAdapter {
    pub fn from_artifact(
        manifest: BrainManifest,
        artifact: Vec<u8>,
        max_tokens: u32,
    ) -> Result<Self, BrainValidationError> {
        validate_manifest(&manifest)?;
        if format!("{:x}", Sha256::digest(&artifact)) != manifest.artifact_sha256 {
            return Err(BrainValidationError::MissingHash);
        }
        let fallback = DeterministicBrain {
            manifest: BrainManifest {
                name: "deterministic-fallback".into(),
                role: manifest.role.clone(),
                schema_version: 1,
                artifact_sha256: "fallback".into(),
            },
        };
        Ok(Self {
            manifest,
            artifact: Some(artifact),
            max_tokens,
            fallback,
        })
    }

    pub fn unavailable(
        manifest: BrainManifest,
        max_tokens: u32,
    ) -> Result<Self, BrainValidationError> {
        validate_manifest(&manifest)?;
        let fallback = DeterministicBrain {
            manifest: BrainManifest {
                name: "deterministic-fallback".into(),
                role: manifest.role.clone(),
                schema_version: 1,
                artifact_sha256: "fallback".into(),
            },
        };
        Ok(Self {
            manifest,
            artifact: None,
            max_tokens,
            fallback,
        })
    }

    pub fn artifact_loaded(&self) -> bool {
        self.artifact.is_some()
    }
}

impl Brain for NanoMiniMindAdapter {
    fn manifest(&self) -> &BrainManifest {
        &self.manifest
    }
    fn infer(&self, request: &BrainRequest) -> BrainResponse {
        if self.artifact.is_none()
            || request.max_tokens == 0
            || request.max_tokens > self.max_tokens
        {
            return self.fallback.infer(request);
        }
        BrainResponse {
            proposal: format!("advisory:{}", request.input),
            confidence: 0,
            model: self.manifest.name.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brain::Brain;
    fn manifest(bytes: &[u8]) -> BrainManifest {
        BrainManifest {
            name: "minimind-fixture".into(),
            role: "advisory".into(),
            schema_version: 1,
            artifact_sha256: format!("{:x}", Sha256::digest(bytes)),
        }
    }
    #[test]
    fn adapter_validates_artifact_and_uses_loaded_model_advisory() {
        let bytes = b"fixture-model".to_vec();
        let adapter = NanoMiniMindAdapter::from_artifact(manifest(&bytes), bytes, 32).unwrap();
        let result = adapter.infer(&BrainRequest {
            input: "hello".into(),
            max_tokens: 8,
        });
        assert!(adapter.artifact_loaded());
        assert_eq!(result.model, "minimind-fixture");
        assert_eq!(result.confidence, 0);
    }
    #[test]
    fn incompatible_hash_is_rejected() {
        let mut m = manifest(b"expected");
        m.artifact_sha256 = "wrong".into();
        assert_eq!(
            NanoMiniMindAdapter::from_artifact(m, b"actual".to_vec(), 8).unwrap_err(),
            BrainValidationError::MissingHash
        );
    }
    #[test]
    fn unavailable_or_over_budget_uses_deterministic_fallback() {
        let m = manifest(b"unused");
        let adapter = NanoMiniMindAdapter::unavailable(m, 8).unwrap();
        let result = adapter.infer(&BrainRequest {
            input: "hello".into(),
            max_tokens: 99,
        });
        assert_eq!(result.model, "deterministic-fallback");
        assert_eq!(result.confidence, 0);
    }
}
