//! Parent-owned worker supervisor contract.
//!
//! The supervisor owns the lifecycle, deadline, request budget, and terminal
//! observation of a worker. Production start is intentionally fail-closed: this
//! build has no OS isolation backend, so it refuses to spawn an unsandboxed
//! process. The process traits exist so a future Linux/Windows backend can be
//! added without weakening the parent-owned contract.
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail, ensure};
use serde_json::json;

use crate::{
    sandbox::{FailClosedSandbox, PreparedSandbox, SandboxBackend, SandboxPolicy},
    worker_protocol::{
        WORKER_PROTOCOL_VERSION, WorkerFrame, WorkerLifecycle, WorkerMessageKind, WorkerSession,
    },
};

pub const MAX_WORKER_IPC_BYTES: usize = 1024 * 1024;
pub const MAX_WORKER_OUTPUT_BYTES: usize = 1024 * 1024;
pub const MAX_WORKER_REQUESTS: u64 = 1_024;
const MAX_RUN_ID_BYTES: usize = 96;

#[derive(Clone, Debug)]
pub struct WorkerRunPolicy {
    pub run_id: String,
    pub worker_executable: PathBuf,
    pub worker_sha256: String,
    pub deadline: Instant,
    pub deadline_unix_ms: u64,
    pub max_ipc_bytes: usize,
    pub max_stdout_bytes: usize,
    pub max_stderr_bytes: usize,
    pub max_requests: u64,
    pub sandbox: SandboxPolicy,
}

impl WorkerRunPolicy {
    pub fn validate(&self, now: Instant) -> Result<()> {
        ensure!(
            !self.run_id.is_empty()
                && self.run_id.len() <= MAX_RUN_ID_BYTES
                && self.run_id.bytes().all(|byte| {
                    byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'_' | b'-' | b'.')
                }),
            "invalid worker run id"
        );
        ensure!(
            !self.worker_executable.as_os_str().is_empty(),
            "worker executable path is empty"
        );
        ensure!(
            self.worker_sha256.len() == 64
                && self
                    .worker_sha256
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit()),
            "worker executable digest must be 64 hexadecimal characters"
        );
        ensure!(self.deadline > now, "worker deadline must be in the future");
        ensure!(
            self.deadline_unix_ms > 0,
            "worker deadline timestamp must be positive"
        );
        ensure!(
            self.max_ipc_bytes > 0 && self.max_ipc_bytes <= MAX_WORKER_IPC_BYTES,
            "worker IPC byte budget is outside the allowed range"
        );
        ensure!(
            self.max_stdout_bytes > 0 && self.max_stdout_bytes <= MAX_WORKER_OUTPUT_BYTES,
            "worker stdout budget is outside the allowed range"
        );
        ensure!(
            self.max_stderr_bytes > 0 && self.max_stderr_bytes <= MAX_WORKER_OUTPUT_BYTES,
            "worker stderr budget is outside the allowed range"
        );
        ensure!(
            self.max_requests > 0 && self.max_requests <= MAX_WORKER_REQUESTS,
            "worker request budget is outside the allowed range"
        );
        self.sandbox.validate()?;
        Ok(())
    }

    pub fn remaining(&self, now: Instant) -> Result<Duration> {
        let remaining = self
            .deadline
            .checked_duration_since(now)
            .ok_or_else(|| anyhow::anyhow!("worker deadline has expired"))?;
        ensure!(!remaining.is_zero(), "worker deadline has expired");
        Ok(remaining)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkerTerminationReason {
    Completed,
    DeadlineExceeded,
    Cancelled,
    ProtocolViolation,
    ParentShutdown,
    WorkerFailed,
    IsolationUnavailable,
}

impl WorkerTerminationReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::DeadlineExceeded => "deadline_exceeded",
            Self::Cancelled => "cancelled",
            Self::ProtocolViolation => "protocol_violation",
            Self::ParentShutdown => "parent_shutdown",
            Self::WorkerFailed => "worker_failed",
            Self::IsolationUnavailable => "isolation_unavailable",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkerExit {
    pub code: Option<i32>,
    pub signal: Option<i32>,
    pub reaped_at_unix_ms: u64,
    pub timed_out: bool,
}

impl WorkerExit {
    #[cfg(test)]
    fn observed(code: Option<i32>, signal: Option<i32>, timed_out: bool) -> Self {
        Self {
            code,
            signal,
            reaped_at_unix_ms: unix_millis(),
            timed_out,
        }
    }
}

#[cfg(test)]
fn unix_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

/// A launched worker process. Implementations must bind the IPC channel to the
/// OS-created child, not to a PID or identity asserted in a frame.
pub trait WorkerProcess: Send {
    fn send_frame(&mut self, frame: &WorkerFrame) -> Result<()>;
    fn receive_frame(&mut self, deadline: Instant) -> Result<Option<WorkerFrame>>;
    fn try_exit(&mut self) -> Result<Option<WorkerExit>>;
    fn terminate_tree(&mut self) -> Result<()>;
    fn wait_reap(&mut self, deadline: Instant) -> Result<WorkerExit>;
    fn stdout_bytes(&self) -> usize;
    fn stderr_bytes(&self) -> usize;
}

pub trait WorkerLauncher {
    fn launch(
        &self,
        policy: &WorkerRunPolicy,
        sandbox: &PreparedSandbox,
    ) -> Result<Box<dyn WorkerProcess>>;
}

/// Production launcher for the future OS-isolated backend. It refuses to spawn
/// until a real backend has prepared the sandbox.
#[derive(Clone, Copy, Debug, Default)]
pub struct SandboxedWorkerLauncher;

impl WorkerLauncher for SandboxedWorkerLauncher {
    fn launch(
        &self,
        _policy: &WorkerRunPolicy,
        _sandbox: &PreparedSandbox,
    ) -> Result<Box<dyn WorkerProcess>> {
        bail!("no OS-isolated worker launcher is enabled; refusing unsandboxed execution")
    }
}

pub struct WorkerSupervisor {
    policy: WorkerRunPolicy,
    session: WorkerSession,
    process: Box<dyn WorkerProcess>,
    lifecycle: WorkerLifecycle,
    ipc_bytes: usize,
    requests: u64,
    termination_reason: Option<WorkerTerminationReason>,
    exit: Option<WorkerExit>,
}

impl WorkerSupervisor {
    /// Production entry point. It verifies the isolation policy and refuses to
    /// start because no platform backend is implemented in this build.
    pub fn start(policy: WorkerRunPolicy) -> Result<Self> {
        let now = Instant::now();
        policy.validate(now)?;
        let prepared = FailClosedSandbox.prepare(&policy.sandbox)?;
        Self::start_with_launcher(policy, &prepared, &SandboxedWorkerLauncher)
    }

    /// Parent-owned lifecycle entry point for a prepared backend. It is private
    /// to this module so production callers cannot supply an arbitrary launcher.
    fn start_with_launcher(
        policy: WorkerRunPolicy,
        prepared: &PreparedSandbox,
        launcher: &dyn WorkerLauncher,
    ) -> Result<Self> {
        ensure!(
            prepared.status.supported && prepared.status.backend == policy.sandbox.backend,
            "prepared sandbox does not satisfy the run policy"
        );
        policy.validate(Instant::now())?;
        let process = launcher.launch(&policy, prepared)?;
        let session = WorkerSession::new(policy.run_id.clone())?;
        Ok(Self {
            policy,
            session,
            process,
            lifecycle: WorkerLifecycle::Starting,
            ipc_bytes: 0,
            requests: 0,
            termination_reason: None,
            exit: None,
        })
    }

    pub fn lifecycle(&self) -> WorkerLifecycle {
        self.lifecycle
    }

    pub fn termination_reason(&self) -> Option<WorkerTerminationReason> {
        self.termination_reason
    }

    pub fn exit(&self) -> Option<&WorkerExit> {
        self.exit.as_ref()
    }

    pub fn send(&mut self, frame: &WorkerFrame) -> Result<()> {
        self.ensure_live()?;
        self.policy.remaining(Instant::now())?;
        self.session.validate_for_worker(frame)?;
        let encoded = serde_json::to_vec(frame).context("could not encode worker frame")?;
        let frame_bytes = encoded
            .len()
            .checked_add(4)
            .context("worker frame size overflow")?;
        ensure!(
            self.ipc_bytes.saturating_add(frame_bytes) <= self.policy.max_ipc_bytes,
            "worker IPC byte budget exhausted"
        );
        ensure!(
            self.requests < self.policy.max_requests,
            "worker request budget exhausted"
        );
        self.process.send_frame(frame)?;
        self.ipc_bytes += frame_bytes;
        self.requests += 1;
        match (self.lifecycle, frame.kind) {
            (WorkerLifecycle::Ready, WorkerMessageKind::Execute) => {
                self.transition(WorkerLifecycle::Running)?;
            }
            (WorkerLifecycle::Running, WorkerMessageKind::Cancel) => {
                self.transition(WorkerLifecycle::Cancelling)?;
            }
            _ => {}
        }
        Ok(())
    }

    pub fn receive(&mut self) -> Result<Option<WorkerFrame>> {
        self.ensure_live()?;
        let remaining = self.policy.remaining(Instant::now())?;
        let frame = self.process.receive_frame(Instant::now() + remaining)?;
        let Some(frame) = frame else {
            return Ok(None);
        };
        if let Err(error) = self.session.accept_from_worker(&frame) {
            self.mark_failed(WorkerTerminationReason::ProtocolViolation);
            return Err(error);
        }
        self.apply_worker_frame(&frame)?;
        Ok(Some(frame))
    }

    /// Cooperative cancellation followed by a bounded grace period. If the
    /// worker does not exit, the parent terminates the process tree and waits
    /// for the OS-observed exit.
    pub fn cancel(&mut self, grace: Duration) -> Result<WorkerExit> {
        if let Some(exit) = &self.exit {
            return Ok(exit.clone());
        }
        self.ensure_live()?;
        if matches!(self.lifecycle, WorkerLifecycle::Starting) {
            self.transition(WorkerLifecycle::Failed)?;
            self.terminate(WorkerTerminationReason::Cancelled)?;
            return self.wait_and_reap(Duration::from_secs(2));
        }
        if !matches!(
            self.lifecycle,
            WorkerLifecycle::Cancelling | WorkerLifecycle::Terminating
        ) {
            self.transition(WorkerLifecycle::Cancelling)?;
        }
        let cancel = self.control_frame(WorkerMessageKind::Cancel, "cancel")?;
        let _ = self.send(&cancel);
        let deadline = Instant::now() + grace;
        loop {
            if let Some(exit) = self.process.try_exit()? {
                return self.finish(exit, WorkerTerminationReason::Cancelled);
            }
            if Instant::now() >= deadline {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        self.terminate(WorkerTerminationReason::Cancelled)?;
        self.wait_and_reap(Duration::from_secs(2))
    }

    pub fn terminate(&mut self, reason: WorkerTerminationReason) -> Result<()> {
        if matches!(self.lifecycle, WorkerLifecycle::Reaped) {
            return Ok(());
        }
        if matches!(self.lifecycle, WorkerLifecycle::Starting) {
            self.transition(WorkerLifecycle::Failed)?;
        }
        if !matches!(self.lifecycle, WorkerLifecycle::Terminating) {
            self.transition(WorkerLifecycle::Terminating)?;
        }
        self.termination_reason = Some(reason);
        if let Err(error) = self.process.terminate_tree() {
            self.mark_failed(reason);
            return Err(error).context("could not terminate worker process tree");
        }
        Ok(())
    }

    pub fn wait_and_reap(&mut self, timeout: Duration) -> Result<WorkerExit> {
        if let Some(exit) = &self.exit {
            return Ok(exit.clone());
        }
        ensure!(
            matches!(
                self.lifecycle,
                WorkerLifecycle::Terminating | WorkerLifecycle::Failed
            ),
            "worker must be terminating before reaping"
        );
        let exit = self.process.wait_reap(Instant::now() + timeout)?;
        self.finish(
            exit,
            self.termination_reason
                .unwrap_or(WorkerTerminationReason::WorkerFailed),
        )
    }

    pub fn shutdown(&mut self, reason: WorkerTerminationReason) -> Result<WorkerExit> {
        if let Some(exit) = &self.exit {
            return Ok(exit.clone());
        }
        self.terminate(reason)?;
        self.wait_and_reap(Duration::from_secs(2))
    }

    fn control_frame(&self, kind: WorkerMessageKind, request_id: &str) -> Result<WorkerFrame> {
        let frame = WorkerFrame {
            protocol_version: WORKER_PROTOCOL_VERSION,
            run_id: self.policy.run_id.clone(),
            request_id: request_id.to_owned(),
            sequence: 1,
            kind,
            deadline_unix_ms: self.policy.deadline_unix_ms,
            payload: json!({}),
        };
        frame.validate()?;
        Ok(frame)
    }

    fn apply_worker_frame(&mut self, frame: &WorkerFrame) -> Result<()> {
        match frame.kind {
            WorkerMessageKind::Hello => {
                if matches!(self.lifecycle, WorkerLifecycle::Starting) {
                    self.transition(WorkerLifecycle::Ready)?;
                }
            }
            WorkerMessageKind::Heartbeat | WorkerMessageKind::Result => {
                if matches!(self.lifecycle, WorkerLifecycle::Ready) {
                    self.transition(WorkerLifecycle::Running)?;
                }
            }
            WorkerMessageKind::Failed => {
                self.mark_failed(WorkerTerminationReason::WorkerFailed);
            }
            WorkerMessageKind::Exited => {
                if matches!(
                    self.lifecycle,
                    WorkerLifecycle::Ready | WorkerLifecycle::Running
                ) {
                    self.transition(WorkerLifecycle::Terminating)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn ensure_live(&self) -> Result<()> {
        ensure!(
            !matches!(
                self.lifecycle,
                WorkerLifecycle::Reaped | WorkerLifecycle::Failed
            ),
            "worker is no longer live"
        );
        Ok(())
    }

    fn transition(&mut self, next: WorkerLifecycle) -> Result<()> {
        self.lifecycle = self.lifecycle.transition(next)?;
        Ok(())
    }

    fn mark_failed(&mut self, reason: WorkerTerminationReason) {
        self.termination_reason = Some(reason);
        if !matches!(self.lifecycle, WorkerLifecycle::Failed) {
            self.lifecycle = WorkerLifecycle::Failed;
        }
    }

    fn finish(
        &mut self,
        mut exit: WorkerExit,
        reason: WorkerTerminationReason,
    ) -> Result<WorkerExit> {
        exit.timed_out = matches!(reason, WorkerTerminationReason::DeadlineExceeded);
        self.termination_reason = Some(reason);
        self.lifecycle = WorkerLifecycle::Reaped;
        self.exit = Some(exit.clone());
        Ok(exit)
    }
}

impl Drop for WorkerSupervisor {
    fn drop(&mut self) {
        if matches!(self.lifecycle, WorkerLifecycle::Reaped) {
            return;
        }
        let _ = self.process.terminate_tree();
        let _ = self
            .process
            .wait_reap(Instant::now() + Duration::from_secs(2));
        self.lifecycle = WorkerLifecycle::Reaped;
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::VecDeque,
        sync::{Arc, Mutex},
    };

    use super::*;
    use crate::sandbox::{SandboxPlatform, SandboxStatus};

    #[derive(Default)]
    struct FakeState {
        incoming: VecDeque<WorkerFrame>,
        sent: Vec<WorkerFrame>,
        exit: Option<WorkerExit>,
        terminate_calls: usize,
        reap_calls: usize,
        ignore_terminate: bool,
    }

    struct FakeProcess {
        state: Arc<Mutex<FakeState>>,
    }

    impl WorkerProcess for FakeProcess {
        fn send_frame(&mut self, frame: &WorkerFrame) -> Result<()> {
            self.state.lock().unwrap().sent.push(frame.clone());
            Ok(())
        }

        fn receive_frame(&mut self, _deadline: Instant) -> Result<Option<WorkerFrame>> {
            Ok(self.state.lock().unwrap().incoming.pop_front())
        }

        fn try_exit(&mut self) -> Result<Option<WorkerExit>> {
            Ok(self.state.lock().unwrap().exit.clone())
        }

        fn terminate_tree(&mut self) -> Result<()> {
            let mut state = self.state.lock().unwrap();
            state.terminate_calls += 1;
            if !state.ignore_terminate {
                state.exit = Some(WorkerExit::observed(None, Some(9), false));
            }
            Ok(())
        }

        fn wait_reap(&mut self, _deadline: Instant) -> Result<WorkerExit> {
            let mut state = self.state.lock().unwrap();
            state.reap_calls += 1;
            Ok(state
                .exit
                .clone()
                .unwrap_or_else(|| WorkerExit::observed(Some(0), None, false)))
        }

        fn stdout_bytes(&self) -> usize {
            0
        }

        fn stderr_bytes(&self) -> usize {
            0
        }
    }

    struct FakeLauncher {
        state: Arc<Mutex<FakeState>>,
    }

    impl WorkerLauncher for FakeLauncher {
        fn launch(
            &self,
            _policy: &WorkerRunPolicy,
            _sandbox: &PreparedSandbox,
        ) -> Result<Box<dyn WorkerProcess>> {
            Ok(Box::new(FakeProcess {
                state: Arc::clone(&self.state),
            }))
        }
    }

    fn policy(deadline: Instant) -> WorkerRunPolicy {
        let scratch = std::env::temp_dir().join("rocksoul-worker-test");
        let sandbox = match SandboxPlatform::current() {
            SandboxPlatform::Windows => SandboxPolicy::windows_default(scratch),
            _ => SandboxPolicy::linux_default(scratch),
        };
        WorkerRunPolicy {
            run_id: "run-1".to_owned(),
            worker_executable: PathBuf::from("/tmp/rocksoul-worker"),
            worker_sha256: "a".repeat(64),
            deadline,
            deadline_unix_ms: 1_800_000_000_000,
            max_ipc_bytes: 64 * 1024,
            max_stdout_bytes: 64 * 1024,
            max_stderr_bytes: 64 * 1024,
            max_requests: 16,
            sandbox,
        }
    }

    fn prepared(policy: &WorkerRunPolicy) -> PreparedSandbox {
        PreparedSandbox {
            backend: policy.sandbox.backend,
            policy_sha256: policy.sandbox.digest(),
            status: SandboxStatus {
                supported: true,
                backend: policy.sandbox.backend,
                platform: SandboxPlatform::current(),
                missing: Vec::new(),
            },
        }
    }

    fn frame(kind: WorkerMessageKind, sequence: u64, request_id: &str) -> WorkerFrame {
        WorkerFrame {
            protocol_version: WORKER_PROTOCOL_VERSION,
            run_id: "run-1".to_owned(),
            request_id: request_id.to_owned(),
            sequence,
            kind,
            deadline_unix_ms: 1_800_000_000_000,
            payload: json!({}),
        }
    }

    fn supervisor(state: Arc<Mutex<FakeState>>, policy: WorkerRunPolicy) -> WorkerSupervisor {
        let prepared = prepared(&policy);
        let launcher = FakeLauncher { state };
        WorkerSupervisor::start_with_launcher(policy, &prepared, &launcher).unwrap()
    }

    #[test]
    fn production_start_fails_closed_without_os_isolation() {
        let policy = policy(Instant::now() + Duration::from_secs(30));
        assert!(WorkerSupervisor::start(policy).is_err());
    }

    #[test]
    fn policy_rejects_network_and_expired_deadline() {
        let mut network_policy = policy(Instant::now() + Duration::from_secs(30));
        network_policy.sandbox.allow_network = true;
        assert!(network_policy.validate(Instant::now()).is_err());
        let expired = policy(Instant::now() - Duration::from_secs(1));
        assert!(expired.validate(Instant::now()).is_err());
    }

    #[test]
    fn lifecycle_hello_execute_result_and_cancel_reap() {
        let state = Arc::new(Mutex::new(FakeState::default()));
        {
            let mut state = state.lock().unwrap();
            state
                .incoming
                .push_back(frame(WorkerMessageKind::Hello, 1, "hello"));
            state
                .incoming
                .push_back(frame(WorkerMessageKind::Result, 2, "result"));
        }
        let mut supervisor = supervisor(
            Arc::clone(&state),
            policy(Instant::now() + Duration::from_secs(30)),
        );
        assert_eq!(supervisor.lifecycle(), WorkerLifecycle::Starting);
        assert!(supervisor.receive().unwrap().is_some());
        assert_eq!(supervisor.lifecycle(), WorkerLifecycle::Ready);
        supervisor
            .send(&frame(WorkerMessageKind::Execute, 1, "execute"))
            .unwrap();
        assert_eq!(supervisor.lifecycle(), WorkerLifecycle::Running);
        assert!(supervisor.receive().unwrap().is_some());
        let exit = supervisor.cancel(Duration::from_millis(1)).unwrap();
        assert_eq!(supervisor.lifecycle(), WorkerLifecycle::Reaped);
        assert_eq!(
            supervisor.termination_reason(),
            Some(WorkerTerminationReason::Cancelled)
        );
        assert_eq!(exit.signal, Some(9));
        let state = state.lock().unwrap();
        assert_eq!(state.terminate_calls, 1);
        assert_eq!(state.reap_calls, 1);
        assert!(
            state
                .sent
                .iter()
                .any(|frame| frame.kind == WorkerMessageKind::Cancel)
        );
    }

    #[test]
    fn worker_to_parent_direction_is_enforced() {
        let state = Arc::new(Mutex::new(FakeState::default()));
        state
            .lock()
            .unwrap()
            .incoming
            .push_back(frame(WorkerMessageKind::Execute, 1, "execute"));
        let mut supervisor = supervisor(
            Arc::clone(&state),
            policy(Instant::now() + Duration::from_secs(30)),
        );
        assert!(supervisor.receive().is_err());
        assert_eq!(supervisor.lifecycle(), WorkerLifecycle::Failed);
    }

    #[test]
    fn replayed_worker_sequence_is_rejected() {
        let state = Arc::new(Mutex::new(FakeState::default()));
        {
            let mut state = state.lock().unwrap();
            state
                .incoming
                .push_back(frame(WorkerMessageKind::Hello, 1, "hello"));
            state
                .incoming
                .push_back(frame(WorkerMessageKind::Hello, 1, "hello-2"));
        }
        let mut supervisor = supervisor(
            Arc::clone(&state),
            policy(Instant::now() + Duration::from_secs(30)),
        );
        assert!(supervisor.receive().unwrap().is_some());
        assert!(supervisor.receive().is_err());
    }

    #[test]
    fn drop_terminates_and_reaps_instead_of_detaching() {
        let state = Arc::new(Mutex::new(FakeState::default()));
        let supervisor = supervisor(
            Arc::clone(&state),
            policy(Instant::now() + Duration::from_secs(30)),
        );
        drop(supervisor);
        let state = state.lock().unwrap();
        assert_eq!(state.terminate_calls, 1);
        assert_eq!(state.reap_calls, 1);
    }

    #[test]
    fn terminal_worker_rejects_further_messages() {
        let state = Arc::new(Mutex::new(FakeState::default()));
        let mut supervisor = supervisor(
            Arc::clone(&state),
            policy(Instant::now() + Duration::from_secs(30)),
        );
        let exit = supervisor
            .shutdown(WorkerTerminationReason::ParentShutdown)
            .unwrap();
        assert_eq!(supervisor.lifecycle(), WorkerLifecycle::Reaped);
        assert!(!exit.timed_out);
        assert!(
            supervisor
                .send(&frame(WorkerMessageKind::Execute, 1, "execute"))
                .is_err()
        );
    }

    #[test]
    fn ipc_budget_is_enforced_before_write() {
        let state = Arc::new(Mutex::new(FakeState::default()));
        let mut policy = policy(Instant::now() + Duration::from_secs(30));
        policy.max_ipc_bytes = 1;
        let mut supervisor = supervisor(Arc::clone(&state), policy);
        assert!(
            supervisor
                .send(&frame(WorkerMessageKind::Execute, 1, "execute"))
                .is_err()
        );
        assert!(state.lock().unwrap().sent.is_empty());
    }
}
