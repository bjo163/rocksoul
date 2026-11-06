//! Content-free, versioned tool audit records and cooperative invocation controls.
//!
//! This module records metadata only. It does not dispatch tools, create workers,
//! enforce OS isolation, or prove that a timed-out worker has stopped.
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::OpenOptions,
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU16, Ordering},
    },
    time::{Duration, Instant},
};

use anyhow::{Context, Result, ensure};
use fs2::FileExt;
use serde::{Deserialize, Serialize};

pub const TOOL_AUDIT_SCHEMA_VERSION: u8 = 1;
const MAX_ID_BYTES: usize = 96;
const MAX_VERSION_BYTES: usize = 64;
const MAX_EVIDENCE_REFS: usize = 16;
const MAX_EVIDENCE_REF_BYTES: usize = 160;
const MAX_RECORD_BYTES: usize = 4_096;
const MAX_AUDIT_FILE_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolAuditKind {
    Proposed,
    AuthorizationAllowed,
    AuthorizationDenied,
    ApprovalGranted,
    ApprovalDenied,
    ExecutionStarted,
    ExecutionFinished,
    TimedOut,
    Cancelled,
    VerifierReportedPassed,
    VerifierReportedFailed,
    VerifierReportedIndeterminate,
    Failed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolAuthorization {
    Allowed,
    Denied,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolEffectClass {
    ReadOnly,
    Mutating,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolFailureCategory {
    InvalidInput,
    PolicyDenied,
    ApprovalDenied,
    DeadlineExceeded,
    Cancelled,
    ResourceExhausted,
    WorkerFailure,
    VerificationFailure,
    Unknown,
}

/// Closed metadata schema: unknown fields (including accidental content fields) are rejected.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ToolAuditEvent {
    pub schema_version: u8,
    pub run_id: String,
    pub sequence: u64,
    pub event_id: String,
    pub kind: ToolAuditKind,
    pub tool_id: String,
    pub tool_version: String,
    pub capability: String,
    pub effect: ToolEffectClass,
    pub policy_version: String,
    pub timestamp_unix_ms: u64,
    pub authorization: Option<ToolAuthorization>,
    pub failure: Option<ToolFailureCategory>,
    pub evidence_refs: Vec<String>,
}

impl ToolAuditEvent {
    pub fn validate(&self) -> Result<()> {
        ensure!(self.schema_version == TOOL_AUDIT_SCHEMA_VERSION, "unsupported tool audit schema version");
        validate_identifier(&self.run_id, "run id")?;
        validate_identifier(&self.event_id, "event id")?;
        validate_identifier(&self.tool_id, "tool id")?;
        validate_version(&self.tool_version, "tool version")?;
        validate_identifier(&self.capability, "capability")?;
        validate_version(&self.policy_version, "policy version")?;
        ensure!(self.sequence > 0, "tool audit sequence must be positive");
        ensure!(self.timestamp_unix_ms > 0, "tool audit timestamp must be positive");
        ensure!(self.evidence_refs.len() <= MAX_EVIDENCE_REFS, "too many evidence references");
        for reference in &self.evidence_refs {
            ensure!(!reference.is_empty() && reference.len() <= MAX_EVIDENCE_REF_BYTES, "invalid evidence reference length");
            ensure!(reference.bytes().all(is_safe_metadata_byte), "evidence reference contains unsupported characters");
        }
        match self.kind {
            ToolAuditKind::AuthorizationAllowed => {
                ensure!(self.authorization == Some(ToolAuthorization::Allowed), "authorization event must record allowed");
            }
            ToolAuditKind::AuthorizationDenied => {
                ensure!(self.authorization == Some(ToolAuthorization::Denied), "authorization event must record denied");
            }
            ToolAuditKind::TimedOut => {
                ensure!(self.failure == Some(ToolFailureCategory::DeadlineExceeded), "timeout event must use deadline_exceeded");
            }
            ToolAuditKind::Cancelled => {
                ensure!(self.failure == Some(ToolFailureCategory::Cancelled), "cancel event must use cancelled");
            }
            ToolAuditKind::Failed => {
                ensure!(self.failure.is_some(), "failure event requires a failure category");
            }
            _ => {}
        }
        Ok(())
    }
}

fn is_safe_metadata_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'/' | b'.' | b'_' | b'-')
}

fn validate_identifier(value: &str, label: &str) -> Result<()> {
    ensure!(!value.is_empty() && value.len() <= MAX_ID_BYTES, "{label} must be 1-96 bytes");
    ensure!(value.bytes().all(is_safe_metadata_byte), "{label} contains unsupported characters");
    Ok(())
}

fn validate_version(value: &str, label: &str) -> Result<()> {
    ensure!(!value.is_empty() && value.len() <= MAX_VERSION_BYTES, "{label} must be 1-64 bytes");
    ensure!(value.bytes().all(is_safe_metadata_byte), "{label} contains unsupported characters");
    Ok(())
}


/// Append-only JSONL audit storage. A corrupt or partial existing file is never repaired
/// automatically: append fails closed and preserves the original bytes.
pub struct ToolAuditStore;

impl ToolAuditStore {
    pub fn append(path: &Path, event: &ToolAuditEvent) -> Result<()> {
        event.validate()?;
        let encoded = serde_json::to_vec(event).context("could not serialize tool audit event")?;
        ensure!(encoded.len() <= MAX_RECORD_BYTES, "tool audit record exceeds size limit");

        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(path)
            .with_context(|| format!("could not open tool audit store {}", path.display()))?;
        FileExt::lock_exclusive(&file)
            .with_context(|| format!("could not lock tool audit store {}", path.display()))?;

        let result = (|| -> Result<()> {
            let end = file.seek(SeekFrom::End(0))?;
            ensure!(end <= MAX_AUDIT_FILE_BYTES, "tool audit store exceeds size limit");
            file.seek(SeekFrom::Start(0))?;
            let mut existing = String::new();
            file.read_to_string(&mut existing)
                .context("tool audit store is not valid UTF-8")?;
            let last_sequence = validate_existing_records(&existing)?;
            let expected = last_sequence
                .get(&event.run_id)
                .copied()
                .unwrap_or(0)
                .checked_add(1)
                .context("tool audit sequence exhausted")?;
            ensure!(event.sequence == expected, "tool audit sequence must be contiguous for each run");
            file.seek(SeekFrom::End(0))?;
            file.write_all(&encoded).context("could not append tool audit event")?;
            file.write_all(b"\n").context("could not terminate tool audit record")?;
            file.sync_data().context("could not persist tool audit event")?;
            Ok(())
        })();

        let unlock_result = FileExt::unlock(&file)
            .with_context(|| format!("could not unlock tool audit store {}", path.display()));
        result?;
        unlock_result?;
        Ok(())
    }

    pub fn read(path: &Path) -> Result<Vec<ToolAuditEvent>> {
        if !path.exists() {
            return Ok(Vec::new());
        }
        let mut file = OpenOptions::new()
            .read(true)
            .open(path)
            .with_context(|| format!("could not open tool audit store {}", path.display()))?;
        FileExt::lock_shared(&file)
            .with_context(|| format!("could not lock tool audit store {}", path.display()))?;
        let result = (|| -> Result<Vec<ToolAuditEvent>> {
            let metadata = file.metadata()?;
            ensure!(metadata.len() <= MAX_AUDIT_FILE_BYTES, "tool audit store exceeds read limit");
            let mut content = String::new();
            file.read_to_string(&mut content)
                .context("tool audit store is not valid UTF-8")?;
            validate_existing_records(&content)?;
            content
                .lines()
                .filter(|line| !line.is_empty())
                .map(|line| {
                    serde_json::from_str::<ToolAuditEvent>(line)
                        .context("tool audit store contains an invalid record")
                })
                .collect()
        })();
        let unlock_result = FileExt::unlock(&file)
            .with_context(|| format!("could not unlock tool audit store {}", path.display()));
        result.and_then(|records| {
            unlock_result?;
            Ok(records)
        })
    }
}

fn validate_existing_records(content: &str) -> Result<BTreeMap<String, u64>> {
    if content.is_empty() {
        return Ok(BTreeMap::new());
    }
    ensure!(content.ends_with('\n'), "tool audit store has an incomplete trailing record");
    let mut sequences = BTreeMap::new();
    let mut event_ids = BTreeSet::new();
    for line in content.lines() {
        ensure!(!line.is_empty() && line.len() <= MAX_RECORD_BYTES, "tool audit store contains an empty or oversized record");
        let event: ToolAuditEvent = serde_json::from_str(line)
            .context("tool audit store contains a malformed record")?;
        event.validate()?;
        ensure!(event_ids.insert(event.event_id.clone()), "tool audit store contains a duplicate event id");
        let expected = sequences
            .get(&event.run_id)
            .copied()
            .unwrap_or(0_u64)
            .checked_add(1)
            .context("tool audit sequence exhausted")?;
        ensure!(event.sequence == expected, "tool audit store contains duplicate or out-of-order sequence");
        sequences.insert(event.run_id, event.sequence);
    }
    Ok(sequences)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InvocationStopReason {
    TimedOut,
    Cancelled,
    CallBudgetExhausted,
}

/// Cooperative host-side pre-dispatch guard. It does not terminate or reap a worker.
#[derive(Clone, Debug)]
pub struct InvocationControl {
    deadline: Instant,
    cancelled: Arc<AtomicBool>,
    remaining_calls: Arc<AtomicU16>,
}

impl InvocationControl {
    pub fn new(timeout: Duration, max_calls: u16) -> Self {
        Self {
            deadline: Instant::now() + timeout,
            cancelled: Arc::new(AtomicBool::new(false)),
            remaining_calls: Arc::new(AtomicU16::new(max_calls)),
        }
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    /// Check immediately before dispatch. A successful check consumes one call.
    pub fn check_and_consume(&self) -> std::result::Result<u16, InvocationStopReason> {
        if self.cancelled.load(Ordering::SeqCst) {
            return Err(InvocationStopReason::Cancelled);
        }
        if Instant::now() >= self.deadline {
            return Err(InvocationStopReason::TimedOut);
        }
        self.remaining_calls
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |remaining| {
                if remaining > 0 { Some(remaining - 1) } else { None }
            })
            .map(|previous| previous - 1)
            .map_err(|_| InvocationStopReason::CallBudgetExhausted)
    }

    pub fn remaining_calls(&self) -> u16 {
        self.remaining_calls.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, sync::{Arc, Barrier}, thread, time::{Duration, SystemTime, UNIX_EPOCH}};

    use super::{
        InvocationControl, InvocationStopReason, ToolAuditEvent, ToolAuditKind, ToolAuditStore,
        ToolAuthorization, ToolEffectClass, ToolFailureCategory,
    };

    fn temp_path(label: &str) -> std::path::PathBuf {
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        std::env::temp_dir().join(format!("rocksoul-tool-audit-{label}-{nonce}.jsonl"))
    }

    fn event(run: &str, sequence: u64) -> ToolAuditEvent {
        ToolAuditEvent {
            schema_version: 1,
            run_id: run.into(),
            sequence,
            event_id: format!("event-{run}-{sequence}"),
            kind: ToolAuditKind::Proposed,
            tool_id: "workspace.read".into(),
            tool_version: "1.0.0".into(),
            capability: "workspace_read".into(),
            effect: ToolEffectClass::ReadOnly,
            policy_version: "1.0.0".into(),
            timestamp_unix_ms: 1,
            authorization: None,
            failure: None,
            evidence_refs: vec!["sha256:0123456789abcdef".into()],
        }
    }

    #[test]
    fn writes_content_free_versioned_records_and_reads_them_back() {
        let path = temp_path("roundtrip");
        ToolAuditStore::append(&path, &event("run-a", 1)).unwrap();
        let records = ToolAuditStore::read(&path).unwrap();
        assert_eq!(records, vec![event("run-a", 1)]);
        let text = fs::read_to_string(&path).unwrap();
        assert!(text.contains("\"schema_version\":1"));
        assert!(!text.contains("prompt"));
        assert!(!text.contains("secret"));
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn rejects_unknown_versions_unsafe_identifiers_and_content_fields() {
        let mut invalid = event("run-a", 1);
        invalid.schema_version = 2;
        assert!(invalid.validate().is_err());
        invalid = event("run-a", 1);
        invalid.tool_id = "tool\napi_key".into();
        assert!(invalid.validate().is_err());
        let raw = r#"{"schema_version":1,"run_id":"r","sequence":1,"event_id":"e","kind":"proposed","tool_id":"t","tool_version":"1","capability":"read","effect":"read_only","policy_version":"1","timestamp_unix_ms":1,"authorization":null,"failure":null,"evidence_refs":[],"prompt":"must reject"}"#;
        assert!(serde_json::from_str::<ToolAuditEvent>(raw).is_err());
    }

    #[test]
    fn enforces_event_specific_authorization_and_failure_metadata() {
        let mut invalid = event("run-a", 1);
        invalid.kind = ToolAuditKind::AuthorizationDenied;
        invalid.authorization = Some(ToolAuthorization::Allowed);
        assert!(invalid.validate().is_err());
        invalid.kind = ToolAuditKind::TimedOut;
        invalid.authorization = None;
        invalid.failure = Some(ToolFailureCategory::WorkerFailure);
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn rejects_duplicate_or_out_of_order_sequences_without_mutating_history() {
        let path = temp_path("sequence");
        ToolAuditStore::append(&path, &event("run-a", 1)).unwrap();
        let before = fs::read(&path).unwrap();
        assert!(ToolAuditStore::append(&path, &event("run-a", 1)).is_err());
        assert_eq!(fs::read(&path).unwrap(), before);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn malformed_or_partial_history_is_preserved_and_append_fails_closed() {
        let path = temp_path("corrupt");
        fs::write(&path, b"{not-json}\n").unwrap();
        let before = fs::read(&path).unwrap();
        assert!(ToolAuditStore::append(&path, &event("run-a", 1)).is_err());
        assert_eq!(fs::read(&path).unwrap(), before);
        fs::remove_file(path).unwrap();

        let path = temp_path("partial");
        fs::write(&path, b"{\"partial\":").unwrap();
        let before = fs::read(&path).unwrap();
        assert!(ToolAuditStore::append(&path, &event("run-a", 1)).is_err());
        assert_eq!(fs::read(&path).unwrap(), before);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn concurrent_writers_are_serialized_and_sequences_remain_valid() {
        let path = temp_path("concurrent");
        let barrier = Arc::new(Barrier::new(8));
        let mut workers = Vec::new();
        for index in 0..8 {
            let path = path.clone();
            let barrier = barrier.clone();
            workers.push(thread::spawn(move || {
                barrier.wait();
                ToolAuditStore::append(&path, &event(&format!("run-{index}"), 1))
            }));
        }
        for worker in workers {
            worker.join().unwrap().unwrap();
        }
        assert_eq!(ToolAuditStore::read(&path).unwrap().len(), 8);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_helper_process() {
        if let (Ok(path), Ok(run_id)) = (
            std::env::var("ROCKSOUL_AUDIT_HELPER_PATH"),
            std::env::var("ROCKSOUL_AUDIT_HELPER_RUN"),
        ) {
            ToolAuditStore::append(std::path::Path::new(&path), &event(&run_id, 1)).unwrap();
        }
    }

    #[test]
    fn cross_process_appends_are_serialized_and_remain_parseable() {
        use std::process::Command;

        let path = temp_path("process");
        let executable = std::env::current_exe().unwrap();
        let mut children = Vec::new();
        for index in 0..4 {
            children.push(
                Command::new(&executable)
                    .arg("--exact")
                    .arg("tool_audit::tests::append_helper_process")
                    .arg("--nocapture")
                    .env("ROCKSOUL_AUDIT_HELPER_PATH", &path)
                    .env("ROCKSOUL_AUDIT_HELPER_RUN", format!("process-{index}"))
                    .spawn()
                    .unwrap(),
            );
        }
        for mut child in children {
            assert!(child.wait().unwrap().success());
        }
        assert_eq!(ToolAuditStore::read(&path).unwrap().len(), 4);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn invocation_control_classifies_cancel_timeout_and_call_budget() {
        let control = InvocationControl::new(Duration::from_secs(5), 1);
        assert_eq!(control.check_and_consume(), Ok(0));
        assert_eq!(control.check_and_consume(), Err(InvocationStopReason::CallBudgetExhausted));
        control.cancel();
        assert_eq!(control.check_and_consume(), Err(InvocationStopReason::Cancelled));

        let expired = InvocationControl::new(Duration::ZERO, 1);
        assert_eq!(expired.check_and_consume(), Err(InvocationStopReason::TimedOut));
    }
}
