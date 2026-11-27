//! Parent-owned worker supervisor contract.
//!
//! The supervisor owns the lifecycle, deadline, request budget, and terminal
//! observation of a worker. Production start is intentionally fail-closed: this
//! build has no OS isolation backend, so it refuses to spawn an unsandboxed
//! process. The process traits exist so a future Linux/Windows backend can be
//! added without weakening the parent-owned contract.
use std::{
    fs,
    io::Read,
    path::PathBuf,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail, ensure};
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::{
    sandbox::{FailClosedSandbox, PreparedSandbox, SandboxBackend, SandboxPolicy},
    worker_protocol::{
        WORKER_PROTOCOL_VERSION, WorkerFrame, WorkerLifecycle, WorkerMessageKind, WorkerSession,
    },
};

pub const MAX_WORKER_IPC_BYTES: usize = 1024 * 1024;
pub const MAX_WORKER_OUTPUT_BYTES: usize = 1024 * 1024;
pub const MAX_WORKER_REQUESTS: u64 = 1_024;
pub const MAX_WORKER_EXECUTABLE_BYTES: u64 = 64 * 1024 * 1024;
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

    /// Every frame must carry the exact policy deadline; a worker cannot extend
    /// its own budget by asserting a later timestamp.
    pub fn verify_frame_deadline(&self, frame: &WorkerFrame) -> Result<()> {
        ensure!(
            frame.deadline_unix_ms == self.deadline_unix_ms,
            "frame deadline does not match the immutable run policy"
        );
        Ok(())
    }

    /// Verify the exact executable bytes before launch. A path or version string
    /// is not identity; a mismatch refuses the run. The file is read with a hard
    /// size ceiling so a hostile path cannot exhaust parent memory.
    pub fn verify_executable(&self) -> Result<()> {
        let metadata = fs::metadata(&self.worker_executable)
            .context("could not inspect the worker executable")?;
        ensure!(
            metadata.is_file() && metadata.len() <= MAX_WORKER_EXECUTABLE_BYTES,
            "worker executable is not a bounded regular file"
        );
        let mut file = fs::File::open(&self.worker_executable)
            .context("could not open the worker executable")?;
        let mut digest = Sha256::new();
        let mut buffer = [0_u8; 64 * 1024];
        loop {
            let read = file
                .read(&mut buffer)
                .context("could not read worker bytes")?;
            if read == 0 {
                break;
            }
            digest.update(&buffer[..read]);
        }
        let observed = format!("{:x}", digest.finalize());
        ensure!(
            observed == self.worker_sha256.to_ascii_lowercase(),
            "worker executable digest does not match the run policy"
        );
        Ok(())
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
    OutputBudgetExceeded,
    UnobservedExit,
    DeadlineMismatch,
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
            Self::OutputBudgetExceeded => "output_budget_exceeded",
            Self::UnobservedExit => "unobserved_exit",
            Self::DeadlineMismatch => "deadline_mismatch",
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
    parent_sequence: u64,
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
        policy.verify_executable()?;
        let process = launcher.launch(&policy, prepared)?;
        let session = WorkerSession::new(policy.run_id.clone())?;
        Ok(Self {
            policy,
            session,
            process,
            lifecycle: WorkerLifecycle::Starting,
            ipc_bytes: 0,
            requests: 0,
            parent_sequence: 0,
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
        self.check_frame_deadline(frame)?;
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
        if let Err(error) = self.process.send_frame(frame) {
            self.mark_failed(WorkerTerminationReason::WorkerFailed);
            return Err(error).context("could not deliver frame to worker");
        }
        self.ipc_bytes += frame_bytes;
        self.requests += 1;
        self.parent_sequence = self
            .parent_sequence
            .checked_add(1)
            .context("parent sequence exhausted")?;
        self.enforce_output_budgets()?;
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
        self.enforce_output_budgets()?;
        let Some(frame) = frame else {
            return Ok(None);
        };
        if let Err(error) = self.check_frame_deadline(&frame) {
            self.shutdown(WorkerTerminationReason::DeadlineMismatch)?;
            return Err(error);
        }
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
            sequence: self
                .parent_sequence
                .checked_add(1)
                .context("parent sequence exhausted")?,
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

    fn check_frame_deadline(&self, frame: &WorkerFrame) -> Result<()> {
        self.policy.verify_frame_deadline(frame)
    }

    /// A pipe that is not drained can deadlock a child and let its output grow
    /// without bound. The platform backend reports observed byte counts; the
    /// parent terminates and reaps the worker when either ceiling is crossed.
    fn enforce_output_budgets(&mut self) -> Result<()> {
        let stdout = self.process.stdout_bytes();
        let stderr = self.process.stderr_bytes();
        if stdout > self.policy.max_stdout_bytes || stderr > self.policy.max_stderr_bytes {
            let _ = self.shutdown(WorkerTerminationReason::OutputBudgetExceeded);
            bail!("worker output budget exceeded");
        }
        Ok(())
    }

    fn finish(
        &mut self,
        mut exit: WorkerExit,
        reason: WorkerTerminationReason,
    ) -> Result<WorkerExit> {
        if exit.code.is_none() && exit.signal.is_none() {
            self.mark_failed(WorkerTerminationReason::UnobservedExit);
            bail!("worker exit has no OS-observed code or signal; run is indeterminate");
        }
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
        let observed = self
            .process
            .wait_reap(Instant::now() + Duration::from_secs(2))
            .is_ok_and(|exit| exit.code.is_some() || exit.signal.is_some());
        if observed {
            self.lifecycle = WorkerLifecycle::Reaped;
            self.termination_reason
                .get_or_insert(WorkerTerminationReason::ParentShutdown);
        } else {
            self.lifecycle = WorkerLifecycle::Failed;
            self.termination_reason = Some(WorkerTerminationReason::UnobservedExit);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::VecDeque,
        sync::{
            Arc, Mutex,
            atomic::{AtomicU64, Ordering},
        },
        time::{SystemTime, UNIX_EPOCH},
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
        stdout_bytes: usize,
        stderr_bytes: usize,
        unobserved_exit: bool,
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
            if !state.ignore_terminate && !state.unobserved_exit {
                state.exit = Some(WorkerExit::observed(None, Some(9), false));
            }
            Ok(())
        }

        fn wait_reap(&mut self, _deadline: Instant) -> Result<WorkerExit> {
            let mut state = self.state.lock().unwrap();
            state.reap_calls += 1;
            if state.unobserved_exit {
                return Ok(WorkerExit::observed(None, None, false));
            }
            Ok(state
                .exit
                .clone()
                .unwrap_or_else(|| WorkerExit::observed(Some(0), None, false)))
        }

        fn stdout_bytes(&self) -> usize {
            self.state.lock().unwrap().stdout_bytes
        }

        fn stderr_bytes(&self) -> usize {
            self.state.lock().unwrap().stderr_bytes
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

    static NONCE: AtomicU64 = AtomicU64::new(0);

    fn policy(deadline: Instant) -> WorkerRunPolicy {
        let scratch = std::env::temp_dir().join("rocksoul-worker-test");
        let sandbox = match SandboxPlatform::current() {
            SandboxPlatform::Windows => SandboxPolicy::windows_default(scratch),
            _ => SandboxPolicy::linux_default(scratch),
        };
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            ^ u128::from(NONCE.fetch_add(1, Ordering::Relaxed));
        let executable = std::env::temp_dir().join(format!("rocksoul-worker-fixture-{nonce:x}"));
        let bytes = b"rocksoul-worker-fixture";
        fs::write(&executable, bytes).unwrap();
        WorkerRunPolicy {
            run_id: "run-1".to_owned(),
            worker_executable: executable,
            worker_sha256: format!("{:x}", Sha256::digest(bytes)),
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

    #[test]
    fn executable_hash_is_verified_before_launch() {
        let state = Arc::new(Mutex::new(FakeState::default()));
        let mut mismatched = policy(Instant::now() + Duration::from_secs(30));
        mismatched.worker_sha256 = "0".repeat(64);
        let prepared = prepared(&mismatched);
        let launcher = FakeLauncher { state };
        assert!(WorkerSupervisor::start_with_launcher(mismatched, &prepared, &launcher).is_err());
    }

    #[test]
    fn frame_deadline_must_match_the_immutable_run_policy() {
        let state = Arc::new(Mutex::new(FakeState::default()));
        let mut tampered = frame(WorkerMessageKind::Execute, 1, "tampered");
        tampered.deadline_unix_ms += 1;
        let mut sending_supervisor = supervisor(
            Arc::clone(&state),
            policy(Instant::now() + Duration::from_secs(30)),
        );
        assert!(sending_supervisor.send(&tampered).is_err());
        assert!(state.lock().unwrap().sent.is_empty());

        let state = Arc::new(Mutex::new(FakeState::default()));
        state.lock().unwrap().incoming.push_back(tampered.clone());
        let mut receiving_supervisor = supervisor(
            Arc::clone(&state),
            policy(Instant::now() + Duration::from_secs(30)),
        );
        assert!(receiving_supervisor.receive().is_err());
        assert_eq!(
            receiving_supervisor.termination_reason(),
            Some(WorkerTerminationReason::DeadlineMismatch)
        );
    }

    #[test]
    fn output_budget_exceeded_terminates_and_reaps() {
        let state = Arc::new(Mutex::new(FakeState::default()));
        state.lock().unwrap().stdout_bytes = 65 * 1024;
        let mut supervisor = supervisor(
            Arc::clone(&state),
            policy(Instant::now() + Duration::from_secs(30)),
        );
        assert!(
            supervisor
                .send(&frame(WorkerMessageKind::Execute, 1, "execute"))
                .is_err()
        );
        assert_eq!(supervisor.lifecycle(), WorkerLifecycle::Reaped);
        assert_eq!(
            supervisor.termination_reason(),
            Some(WorkerTerminationReason::OutputBudgetExceeded)
        );
        let state = state.lock().unwrap();
        assert_eq!(state.terminate_calls, 1);
        assert_eq!(state.reap_calls, 1);
    }

    #[test]
    fn unobserved_exit_is_indeterminate_not_success() {
        let state = Arc::new(Mutex::new(FakeState::default()));
        state.lock().unwrap().unobserved_exit = true;
        let mut supervisor = supervisor(
            Arc::clone(&state),
            policy(Instant::now() + Duration::from_secs(30)),
        );
        assert!(
            supervisor
                .shutdown(WorkerTerminationReason::ParentShutdown)
                .is_err()
        );
        assert_eq!(supervisor.lifecycle(), WorkerLifecycle::Failed);
        assert_eq!(
            supervisor.termination_reason(),
            Some(WorkerTerminationReason::UnobservedExit)
        );
        assert!(supervisor.exit().is_none());
    }

    #[test]
    fn parent_sequence_advances_for_control_frames() {
        let state = Arc::new(Mutex::new(FakeState::default()));
        let mut supervisor = supervisor(
            Arc::clone(&state),
            policy(Instant::now() + Duration::from_secs(30)),
        );
        supervisor
            .send(&frame(WorkerMessageKind::Execute, 1, "execute"))
            .unwrap();
        let cancel = supervisor
            .control_frame(WorkerMessageKind::Cancel, "cancel")
            .unwrap();
        assert_eq!(cancel.sequence, 2);
        supervisor.send(&cancel).unwrap();
        let state = state.lock().unwrap();
        assert_eq!(state.sent.len(), 2);
        assert_ne!(state.sent[0].sequence, state.sent[1].sequence);
    }
}
