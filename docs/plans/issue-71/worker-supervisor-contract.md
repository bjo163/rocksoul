# Issue #75 — Parent Worker Supervisor Contract

**Status:** Design contract only; no worker process supervisor or authenticated IPC transport is implemented by this document.  
**Parent:** [Issue #75](https://github.com/bjo163/rocksoul/issues/75), under [Issue #71](https://github.com/bjo163/rocksoul/issues/71) and [Issue #13](https://github.com/bjo163/rocksoul/issues/13).  
**Baseline:** Rust 1.85; bounded framing/session protocol in `src/worker_protocol.rs`; production dispatch disabled.

## 1. Security objective

The parent is the sole authority for worker creation, policy, time, cancellation, termination, process reaping, audit persistence, and task outcome. A worker is untrusted even after OS isolation is established. Protocol validity is not authorization, and a worker result is not verifier evidence.

A worker may start only when a platform backend has established and the parent has independently checked every mandatory isolation control. Missing, partially configured, or unobservable controls mean **unsupported/refuse start**. There is no in-process or unsandboxed fallback.

## 2. Trust and authority model

| Component | Authority | Forbidden authority |
|---|---|---|
| Parent supervisor | Create run policy, spawn, send/cancel, enforce deadline, terminate, reap, persist observations | Trust worker self-reported identity, exit, isolation, or success |
| OS isolation backend | Establish platform controls and return OS-owned process/handle observations | Relax policy or silently omit controls |
| Worker | Execute the exact bounded request received over its private channel | Grant capabilities, change policy, access verifier/persistence secrets, claim process reaping |
| Audit store | Append content-free lifecycle and outcome metadata | Persist raw prompts, tool arguments, secrets, or guest-controlled payload |
| Independent verifier | Observe the task effect in a separate trust domain | Trust worker stdout/result as ground truth |

The supervisor must not share verifier credentials, signing material, trust-store write access, or report/XP/RID persistence handles with the worker.

## 3. Immutable run policy

Create and validate a host-owned policy before process creation. It binds:

- protocol version, unique run ID, unique request/task-attempt ID, and source commit;
- exact worker executable path and expected digest/version;
- selected backend name/version and required isolation capability set;
- private IPC endpoint/handle identity and explicit inherited-handle allowlist;
- monotonic wall-clock deadline and startup/handshake deadline;
- CPU quota/time, memory, process-count, file-descriptor/handle, IPC, stdout/stderr, call-count and aggregate I/O ceilings;
- allowed tool IDs/capabilities and approval state;
- task-scoped scratch root, mount/ACL policy, and network policy;
- audit correlation ID and policy schema/version.

The worker cannot modify the policy. No environment-wide secrets, home directories, SSH agents, browser profiles, cloud credentials, or ambient host handles are passed through.

## 4. Supervisor lifecycle

Allowed lifecycle is parent-controlled:

`Created → Starting → Handshaking → Ready → Running → {Cancelling | Terminating} → Reaped`

Any state may move to `Failed` only with a typed reason. `Reaped` is terminal. A worker message can request cancellation or report intent, but cannot set `Terminating` or `Reaped`. Only the parent, after OS observation, records process termination and reaping.

Required operations:

- `prepare(policy) -> PreparedWorker`: validate all policy fields and backend support without spawning.
- `start(prepared) -> RunningWorker`: create private IPC, spawn under OS restrictions, verify process identity and isolation, then begin handshake.
- `send(request)`: parent-authorized, bounded, run-bound, sequence-checked request only.
- `receive()`: bounded frame read with monotonic deadline, direction validation, run/session validation, and byte accounting.
- `cancel(reason)`: signal cooperative cancellation, start a short bounded grace period, then terminate if the worker has not exited.
- `terminate(reason)`: invoke backend process-tree/job termination; never rely on a protocol `Shutdown` response.
- `wait_and_reap(deadline)`: wait on the OS process handle/PID, collect actual exit status, and prove the process/job is gone.
- `finish(observation)`: persist terminal audit metadata only after the OS observation is known.

Dropping a running supervisor handle must not detach the worker. The implementation must provide a reliable shutdown path that terminates and waits/reaps. If cleanup cannot be confirmed, return a hard failure/indeterminate outcome and keep production dispatch disabled.

## 5. IPC authentication and message direction

The current protocol module validates framing, version, size, IDs, sequence, replay and run binding. It does not authenticate a peer or define a complete directional state machine.

The transport must be private and created by the parent:
- Prefer anonymous/inherited pipe handles or a socket pair whose endpoints are created before spawn and whose unused ends are closed.
- If a named endpoint is unavoidable, use a random non-guessable name plus restrictive OS ACL/permissions, verify peer OS credentials/handle identity, and prevent other local users from connecting.
- Bind the accepted channel to the OS-created child process handle/PID obtained by the parent. Never trust a PID, username, or token claimed in JSON.
- Do not open a public TCP listener or use loopback TCP as an authentication substitute.
- Inherit only the exact IPC handles required. Set close-on-exec/non-inheritable defaults for every other handle.
- If the chosen transport cannot prove peer identity and channel ownership, backend capability is unsupported.

Define legal direction by state. For example, worker-to-parent may contain `Hello`, `Heartbeat`, `Result`, `Failed`, and `Exited`; parent-to-worker may contain `Ready`, `Execute`, `Cancel`, and `Shutdown`. Exact transitions must be encoded and tested; an unexpected direction or message is a protocol violation and triggers termination.

The parent checks run ID, request binding, sequence, message direction, monotonic deadline, payload size, request ceiling, and current lifecycle on every frame. A valid frame is still not authorization.

## 6. Time, cancellation, and cleanup

- Use `std::time::Instant` for elapsed deadlines. Unix wall-clock timestamps may be metadata only and must not drive timeout enforcement.
- Enforce separate startup, handshake, execution, cancellation-grace, and reap deadlines.
- Check deadlines while waiting for IPC and process exit; never await an unbounded read.
- On deadline, cancellation, malformed/replayed frame, unexpected message, output-budget exhaustion, parent shutdown, or audit failure: stop accepting work, close/disable request channel, terminate the entire process tree/job, and wait for OS-observed exit.
- A worker's `Exited` frame or self-reported exit code is advisory only.
- Reap is successful only when the OS wait primitive reports process exit and the platform backend confirms no surviving job/process descendants remain under the worker's job/container.
- If kill or reap times out/fails, mark the run failed/indeterminate, preserve the incident metadata, and refuse further production dispatch.

## 7. Output and resource bounds

Never buffer unbounded stdout/stderr. Prefer null/discard handles unless diagnostics are explicitly required. If captured, drain concurrently and cap each stream and total bytes; truncate content and persist only safe metadata/digests. IPC reads must reject oversized lengths before allocating the body.

Resource ceilings are enforced twice where appropriate:
- parent policy: wall clock, calls, bytes, steps, aggregate I/O;
- OS backend: CPU, memory, process count, open handles/file descriptors, filesystem/mount access, and network reachability.

Cooperative cancellation, a Rust timer, or a WASM fuel counter is not an OS resource limit.

## 8. Platform backend requirements

### Linux backend

Required: dedicated unprivileged identity; user/mount/PID/network namespaces; no network route; minimal read-only root and task-scoped scratch; `no_new_privs`; seccomp allowlist; cgroup v2 CPU/memory/PID limits. Verify namespace identities, credentials, cgroup membership, mount layout, and network absence from the parent. Do not assume rootless namespace support or cgroup delegation exists. Refuse start if any required feature is missing.

### Windows backend

Required: restricted token and/or AppContainer policy with no network capability; Job Object CPU/memory/process limits and kill-on-close; explicit handle inheritance allowlist; parent-owned process handle and wait/reap. Verify token/AppContainer state, job assignment, handle inheritance and process-tree termination. Refuse start if any required primitive is unavailable.

### Backend interface

The common supervisor consumes a narrow backend contract: capability probe, prepare, spawn, authenticated channel acquisition, verify isolation, terminate process tree, wait/reap, and report OS observations. Platform-specific code owns native handles and must not expose raw unrestricted handles to higher layers.

## 9. Audit and evidence

Emit content-free lifecycle events correlated to the existing `ToolAuditStore`: worker prepared, start denied, process started, handshake complete, request sent, cancellation requested, termination requested, OS exit observed, reaped, cleanup failed. Do not persist raw IPC payload, stdout/stderr, prompt, arguments, secrets, or environment.

The supervisor's process observation is execution evidence only. It is not an independent effect-verifier attestation and cannot award XP or RID scores. The verifier remains a separate process/trust domain under Issue #13.

## 10. Test contract

Use helper worker processes on actual Linux and Windows runners:
- exits cleanly; hangs; ignores cancellation; floods stdout/stderr; emits malformed, oversized, wrong-direction, wrong-run, replayed or out-of-order frames; crashes; spawns a child; attempts to inherit/read ambient environment or handles.
- Verify startup refusal when a required OS feature is absent.
- Verify timeout/cancel/protocol violation kills and reaps the process tree/job.
- Verify parent does not treat worker `Exited` as OS reaping.
- Verify cleanup failure cannot produce success or enable the next task.
- Verify output and IPC limits are enforced before unbounded allocation.
- Verify no network, ambient filesystem, credential, process-spawn, or unrelated handle capability is available by default.

Mock backend tests are useful for state-machine coverage but do not satisfy platform isolation acceptance. CI must exercise real OS primitives on Linux and Windows.

## 11. Dependency and MSRV gate

Keep Rust 1.85. Do not add a process/IPC/sandbox crate until its exact version, license, maintenance, advisory history, MSRV, target support, feature set, and platform behavior have been reviewed. Avoid adding a WASM runtime in this slice. Wasmi 2.x requires Rust 1.86 and remains out of scope absent an explicit MSRV policy change.

## 12. Completion gate

This document is design only. The issue is complete only when the parent supervisor, authenticated IPC, real Linux/Windows isolation, hard termination/reaping, adversarial tests, exact-head CI, and security review all pass. Production dispatch remains disabled until the separate independent verifier and Issue #13 held-out evaluation/publication gates pass.
