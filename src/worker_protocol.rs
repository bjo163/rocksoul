//! Bounded, versioned parent/worker IPC contract.
//!
//! This module defines framing and protocol validation only. It does not spawn,
//! sandbox, supervise, terminate, or reap an OS process. Production dispatch must
//! remain disabled until a platform backend proves its isolation requirements.
use std::{
    collections::BTreeSet,
    io::{Read, Write},
};

use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};

pub const WORKER_PROTOCOL_VERSION: u16 = 1;
pub const MAX_WORKER_FRAME_BYTES: usize = 64 * 1024;
pub const MAX_WORKER_PAYLOAD_BYTES: usize = 48 * 1024;
const MAX_PROTOCOL_ID_BYTES: usize = 96;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerMessageKind {
    Hello,
    Ready,
    Execute,
    Cancel,
    Heartbeat,
    Result,
    Failed,
    Shutdown,
    Exited,
}

/// A single bounded protocol envelope. Payload is untrusted and never constitutes
/// authorization or verifier evidence by itself.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkerFrame {
    pub protocol_version: u16,
    pub run_id: String,
    pub request_id: String,
    pub sequence: u64,
    pub kind: WorkerMessageKind,
    pub deadline_unix_ms: u64,
    pub payload: serde_json::Value,
}

impl WorkerFrame {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.protocol_version == WORKER_PROTOCOL_VERSION,
            "unsupported worker protocol version"
        );
        validate_id(&self.run_id, "run id")?;
        validate_id(&self.request_id, "request id")?;
        ensure!(self.sequence > 0, "worker sequence must be positive");
        ensure!(
            self.deadline_unix_ms > 0,
            "worker deadline must be positive"
        );
        let payload_bytes = serde_json::to_vec(&self.payload)
            .context("could not encode worker payload for size validation")?;
        ensure!(
            payload_bytes.len() <= MAX_WORKER_PAYLOAD_BYTES,
            "worker payload exceeds size limit"
        );
        let encoded = serde_json::to_vec(self).context("could not encode worker frame")?;
        ensure!(
            encoded.len() <= MAX_WORKER_FRAME_BYTES,
            "worker frame exceeds size limit"
        );
        Ok(())
    }
}

/// Read one big-endian length-prefixed JSON frame. A clean EOF before any header
/// byte returns None; partial headers/bodies are protocol errors.
pub fn read_frame<R: Read>(reader: &mut R) -> Result<Option<WorkerFrame>> {
    let mut header = [0_u8; 4];
    match reader.read(&mut header[..1]) {
        Ok(0) => return Ok(None),
        Ok(1) => {}
        Ok(_) => unreachable!("one-byte read cannot return more than one byte"),
        Err(error) => return Err(error).context("could not read worker frame header"),
    }
    reader
        .read_exact(&mut header[1..])
        .context("truncated worker frame header")?;
    let length = u32::from_be_bytes(header) as usize;
    ensure!(
        length > 0 && length <= MAX_WORKER_FRAME_BYTES,
        "worker frame length is outside the allowed range"
    );
    let mut body = vec![0_u8; length];
    reader
        .read_exact(&mut body)
        .context("truncated worker frame body")?;
    let frame: WorkerFrame =
        serde_json::from_slice(&body).context("invalid worker frame JSON")?;
    frame.validate()?;
    Ok(Some(frame))
}

/// Write a validated frame with a four-byte big-endian length prefix.
pub fn write_frame<W: Write>(writer: &mut W, frame: &WorkerFrame) -> Result<()> {
    frame.validate()?;
    let body = serde_json::to_vec(frame).context("could not encode worker frame")?;
    let length = u32::try_from(body.len()).context("worker frame length overflow")?;
    writer
        .write_all(&length.to_be_bytes())
        .context("could not write worker frame header")?;
    writer
        .write_all(&body)
        .context("could not write worker frame body")?;
    writer.flush().context("could not flush worker frame")?;
    Ok(())
}

/// Stateful replay/ordering guard owned by the parent process.
#[derive(Debug)]
pub struct WorkerSession {
    run_id: String,
    next_sequence: u64,
    seen_request_ids: BTreeSet<String>,
}

impl WorkerSession {
    pub fn new(run_id: impl Into<String>) -> Result<Self> {
        let run_id = run_id.into();
        validate_id(&run_id, "run id")?;
        Ok(Self {
            run_id,
            next_sequence: 1,
            seen_request_ids: BTreeSet::new(),
        })
    }

    pub fn accept(&mut self, frame: &WorkerFrame) -> Result<()> {
        frame.validate()?;
        ensure!(frame.run_id == self.run_id, "worker run id mismatch");
        ensure!(
            frame.sequence == self.next_sequence,
            "worker frame is replayed or out of order"
        );
        ensure!(
            self.seen_request_ids.insert(frame.request_id.clone()),
            "worker request id was replayed"
        );
        self.next_sequence = self
            .next_sequence
            .checked_add(1)
            .context("worker sequence exhausted")?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkerLifecycle {
    Starting,
    Ready,
    Running,
    Cancelling,
    Terminating,
    Reaped,
    Failed,
}

impl WorkerLifecycle {
    /// Enforce terminal-state monotonicity. OS process exit/reaping must be observed
    /// by the platform supervisor; a protocol message cannot establish Reaped.
    pub fn transition(self, next: Self) -> Result<Self> {
        let allowed = matches!(
            (self, next),
            (Self::Starting, Self::Ready)
                | (Self::Starting, Self::Failed)
                | (Self::Ready, Self::Running)
                | (Self::Ready, Self::Terminating)
                | (Self::Ready, Self::Failed)
                | (Self::Running, Self::Ready)
                | (Self::Running, Self::Cancelling)
                | (Self::Running, Self::Terminating)
                | (Self::Running, Self::Failed)
                | (Self::Cancelling, Self::Terminating)
                | (Self::Cancelling, Self::Failed)
                | (Self::Terminating, Self::Reaped)
                | (Self::Terminating, Self::Failed)
                | (Self::Failed, Self::Terminating)
                | (Self::Failed, Self::Reaped)
        );
        ensure!(allowed, "invalid worker lifecycle transition");
        Ok(next)
    }
}

fn validate_id(value: &str, label: &str) -> Result<()> {
    ensure!(
        !value.is_empty()
            && value.len() <= MAX_PROTOCOL_ID_BYTES
            && value.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'_' | b'-' | b'.')
            }),
        "invalid {label}"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn frame(sequence: u64, request_id: &str) -> WorkerFrame {
        WorkerFrame {
            protocol_version: WORKER_PROTOCOL_VERSION,
            run_id: "run-1".to_owned(),
            request_id: request_id.to_owned(),
            sequence,
            kind: WorkerMessageKind::Execute,
            deadline_unix_ms: 1_800_000_000_000,
            payload: serde_json::json!({"tool_id":"fixture.state.read"}),
        }
    }

    #[test]
    fn framed_round_trip_preserves_valid_envelope() {
        let original = frame(1, "req-1");
        let mut bytes = Vec::new();
        write_frame(&mut bytes, &original).expect("write frame");
        let decoded = read_frame(&mut Cursor::new(bytes))
            .expect("read frame")
            .expect("frame present");
        assert_eq!(decoded, original);
    }

    #[test]
    fn clean_eof_is_distinct_from_truncated_header() {
        assert!(read_frame(&mut Cursor::new(Vec::<u8>::new()))
            .expect("clean eof")
            .is_none());
        assert!(read_frame(&mut Cursor::new(vec![0, 0])).is_err());
    }

    #[test]
    fn rejects_oversized_frame_before_allocating_body() {
        let bytes = ((MAX_WORKER_FRAME_BYTES + 1) as u32).to_be_bytes();
        assert!(read_frame(&mut Cursor::new(bytes)).is_err());
    }

    #[test]
    fn rejects_unknown_fields_and_unsupported_versions() {
        let mut unsupported = frame(1, "req-1");
        unsupported.protocol_version = WORKER_PROTOCOL_VERSION + 1;
        assert!(unsupported.validate().is_err());

        let raw = br#"{"protocol_version":1,"run_id":"run-1","request_id":"req-1","sequence":1,"kind":"execute","deadline_unix_ms":1800000000000,"payload":{},"unexpected":"no"}"#;
        assert!(serde_json::from_slice::<WorkerFrame>(raw).is_err());
    }

    #[test]
    fn rejects_oversized_payload_and_invalid_identifiers() {
        let mut oversized = frame(1, "req-1");
        oversized.payload = serde_json::json!({"data":"x".repeat(MAX_WORKER_PAYLOAD_BYTES)});
        assert!(oversized.validate().is_err());

        let mut invalid_id = frame(1, "req-1");
        invalid_id.request_id = "../escape".to_owned();
        assert!(invalid_id.validate().is_err());
    }

    #[test]
    fn session_rejects_replay_out_of_order_and_wrong_run() {
        let mut session = WorkerSession::new("run-1").expect("session");
        let first = frame(1, "req-1");
        session.accept(&first).expect("first frame");
        assert!(session.accept(&first).is_err());
        assert!(session.accept(&frame(3, "req-3")).is_err());

        let mut wrong_run = frame(2, "req-2");
        wrong_run.run_id = "run-2".to_owned();
        assert!(session.accept(&wrong_run).is_err());
    }

    #[test]
    fn lifecycle_does_not_allow_ready_to_claim_reaped() {
        assert!(WorkerLifecycle::Ready
            .transition(WorkerLifecycle::Reaped)
            .is_err());
        assert_eq!(
            WorkerLifecycle::Terminating
                .transition(WorkerLifecycle::Reaped)
                .expect("observed reap"),
            WorkerLifecycle::Reaped
        );
    }
}
