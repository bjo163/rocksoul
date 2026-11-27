//! Private, anonymous local IPC transport for a future isolated worker.
//!
//! This module creates a dedicated socket pair with no public listener, no
//! network endpoint, and no identity asserted by the peer. The child side is
//! exposed only as inherited standard handles so a platform launcher can bind
//! it to the OS-created process. It is a transport primitive, **not** an OS
//! sandbox: it does not isolate CPU, memory, filesystem, or process capabilities.
#[cfg(unix)]
mod platform {
    use std::{
        os::{fd::OwnedFd, unix::net::UnixStream},
        process::Stdio,
        time::Duration,
    };

    use anyhow::{Context, Result};

    use crate::worker_protocol::{WorkerFrame, read_frame, write_frame};

    pub struct ParentEndpoint {
        stream: UnixStream,
    }

    pub struct ChildEndpoints {
        stdin: Stdio,
        stdout: Stdio,
    }

    impl ChildEndpoints {
        pub fn into_stdio(self) -> (Stdio, Stdio) {
            (self.stdin, self.stdout)
        }
    }

    /// Create a fresh anonymous pair. Neither endpoint is addressable by path or
    /// port; only the owner of the child handle can reach the parent endpoint.
    pub fn create_private_pair() -> Result<(ParentEndpoint, ChildEndpoints)> {
        let (parent, child_read) =
            UnixStream::pair().context("could not create IPC socket pair")?;
        let child_write = child_read
            .try_clone()
            .context("could not clone the child IPC handle")?;
        let pair = ChildEndpoints {
            stdin: Stdio::from(OwnedFd::from(child_read)),
            stdout: Stdio::from(OwnedFd::from(child_write)),
        };
        parent
            .set_read_timeout(Some(Duration::from_secs(1)))
            .context("could not set IPC read timeout")?;
        Ok((ParentEndpoint { stream: parent }, pair))
    }

    /// Test-only pair whose two ends are both parent-style endpoints. Used to
    /// exercise framing without spawning a process.
    #[cfg(test)]
    pub fn test_pair() -> Result<(ParentEndpoint, ParentEndpoint)> {
        let (left, right) = UnixStream::pair().context("could not create test socket pair")?;
        left.set_read_timeout(Some(Duration::from_millis(50)))?;
        right.set_read_timeout(Some(Duration::from_millis(50)))?;
        Ok((
            ParentEndpoint { stream: left },
            ParentEndpoint { stream: right },
        ))
    }

    impl ParentEndpoint {
        pub fn send(&mut self, frame: &WorkerFrame) -> Result<()> {
            write_frame(&mut self.stream, frame)
        }

        pub fn receive(&mut self) -> Result<Option<WorkerFrame>> {
            read_frame(&mut self.stream)
        }

        pub fn set_read_timeout(&self, timeout: Duration) -> Result<()> {
            self.stream
                .set_read_timeout(Some(timeout))
                .context("could not set IPC read timeout")?;
            Ok(())
        }

        #[cfg(test)]
        pub fn write_raw(&mut self, bytes: &[u8]) -> Result<()> {
            use std::io::Write;
            self.stream
                .write_all(bytes)
                .context("could not write raw test bytes")?;
            Ok(())
        }
    }
}

#[cfg(not(unix))]
mod platform {
    use std::time::Duration;

    use anyhow::{Result, bail};

    use crate::worker_protocol::WorkerFrame;

    pub struct ParentEndpoint {
        _private: (),
    }

    pub struct ChildEndpoints {
        _private: (),
    }

    pub fn create_private_pair() -> Result<(ParentEndpoint, ChildEndpoints)> {
        bail!("private IPC transport is unavailable on this platform; refusing to start a worker")
    }

    impl ParentEndpoint {
        pub fn send(&mut self, _frame: &WorkerFrame) -> Result<()> {
            bail!("private IPC transport is unavailable on this platform")
        }

        pub fn receive(&mut self) -> Result<Option<WorkerFrame>> {
            bail!("private IPC transport is unavailable on this platform")
        }

        pub fn set_read_timeout(&self, _timeout: Duration) -> Result<()> {
            bail!("private IPC transport is unavailable on this platform")
        }
    }
}

#[cfg(all(test, unix))]
pub use platform::test_pair;
pub use platform::{ChildEndpoints, ParentEndpoint, create_private_pair};

#[cfg(test)]
mod tests {
    use std::time::Duration;

    #[cfg(unix)]
    use super::test_pair;
    use super::*;
    #[cfg(unix)]
    use crate::worker_protocol::{WORKER_PROTOCOL_VERSION, WorkerFrame, WorkerMessageKind};

    #[cfg(unix)]
    #[test]
    fn anonymous_pair_round_trips_a_validated_frame() {
        let (mut sender, mut receiver) = test_pair().unwrap();
        let frame = WorkerFrame {
            protocol_version: WORKER_PROTOCOL_VERSION,
            run_id: "run-ipc".to_owned(),
            request_id: "req-1".to_owned(),
            sequence: 1,
            kind: WorkerMessageKind::Execute,
            deadline_unix_ms: 1_800_000_000_000,
            payload: serde_json::json!({"tool":"fixture.state.read"}),
        };
        sender.send(&frame).unwrap();
        assert_eq!(receiver.receive().unwrap(), Some(frame));
    }

    #[cfg(unix)]
    #[test]
    fn separate_pairs_do_not_cross_talk() {
        let (mut first_parent, mut first_peer) = test_pair().unwrap();
        let (mut second_parent, _second_peer) = test_pair().unwrap();
        let frame = WorkerFrame {
            protocol_version: WORKER_PROTOCOL_VERSION,
            run_id: "run-a".to_owned(),
            request_id: "req-a".to_owned(),
            sequence: 1,
            kind: WorkerMessageKind::Execute,
            deadline_unix_ms: 1_800_000_000_000,
            payload: serde_json::json!({}),
        };
        first_parent.send(&frame).unwrap();
        second_parent
            .set_read_timeout(Duration::from_millis(10))
            .unwrap();
        assert!(second_parent.receive().is_err());
        first_parent
            .set_read_timeout(Duration::from_millis(10))
            .unwrap();
        assert_eq!(first_peer.receive().unwrap(), Some(frame));
    }

    #[cfg(unix)]
    #[test]
    fn multiple_frames_in_one_write_are_read_in_order() {
        let (mut sender, mut receiver) = test_pair().unwrap();
        let first = WorkerFrame {
            protocol_version: crate::worker_protocol::WORKER_PROTOCOL_VERSION,
            run_id: "run-ipc".to_owned(),
            request_id: "req-1".to_owned(),
            sequence: 1,
            kind: WorkerMessageKind::Heartbeat,
            deadline_unix_ms: 1_800_000_000_000,
            payload: serde_json::json!({}),
        };
        let second = WorkerFrame {
            request_id: "req-2".to_owned(),
            sequence: 2,
            ..first.clone()
        };
        let mut bytes = Vec::new();
        crate::worker_protocol::write_frame(&mut bytes, &first).unwrap();
        crate::worker_protocol::write_frame(&mut bytes, &second).unwrap();
        sender.write_raw(&bytes).unwrap();
        assert_eq!(receiver.receive().unwrap(), Some(first));
        assert_eq!(receiver.receive().unwrap(), Some(second));
    }

    #[cfg(unix)]
    #[test]
    fn partial_header_and_peer_close_are_distinct() {
        let (mut sender, mut receiver) = test_pair().unwrap();
        receiver
            .set_read_timeout(Duration::from_millis(10))
            .unwrap();
        sender.write_raw(&[0, 0]).unwrap();
        assert!(receiver.receive().is_err());
        drop(sender);
        assert_eq!(receiver.receive().unwrap(), None);
    }

    #[cfg(unix)]
    #[test]
    fn malformed_frame_errors_do_not_echo_payload() {
        let (mut sender, mut receiver) = test_pair().unwrap();
        let secret = "api_key=do-not-echo";
        let body = format!("{{\"secret\":\"{secret}\"}}");
        let length = u32::try_from(body.len()).unwrap();
        let mut bytes = length.to_be_bytes().to_vec();
        bytes.extend_from_slice(body.as_bytes());
        sender.write_raw(&bytes).unwrap();
        let error = format!("{:#}", receiver.receive().unwrap_err());
        assert!(error.contains("invalid worker frame"));
        assert!(!error.contains("do-not-echo"));
        assert!(!error.contains("api_key"));
    }

    /// Proves only the transport and reaping primitive: a real OS child owns
    /// the private inherited handles, frames round-trip, and `wait` observes
    /// termination. This child is NOT isolated and this test is never used by
    /// production dispatch.
    #[cfg(unix)]
    #[test]
    fn private_ipc_is_bound_to_an_os_child_and_can_be_reaped() {
        use std::process::{Command, Stdio};

        let echo = std::path::Path::new("/bin/cat");
        if !echo.exists() {
            eprintln!("skipping child-handle test: /bin/cat unavailable");
            return;
        }
        let (mut parent, child) = create_private_pair().unwrap();
        let (stdin, stdout) = child.into_stdio();
        let mut child_process = Command::new(echo)
            .stdin(stdin)
            .stdout(stdout)
            .stderr(Stdio::null())
            .spawn()
            .expect("could not spawn bounded echo child");
        parent.set_read_timeout(Duration::from_secs(2)).unwrap();

        let frame = WorkerFrame {
            protocol_version: crate::worker_protocol::WORKER_PROTOCOL_VERSION,
            run_id: "run-os-child".to_owned(),
            request_id: "req-echo".to_owned(),
            sequence: 1,
            kind: WorkerMessageKind::Heartbeat,
            deadline_unix_ms: 1_800_000_000_000,
            payload: serde_json::json!({"probe":"child-handle"}),
        };
        parent.send(&frame).unwrap();
        assert_eq!(parent.receive().unwrap(), Some(frame));

        child_process
            .kill()
            .expect("could not terminate echo child");
        let status = child_process.wait().expect("could not reap echo child");
        assert!(!status.success());
        assert!(
            status.code().is_none(),
            "killed child must have no exit code"
        );
    }

    #[cfg(unix)]
    #[test]
    fn child_endpoint_is_convertible_to_inherited_stdio() {
        let (_parent, child) = create_private_pair().unwrap();
        let (stdin, stdout) = child.into_stdio();
        let _ = (stdin, stdout);
    }

    #[cfg(not(unix))]
    #[test]
    fn transport_fails_closed_on_unsupported_platforms() {
        assert!(create_private_pair().is_err());
    }
}
