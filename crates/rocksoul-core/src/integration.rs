//! Provider-neutral boundary for embedding cognitive services in a host runtime.
//!
//! The host owns lifecycle, persistence, and side effects. This contract exposes
//! bounded advisory cognition and health metadata without granting canonical
//! research writes or action authority.

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const CONTRACT_SCHEMA_VERSION: u16 = 1;

const SMOKE_STAGES: [&str; 6] = [
    "sense",
    "advisory-inference",
    "policy",
    "evaluation",
    "event-replay",
    "snapshot",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CognitiveRuntimeHealth {
    pub schema_version: u16,
    pub status: HealthStatus,
    pub advisory_only: bool,
    pub canonical_write_authority: bool,
    pub max_input_bytes: usize,
    pub max_output_bytes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Ready,
    Degraded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CognitiveRequest {
    pub schema_version: u16,
    pub input: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdvisoryResponse {
    pub schema_version: u16,
    pub output: String,
    pub advisory_only: bool,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum IntegrationError {
    #[error("unsupported integration schema version {0}")]
    UnsupportedSchema(u16),
    #[error("request exceeds the {limit}-byte input bound")]
    InputTooLarge { limit: usize },
    #[error("response exceeds the {limit}-byte output bound")]
    OutputTooLarge { limit: usize },
    #[error("canonical write authority is not allowed at the cognitive boundary")]
    CanonicalWriteAuthority,
    #[error("resource usage exceeds the configured budget")]
    ResourceBudgetExceeded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceBudget {
    pub max_input_bytes: usize,
    pub max_output_bytes: usize,
    pub max_events: usize,
    pub max_replay_records: usize,
}

impl ResourceBudget {
    #[must_use]
    pub const fn bounded() -> Self {
        Self {
            max_input_bytes: 16 * 1024,
            max_output_bytes: 16 * 1024,
            max_events: 1024,
            max_replay_records: 4096,
        }
    }

    #[must_use]
    pub const fn allows(self, input: usize, output: usize, events: usize, replay: usize) -> bool {
        input <= self.max_input_bytes
            && output <= self.max_output_bytes
            && events <= self.max_events
            && replay <= self.max_replay_records
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CognitiveBoundary {
    pub max_input_bytes: usize,
    pub max_output_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CognitiveSmokeReport {
    pub schema_version: u16,
    pub stages: Vec<String>,
    pub side_effects: bool,
    pub canonical_writes: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperatorHealthSnapshot {
    pub schema_version: u16,
    pub status: HealthStatus,
    pub smoke_successful: bool,
    pub certification_verified: bool,
    pub resource_budget: ResourceBudget,
    pub canonical_write_authority: bool,
}

impl OperatorHealthSnapshot {
    #[must_use]
    pub fn from_boundary(
        boundary: CognitiveBoundary,
        smoke: &CognitiveSmokeReport,
        certification_verified: bool,
    ) -> Self {
        Self {
            schema_version: CONTRACT_SCHEMA_VERSION,
            status: if smoke.is_successful() && certification_verified {
                HealthStatus::Ready
            } else {
                HealthStatus::Degraded
            },
            smoke_successful: smoke.is_successful(),
            certification_verified,
            resource_budget: ResourceBudget {
                max_input_bytes: boundary.max_input_bytes,
                max_output_bytes: boundary.max_output_bytes,
                ..ResourceBudget::bounded()
            },
            canonical_write_authority: false,
        }
    }
}

impl CognitiveSmokeReport {
    #[must_use]
    pub fn complete() -> Self {
        Self {
            schema_version: CONTRACT_SCHEMA_VERSION,
            stages: SMOKE_STAGES
                .iter()
                .map(|stage| (*stage).to_owned())
                .collect(),
            side_effects: false,
            canonical_writes: false,
        }
    }

    #[must_use]
    pub fn is_successful(&self) -> bool {
        self.schema_version == CONTRACT_SCHEMA_VERSION
            && self.stages == SMOKE_STAGES
            && !self.side_effects
            && !self.canonical_writes
    }
}

impl Default for CognitiveBoundary {
    fn default() -> Self {
        Self {
            max_input_bytes: 16 * 1024,
            max_output_bytes: 16 * 1024,
        }
    }
}

impl CognitiveBoundary {
    pub fn health(self) -> CognitiveRuntimeHealth {
        CognitiveRuntimeHealth {
            schema_version: CONTRACT_SCHEMA_VERSION,
            status: HealthStatus::Ready,
            advisory_only: true,
            canonical_write_authority: false,
            max_input_bytes: self.max_input_bytes,
            max_output_bytes: self.max_output_bytes,
        }
    }

    pub fn validate_request(self, request: &CognitiveRequest) -> Result<(), IntegrationError> {
        if request.schema_version != CONTRACT_SCHEMA_VERSION {
            return Err(IntegrationError::UnsupportedSchema(request.schema_version));
        }
        if request.input.len() > self.max_input_bytes {
            return Err(IntegrationError::InputTooLarge {
                limit: self.max_input_bytes,
            });
        }
        Ok(())
    }

    pub fn advisory_response(self, output: String) -> Result<AdvisoryResponse, IntegrationError> {
        if output.len() > self.max_output_bytes {
            return Err(IntegrationError::OutputTooLarge {
                limit: self.max_output_bytes,
            });
        }
        Ok(AdvisoryResponse {
            schema_version: CONTRACT_SCHEMA_VERSION,
            output,
            advisory_only: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_is_advisory_and_has_no_canonical_write_authority() {
        let health = CognitiveBoundary::default().health();
        assert_eq!(health.status, HealthStatus::Ready);
        assert!(health.advisory_only);
        assert!(!health.canonical_write_authority);
    }

    #[test]
    fn boundary_rejects_unknown_schema_and_oversized_input() {
        let boundary = CognitiveBoundary {
            max_input_bytes: 4,
            ..Default::default()
        };
        assert_eq!(
            boundary.validate_request(&CognitiveRequest {
                schema_version: 99,
                input: "ok".into(),
            }),
            Err(IntegrationError::UnsupportedSchema(99))
        );
        assert_eq!(
            boundary.validate_request(&CognitiveRequest {
                schema_version: CONTRACT_SCHEMA_VERSION,
                input: "large".into(),
            }),
            Err(IntegrationError::InputTooLarge { limit: 4 })
        );
    }

    #[test]
    fn response_is_bounded_and_explicitly_advisory() {
        let boundary = CognitiveBoundary {
            max_output_bytes: 3,
            ..Default::default()
        };
        let response = boundary.advisory_response("yes".into()).unwrap();
        assert!(response.advisory_only);
        assert_eq!(
            boundary.advisory_response("nope".into()),
            Err(IntegrationError::OutputTooLarge { limit: 3 })
        );
    }

    #[test]
    fn smoke_report_covers_the_cognitive_vertical_slice_without_side_effects() {
        let report = CognitiveSmokeReport::complete();
        assert!(report.is_successful());
        assert_eq!(report.stages.len(), 6);
        assert!(!report.side_effects);
        assert!(!report.canonical_writes);
    }

    #[test]
    fn smoke_report_rejects_missing_or_unsafe_stages() {
        let mut report = CognitiveSmokeReport::complete();
        report.stages.pop();
        assert!(!report.is_successful());
        report = CognitiveSmokeReport::complete();
        report.canonical_writes = true;
        assert!(!report.is_successful());
    }

    #[test]
    fn resource_budget_is_bounded_and_rejects_overages() {
        let budget = ResourceBudget::bounded();
        assert!(budget.allows(100, 100, 2, 3));
        assert!(!budget.allows(100, 100, budget.max_events + 1, 3));
        assert!(!budget.allows(100, 100, 2, budget.max_replay_records + 1));
    }

    #[test]
    fn operator_snapshot_is_read_only_and_degrades_on_failed_certification() {
        let boundary = CognitiveBoundary::default();
        let smoke = CognitiveSmokeReport::complete();
        let ready = OperatorHealthSnapshot::from_boundary(boundary, &smoke, true);
        assert_eq!(ready.status, HealthStatus::Ready);
        assert!(!ready.canonical_write_authority);
        let degraded = OperatorHealthSnapshot::from_boundary(boundary, &smoke, false);
        assert_eq!(degraded.status, HealthStatus::Degraded);
    }
}
