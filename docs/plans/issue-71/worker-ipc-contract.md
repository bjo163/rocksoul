# Issue #71 — bounded worker IPC contract

**Status:** Protocol foundation implemented on branch `feat/worker-ipc-contract`; OS sandbox and process supervisor are not implemented.
**MSRV:** Preserve Rust 1.85. No runtime dependency was added.
**Security posture:** Production tool dispatch remains disabled.

## Implemented slice

`src/worker_protocol.rs` defines:

- Protocol version 1 with a four-byte big-endian length-prefixed JSON frame.
- 64 KiB maximum encoded frame and 48 KiB maximum JSON payload.
- Closed envelope fields (unknown fields rejected), bounded run/request identifiers, positive sequence and deadline.
- Message kinds for hello, ready, execute, cancel, heartbeat, result, failure, shutdown and exit.
- Parent-owned `WorkerSession` enforcing run binding, contiguous sequence numbers and unique request IDs.
- Monotonic worker lifecycle transitions. A protocol message cannot claim that a process was reaped; only the OS supervisor may make that observation.
- Unit tests for round-trip framing, clean EOF versus truncated headers, oversized frames, unknown fields, unsupported versions, oversized payloads, invalid IDs, replay, ordering, run mismatch and lifecycle transitions.

## Explicit limitations

This slice is a protocol library, not a worker executable or supervisor. It does not create processes, authenticate IPC peers, enforce deadlines against a running process, set OS resource limits, isolate filesystem/network/process access, or prove process termination/reaping. The current session guard is in-memory and must be bound to an authenticated OS IPC channel by the future supervisor. Payloads are untrusted proposals and are not verifier attestations.

No WASM runtime was added. Wasmi 2.0's Rust 1.86 MSRV conflicts with the current Rust 1.85 package contract; no MSRV bump is authorized by this slice.

## Required next slices

1. Parent supervisor and authenticated local IPC transport with OS-observed process lifecycle.
2. Linux backend: user/mount/PID/network namespaces, no network route, seccomp, cgroup v2 CPU/memory/PID ceilings, read-only root and scoped scratch mount.
3. Windows backend: restricted token/AppContainer, Job Object resource ceilings and kill-on-close, explicit handle allowlist, process wait/reap.
4. Adversarial Linux/Windows integration tests proving no network, no ambient filesystem/credentials, resource ceilings, timeout kill and reap.
5. Runtime/MSRV and license/security review, then no-WASI guest runner with fuel/memory/module/stack ceilings.
6. Independent verifier process and authenticated task-bound evidence, outside the guest trust domain.

If a platform cannot establish every required control, it must report unsupported and refuse production execution. No unsandboxed fallback is permitted.
