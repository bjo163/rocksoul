# Linux sandbox dependency review (proposal only, 2026-11-27)

Status: **review document**. No dependency has been added. `Cargo.lock` is unchanged and production dispatch remains disabled.

## Decision under review

Use a single, exact-pinned low-level dependency for the Linux isolation backend:

```toml
[target.'cfg(target_os = "linux")'.dependencies]
libc = { version = "=0.2.191", default-features = false, features = ["std"] }
```

Rationale: the required primitives (user/mount/PID/network namespaces, `setns`, `unshare`, `clone3`, `seccomp` BPF, `prctl(PR_SET_NO_NEW_PRIVS)`, credentials and process reaping) are kernel interfaces. `std` does not expose them. Avoiding a broad sandbox framework keeps the attack surface small and keeps the supervisor, not a third-party runtime, in control of the lifecycle.

## Crate evidence (crates.io API, checked 2026-11-27)

| Field | Value |
|---|---|
| Crate | `libc` |
| Proposed exact version | `=0.2.191` (stable; `1.0.0-alpha.*` and `max_version` alpha are explicitly rejected) |
| Published | 2026-10-16 |
| License | `MIT OR Apache-2.0` |
| Declared `rust-version` | `1.65` — compatible with Rocksoul MSRV 1.85 |
| Edition | 2021 |
| Maintenance | `rust-lang/libc`, 2,075,521,078 lifetime downloads, 526,564,354 recent downloads (crates.io metadata) |
| Features used | `std` only; no `extra_traits`, no `use_std`, no internal rustc features |
| Target support | Linux is a first-class target; dependency is `cfg(target_os = "linux")` only |
| Supply-chain metadata | crates.io trust-public data present for 0.2.191; checksum published |

Security/advisory check: crates.io metadata lists no yanked status and no recorded advisory actions for the proposed version. A reviewed PR must still run `cargo audit`/`cargo deny` equivalent checks in CI before the dependency is accepted. No claim of "audited" is made here.

## Why not other crates

- `nix`: convenience wrappers add surface area without adding kernel guarantees; `libc` is sufficient and smaller.
- `seccomp` crates: a minimal BPF allowlist can be built with `libc` constants plus a small internal module; a separate parser dependency is not justified for a fixed syscall set.
- `wasmi`/`wasmtime`: see the separate WASM decision; neither is part of this backend.

## Required implementation gates before any code

1. A reviewed PR updates `Cargo.toml` and `Cargo.lock` together with `--locked` commands.
2. `SandboxBackend::prepare` must independently verify namespace membership, no-new-privs, seccomp filter installation, cgroup v2 delegation, read-only root, and absence of a network route. Any missing control refuses start.
3. Worker start remains gated by `WorkerSupervisor::start`; no unsandboxed fallback, no inherited secrets, no ambient network.
4. Adversarial Linux tests: namespace/cgroup credential checks, denied filesystem and network access, child-spawn attempt, process-tree escape, timeout/cancel kill and OS-observed reaping.
5. Production tool dispatch stays disabled until Issue #13's independent verifier and held-out evaluation gates also pass.

## Current state

The repository contains only the fail-closed policy contract (`src/sandbox.rs`), supervisor contract (`src/worker_supervisor.rs`), and anonymous transport (`src/ipc_transport.rs`). No platform backend exists. This document does not authorize adding the dependency by itself.
