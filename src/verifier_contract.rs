//! Test-only contract for host-created verifier challenges and evidence.
//!
//! This module does not implement a verifier process, signatures, key
//! management, or an independent trust domain. It only enforces the shape of
//! the contract so a future real adapter cannot be replaced by a model
//! self-report. A `Pass` requires `independent = true`; this flag is evidence
//! supplied by the host, never by the model or the guest worker.
use anyhow::{Result, ensure};

const MAX_ID_BYTES: usize = 96;
const MAX_NONCE_BYTES: usize = 96;
const MIN_NONCE_BYTES: usize = 16;
const MAX_EVIDENCE_REFS: usize = 16;
const MAX_REF_BYTES: usize = 160;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerificationDecision {
    Pass,
    Fail,
    Indeterminate,
}

impl VerificationDecision {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
            Self::Indeterminate => "indeterminate",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskAttemptEnvelope {
    pub task_id: String,
    pub attempt_id: String,
    pub request_sha256: String,
    pub tool_calls_sha256: String,
    pub created_at_unix: u64,
    pub challenge_nonce: String,
}

impl TaskAttemptEnvelope {
    pub fn validate(&self) -> Result<()> {
        validate_id(&self.task_id, "task id")?;
        validate_id(&self.attempt_id, "attempt id")?;
        validate_digest(&self.request_sha256, "request digest")?;
        validate_digest(&self.tool_calls_sha256, "tool calls digest")?;
        ensure!(
            self.created_at_unix > 0,
            "attempt timestamp must be positive"
        );
        validate_nonce(&self.challenge_nonce)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifierEvidence {
    pub verifier_id: String,
    /// Host-asserted trust-domain flag. Never accept `Pass` without it.
    pub independent: bool,
    pub challenge_nonce: String,
    pub observation_refs: Vec<String>,
    pub decision: VerificationDecision,
    pub confidence_basis_points: u16,
    pub attested_at_unix: u64,
}

impl VerifierEvidence {
    pub fn validate(&self) -> Result<()> {
        validate_id(&self.verifier_id, "verifier id")?;
        validate_nonce(&self.challenge_nonce)?;
        ensure!(
            !self.observation_refs.is_empty() && self.observation_refs.len() <= MAX_EVIDENCE_REFS,
            "verifier observation reference count is outside the allowed range"
        );
        for reference in &self.observation_refs {
            ensure!(
                !reference.is_empty()
                    && reference.len() <= MAX_REF_BYTES
                    && reference.bytes().all(|byte| byte.is_ascii_alphanumeric()
                        || matches!(byte, b':' | b'/' | b'.' | b'_' | b'-')),
                "invalid verifier observation reference"
            );
        }
        ensure!(
            self.confidence_basis_points <= 10_000,
            "verifier confidence exceeds 10000 basis points"
        );
        ensure!(
            self.attested_at_unix > 0,
            "attestation timestamp must be positive"
        );
        Ok(())
    }
}

/// Single-use challenge session owned by the host. A verifier can answer once;
/// a replayed or mismatched attempt is refused.
#[derive(Debug)]
pub struct VerifierSession {
    envelope: TaskAttemptEnvelope,
    used: bool,
}

impl VerifierSession {
    pub fn new(envelope: TaskAttemptEnvelope) -> Result<Self> {
        envelope.validate()?;
        Ok(Self {
            envelope,
            used: false,
        })
    }

    pub fn is_used(&self) -> bool {
        self.used
    }

    pub fn verify(&mut self, evidence: &VerifierEvidence) -> Result<VerificationDecision> {
        ensure!(!self.used, "verifier challenge was already consumed");
        evidence.validate()?;
        ensure!(
            evidence.challenge_nonce == self.envelope.challenge_nonce,
            "verifier challenge nonce does not match the host-created envelope"
        );
        ensure!(
            evidence.attested_at_unix >= self.envelope.created_at_unix,
            "verifier attestation predates the task attempt"
        );
        if evidence.decision == VerificationDecision::Pass {
            ensure!(
                evidence.independent,
                "a non-independent verifier cannot establish success"
            );
        }
        self.used = true;
        Ok(evidence.decision)
    }
}

fn validate_id(value: &str, label: &str) -> Result<()> {
    ensure!(
        !value.is_empty()
            && value.len() <= MAX_ID_BYTES
            && value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric()
                    || matches!(byte, b':' | b'_' | b'-' | b'.')),
        "invalid {label}"
    );
    Ok(())
}

fn validate_digest(value: &str, label: &str) -> Result<()> {
    ensure!(
        value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()),
        "invalid {label}"
    );
    Ok(())
}

fn validate_nonce(value: &str) -> Result<()> {
    ensure!(
        value.len() >= MIN_NONCE_BYTES
            && value.len() <= MAX_NONCE_BYTES
            && value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-')),
        "invalid verifier challenge nonce"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope() -> TaskAttemptEnvelope {
        TaskAttemptEnvelope {
            task_id: "task-1".to_owned(),
            attempt_id: "attempt-1".to_owned(),
            request_sha256: "a".repeat(64),
            tool_calls_sha256: "b".repeat(64),
            created_at_unix: 1_800_000_000,
            challenge_nonce: "nonce-0123456789abcdef".to_owned(),
        }
    }

    fn evidence(decision: VerificationDecision, independent: bool) -> VerifierEvidence {
        VerifierEvidence {
            verifier_id: "verifier-1".to_owned(),
            independent,
            challenge_nonce: envelope().challenge_nonce,
            observation_refs: vec!["observed://file/digest".to_owned()],
            decision,
            confidence_basis_points: 9_000,
            attested_at_unix: 1_800_000_001,
        }
    }

    #[test]
    fn independent_pass_is_accepted_once() {
        let mut session = VerifierSession::new(envelope()).unwrap();
        assert_eq!(
            session
                .verify(&evidence(VerificationDecision::Pass, true))
                .unwrap(),
            VerificationDecision::Pass
        );
        assert!(session.is_used());
        assert!(
            session
                .verify(&evidence(VerificationDecision::Pass, true))
                .is_err()
        );
    }

    #[test]
    fn self_grading_cannot_establish_success() {
        let mut session = VerifierSession::new(envelope()).unwrap();
        assert!(
            session
                .verify(&evidence(VerificationDecision::Pass, false))
                .is_err()
        );
        assert!(!session.is_used());
    }

    #[test]
    fn failure_and_indeterminate_do_not_require_independence_but_are_bounded() {
        let mut session = VerifierSession::new(envelope()).unwrap();
        assert_eq!(
            session
                .verify(&evidence(VerificationDecision::Fail, false))
                .unwrap(),
            VerificationDecision::Fail
        );
        let mut session = VerifierSession::new(envelope()).unwrap();
        let mut bad = evidence(VerificationDecision::Indeterminate, false);
        bad.confidence_basis_points = 10_001;
        assert!(session.verify(&bad).is_err());
    }

    #[test]
    fn replay_and_wrong_challenge_are_refused() {
        let mut session = VerifierSession::new(envelope()).unwrap();
        let mut wrong = evidence(VerificationDecision::Pass, true);
        wrong.challenge_nonce = "another-nonce-0123456789".to_owned();
        assert!(session.verify(&wrong).is_err());
        assert!(!session.is_used());
        let mut stale = evidence(VerificationDecision::Pass, true);
        stale.attested_at_unix = 1_799_999_999;
        assert!(session.verify(&stale).is_err());
        assert!(
            session
                .verify(&evidence(VerificationDecision::Pass, true))
                .is_ok()
        );
        assert!(session.is_used());
    }

    #[test]
    fn malformed_envelope_and_evidence_are_rejected() {
        let mut invalid = envelope();
        invalid.challenge_nonce = "short".to_owned();
        assert!(VerifierSession::new(invalid).is_err());
        let mut bad_refs = evidence(VerificationDecision::Pass, true);
        bad_refs.observation_refs.clear();
        assert!(bad_refs.validate().is_err());
    }
}
