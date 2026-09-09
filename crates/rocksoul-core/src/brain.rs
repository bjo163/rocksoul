use serde::{Deserialize, Serialize};
use thiserror::Error;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrainManifest {
    pub name: String,
    pub role: String,
    pub schema_version: u16,
    pub artifact_sha256: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrainRequest {
    pub input: String,
    pub max_tokens: u32,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrainResponse {
    pub proposal: String,
    pub confidence: u8,
    pub model: String,
}
pub trait Brain {
    fn manifest(&self) -> &BrainManifest;
    fn infer(&self, request: &BrainRequest) -> BrainResponse;
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum BrainValidationError {
    #[error("unsupported brain schema {0}")]
    UnsupportedSchema(u16),
    #[error("brain artifact hash is missing")]
    MissingHash,
}

pub fn validate_manifest(manifest: &BrainManifest) -> Result<(), BrainValidationError> {
    if manifest.schema_version != 1 {
        return Err(BrainValidationError::UnsupportedSchema(
            manifest.schema_version,
        ));
    }
    if manifest.artifact_sha256.trim().is_empty() {
        return Err(BrainValidationError::MissingHash);
    }
    Ok(())
}
#[derive(Debug, Clone)]
pub struct DeterministicBrain {
    pub manifest: BrainManifest,
}
impl Brain for DeterministicBrain {
    fn manifest(&self) -> &BrainManifest {
        &self.manifest
    }
    fn infer(&self, request: &BrainRequest) -> BrainResponse {
        BrainResponse {
            proposal: format!("observe: {}", request.input),
            confidence: 0,
            model: self.manifest.name.clone(),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fallback_brain_is_advisory() {
        let b = DeterministicBrain {
            manifest: BrainManifest {
                name: "fallback".into(),
                role: "nano".into(),
                schema_version: 1,
                artifact_sha256: "fixture".into(),
            },
        };
        assert_eq!(
            b.infer(&BrainRequest {
                input: "x".into(),
                max_tokens: 8
            })
            .confidence,
            0
        );
    }

    #[test]
    fn brain_manifest_requires_compatible_schema_and_hash() {
        let manifest = BrainManifest {
            name: "nano".into(),
            role: "router".into(),
            schema_version: 1,
            artifact_sha256: "fixture".into(),
        };
        assert!(validate_manifest(&manifest).is_ok());
        assert_eq!(
            validate_manifest(&BrainManifest {
                schema_version: 2,
                ..manifest.clone()
            }),
            Err(BrainValidationError::UnsupportedSchema(2))
        );
        assert_eq!(
            validate_manifest(&BrainManifest {
                artifact_sha256: String::new(),
                ..manifest
            }),
            Err(BrainValidationError::MissingHash)
        );
    }
}
