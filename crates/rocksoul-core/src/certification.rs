use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateResult {
    pub name: String,
    pub passed: bool,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertificationReport {
    pub version: u16,
    pub commit: String,
    pub artifact_sha256: String,
    pub binary_identity: String,
    pub gates: Vec<GateResult>,
    pub rollback_plan: String,
    pub operator_health: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CertificationError {
    #[error("certification schema is unsupported")]
    UnsupportedSchema,
    #[error("certification identity or rollback evidence is missing")]
    MissingEvidence,
    #[error("one or more certification gates failed")]
    GateFailed,
}

impl CertificationReport {
    pub fn verify(&self) -> Result<(), CertificationError> {
        if self.version != 1 {
            return Err(CertificationError::UnsupportedSchema);
        }
        if self.commit.trim().is_empty()
            || self.artifact_sha256.trim().is_empty()
            || self.binary_identity.trim().is_empty()
            || self.rollback_plan.trim().is_empty()
            || self.operator_health.trim().is_empty()
        {
            return Err(CertificationError::MissingEvidence);
        }
        if self.gates.is_empty()
            || self
                .gates
                .iter()
                .any(|gate| !gate.passed || gate.evidence.trim().is_empty())
        {
            return Err(CertificationError::GateFailed);
        }
        Ok(())
    }
    pub fn required_gate_names() -> [&'static str; 8] {
        [
            "provenance",
            "policy",
            "dependencies",
            "tests",
            "audit",
            "resource-bounds",
            "rollback",
            "operator-health",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn report() -> CertificationReport {
        CertificationReport {
            version: 1,
            commit: "abc123".into(),
            artifact_sha256: "hash".into(),
            binary_identity: "rocksoul 0.1.0".into(),
            gates: CertificationReport::required_gate_names()
                .into_iter()
                .map(|name| GateResult {
                    name: name.into(),
                    passed: true,
                    evidence: "verified".into(),
                })
                .collect(),
            rollback_plan: "restore previous signed artifact".into(),
            operator_health: "healthy".into(),
        }
    }
    #[test]
    fn complete_report_is_certifiable() {
        assert!(report().verify().is_ok());
        assert_eq!(report().gates.len(), 8);
    }
    #[test]
    fn failed_gate_or_missing_identity_fails_closed() {
        let mut failed = report();
        failed.gates[3].passed = false;
        assert_eq!(failed.verify(), Err(CertificationError::GateFailed));
        let mut missing = report();
        missing.rollback_plan.clear();
        assert_eq!(missing.verify(), Err(CertificationError::MissingEvidence));
    }
    #[test]
    fn unsupported_schema_fails_closed() {
        let mut unsupported = report();
        unsupported.version = 2;
        assert_eq!(
            unsupported.verify(),
            Err(CertificationError::UnsupportedSchema)
        );
    }
}
