# Issue #13 — Production Sandbox Threat Model and Execution Plan

**Status:** Design baseline only. This document does not claim that a production sandbox exists.
**Scope:** A future model-dependent agent tool executor for Rocksoul.
**Current baseline:** deny-by-default metadata registry + bounded in-process fixture executor. The fixture executor is not an OS sandbox.

## 1. Security objective

Treat model output, tool arguments, task datasets, and third-party tool modules as untrusted input. A compromised or prompt-injected model must not gain ambient host authority. Tool execution must be impossible unless the exact tool, capability, arguments, resource budget, and execution backend are explicitly authorized.

The security goal is bounded damage and auditable effects, not a claim that WebAssembly or any one runtime is invulnerable.

## 2. Trust boundaries

1. **Model boundary:** model emits structured proposals only. It cannot grant permissions, verify its own effects, award XP, publish RID scores, or mark a task successful.
2. **Policy boundary:** a host-owned immutable run policy lists exact tool IDs, capability grants, argument/output ceilings, step count, and deadline. Empty policy denies everything.
3. **Guest boundary:** untrusted tool modules execute as WebAssembly with no WASI by default. Unknown imports fail module instantiation. No ambient filesystem, process, environment, clock, randomness, or network APIs.
4. **Host-call boundary:** only explicitly registered, typed host functions may be imported. Each call rechecks run policy, argument bounds, cancellation, remaining call budget, and approval state.
5. **OS boundary:** WebAssembly execution belongs in a dedicated low-privilege worker process. The process receives no inherited credentials/secrets and no network route. WASM is not a substitute for OS isolation.
6. **Verifier boundary:** effect verification runs outside the guest worker and outside the model's trust boundary. It reads independently observed host state through narrow read-only adapters and emits content-free evidence references/digests.
7. **Persistence boundary:** the parent process validates and persists versioned reports. The guest cannot write report files or alter the RID/XP ledger.

## 3. Deny-by-default rules

- No WASI, sockets, DNS, HTTP, shell, subprocess, dynamic library loading, registry access, or unrestricted host imports.
- No filesystem preopens. Any future file tool must be rooted to one explicit task-scoped directory, reject symlinks/reparse points, canonicalize paths, and enforce an allowlisted operation set.
- No network egress by default. Remote access is a separate capability, disabled in CI and requiring explicit user approval.
- No inherited API keys, environment variables, home directories, SSH agents, browser profiles, or credential stores.
- No automatic fallback to unsandboxed execution when an OS backend is unavailable. Fail closed.
- No model-provided success assertion counts as verifier evidence.
- No safety/privacy/security/permission violation may be offset by task score.

## 4. Proposed initial resource ceilings

These are conservative starting defaults for a first implementation, subject to measurement and security review:

| Resource | Initial ceiling | Enforcement owner |
|---|---:|---|
| Wasm module bytes | 4 MiB | Parent before module compilation |
| Guest linear memory | 64 MiB | Wasm store/memory limiter |
| Fuel | 10,000,000 units | Wasm interpreter |
| Wall-clock runtime | 10 seconds | Parent worker watchdog |
| Tool calls per task | 32 | Parent policy |
| Trajectory steps | 64 | Parent runner |
| Input bytes per call | 64 KiB | Host-call boundary |
| Output bytes per call | 64 KiB | Host-call boundary |
| Aggregate I/O | 1 MiB | Parent policy |
| Workspace | task-scoped scratch only | OS worker sandbox |

A fuel budget does not replace a wall-clock watchdog. A memory limiter does not replace OS process memory limits. A timeout must terminate and reap the worker, not merely stop awaiting its response.

## 5. OS worker requirements

### Windows
- Dedicated low-privilege worker identity or AppContainer with no network capability.
- Job Object limits for process count, memory, CPU time, and kill-on-close.
- Explicit inherited-handle allowlist; no inherited secret-bearing handles.
- Restrict worker to a task-scoped scratch directory.
- If required controls cannot be established, do not start the worker.

### Linux
- Dedicated unprivileged UID/GID.
- no_new_privs, mount/user/PID/network namespaces, read-only root, task-scoped writable mount.
- Seccomp allowlist and cgroup CPU/memory/PID limits.
- No network namespace route by default.
- If required controls cannot be established, do not start the worker.

### Cross-platform protocol
- Versioned, length-prefixed IPC with strict message size ceilings.
- Parent validates every request and response; worker cannot self-authorize.
- Parent owns cancellation, deadline, call budget, and process cleanup.
- Worker restart is a new run with new provenance; never silently resume untrusted process state.

## 6. WebAssembly runtime selection gate

Wasmi is a candidate lightweight interpreter with fuel metering and configurable store limits. However, the current Wasmi 2.0.0 package declares Rust 1.86 as its MSRV, while Rocksoul currently declares Rust 1.85. Do not add Wasmi 2.0.0 without an explicit MSRV decision. An older compatible Wasmi release must be evaluated for maintenance/security status and API compatibility; do not assume compatibility from its version number.

Even with a Wasm interpreter:
- Disable WASI and all unneeded proposals/features.
- Disallow guest start functions unless a reviewed ABI requires them.
- Register only exact host imports; unknown imports fail.
- Enforce module, memory, table, stack, fuel, input, output, and wall-clock limits.
- Pin and audit the interpreter version; fuzz module parsing and host-call ABI.
- Keep the worker OS sandbox because an interpreter vulnerability can compromise its host process.

## 7. Independent effect-verifier contract

A verifier must:
- Have a distinct identity and versioned rubric.
- Observe the effect from a source not controlled by the model or guest worker.
- Read only the task-scoped expected state.
- Emit pass/fail, bounded confidence, and evidence references/digests.
- Record verifier errors as inconclusive/fail-closed, never as success.
- Be unable to mutate workspace, memory, XP, RID, or task policy.

The current in-process fixture verifier deliberately reports independent=false; it is only a regression test for evidence flow.

## 8. Required adversarial tests before enabling execution

- Unknown import, WASI import, and unauthorized host function are rejected.
- Oversized/invalid module is rejected before instantiation.
- Guest infinite loop hits fuel limit and worker wall-clock watchdog.
- Memory/table/stack growth beyond limits traps or terminates safely.
- Malformed IPC, oversized input/output, duplicate request IDs, and replay are rejected.
- Path traversal, symlink/reparse-point escape, and writes outside task root are blocked.
- Guest cannot read process environment, credentials, home directory, or unrelated files.
- Guest cannot reach loopback, LAN, or internet when network is disabled.
- Cancellation kills and reaps the worker and leaves no partial unauthorized effect.
- Worker crash, timeout, malformed evidence, verifier failure, and cleanup failure never become task success.
- Windows and Linux isolation tests run on actual CI runners; mocked policy tests are not sufficient proof of OS isolation.

## 9. Implementation sequence

1. Decide Wasmi/MSRV compatibility and OS isolation backends; record exact crate versions and license/security review.
2. Implement worker protocol and OS sandbox bootstrap with a no-op guest; test that missing isolation support fails closed.
3. Add WASM interpreter with no WASI/imports, module limits, memory limits, fuel, and disabled start function.
4. Add a tiny versioned host ABI; each import maps to one registry descriptor and capability.
5. Add parent-owned watchdog, cancellation, process reap, and content-free audit events.
6. Add independent read-only verifier process/adapters for observed filesystem effects.
7. Add held-out task manifests, train/test separation, leakage controls, and task digests.
8. Connect the model planner to structured tool proposals only after the previous gates pass.
9. Persist complete trajectory reports, compare matched baselines, and require trusted evaluator gates before RID publication.

## 10. Completion gates

Production agent trajectory execution remains **planned** until all of the following are demonstrated:
- OS isolation is enforced on supported platforms, not merely configured in code.
- No ambient network/filesystem/process/secret access is available.
- Resource and time ceilings are enforced under adversarial tests.
- Independent verifiers observe effects outside the model/guest trust boundary.
- Held-out tasks and dataset provenance are reproducible and leakage-controlled.
- Reports carry exact source/model/tool/task/rubric provenance.
- Exact-head Linux and Windows CI pass.
- Trusted evaluator and RID publication rules are satisfied.

No fixture-only result, unit test, or single benchmark proves AGI, consciousness, or general intelligence.
