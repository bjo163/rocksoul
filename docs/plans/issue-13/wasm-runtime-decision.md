# WASM runtime decision (2026-11-27)

Status: **decision record, not implementation**. No WASM runtime dependency has been added.

## Decision

Do **not** add a WASM runtime under the current Rocksoul MSRV of Rust 1.85. The first production isolation boundary is the native, OS-isolated worker described in [the Linux dependency review](linux-sandbox-dependency-review.md). WASM fuel and memory limits are supplemental controls, never a substitute for process isolation.

## Reasons

- Wasmi 2.0 declares `rust-version = 1.86`; adopting it would silently raise the declared MSRV, which is prohibited without an explicit, reviewed policy change and impact analysis.
- Wasmtime tracks the latest three stable Rust releases and is therefore not compatible with a stable 1.85 contract without a deliberate policy decision.
- Older Wasmi versions are not automatically security-equivalent to currently maintained releases; choosing one requires its own version, license, advisory, and MSRV review.
- Issue #71 requires OS-enforced CPU/memory/PID limits and a no-WASI guest with no default imports. Those requirements cannot be satisfied by interpreter fuel alone.

## Conditions for revisiting

A separate PR may propose a runtime only if it includes:
1. exact crate version, license, maintenance and security/advisory review;
2. explicit MSRV decision (keep 1.85 with a compatible runtime, or approve a bump with impact analysis);
3. WASI disabled by default, zero default imports, explicit narrow capability shims only;
4. bounded module bytes, fuel, memory, stack, and instruction count as defense-in-depth on top of OS isolation;
5. exact-head Linux CI evidence; Windows work is explicitly out of scope for this decision cycle.

## Current state

`rocksoul.agent-trajectory` remains `planned`. No guest module execution, WASM adapter, or production tool dispatch exists.
