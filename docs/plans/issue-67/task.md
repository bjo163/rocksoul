# Issue #67 — Tool Audit and Deadline Handoff

**Parent:** [Issue #13](https://github.com/bjo163/rocksoul/issues/13)  
**Implementation issue:** [Issue #67](https://github.com/bjo163/rocksoul/issues/67)  
**Target:** `main`  
**Status:** Implementation merged and validated on `main`; no production tool execution is enabled. PR #69 merged as `c845a9485a9b1b2882691c8dc268d6e6385b6be2`; exact-head CI #42211510844 passed Linux and Windows.

## Current verified boundary

The current `src/tool_registry.rs` is a deny-by-default metadata registry. It validates tool IDs, versions, capability/effect pairs, explicit allowlists, separate capability grants, and input-byte budgets. The fixture executor described in Issue #13 is deterministic in-process test infrastructure only. It has no shell, network, filesystem, process, environment-variable, or arbitrary-code capability. Its verifier reports `independent=false`.

Do not describe either component as a production executor or OS sandbox.

## Implementation sequence

### A — Typed audit contract

- Add a versioned `ToolAuditEvent` contract with stable event/run identifiers and monotonic per-run sequence.
- Use enums for event kind, authorization result, effect class, terminal outcome, and failure category.
- Include only bounded metadata: tool ID/version, capability, effect, policy version, timestamp, and opaque evidence references/digests.
- Explicitly exclude prompt text, model output, arguments, file content, environment values, credentials, and arbitrary free-form error strings.
- Validate identifiers, sequence, schema version, and all field ceilings before serialization.
- Add positive, malformed, oversized, unknown-version, duplicate-sequence, and redaction tests.

### B — Durable append store

- Reuse the repository's established portable file-locking and append/recovery patterns rather than inventing a parallel persistence mechanism.
- Store content-free, versioned records under the portable executable root.
- Serialize cross-process writers; define behavior for partial final records and malformed interior records.
- Fail closed and preserve original bytes on corruption. Never silently truncate or rewrite prior audit history.
- Test concurrent writers, process-exit lock release, partial-tail recovery policy, malformed records, and immutable prior bytes on Linux and Windows.

### C — Deadline and cancellation contract

- Represent an absolute per-run deadline and cooperative cancellation state at the host invocation boundary.
- Check authorization, deadline, cancellation, and remaining call budget immediately before dispatch.
- Distinguish `TimedOut`, `Cancelled`, `Denied`, `Failed`, `Indeterminate`, and `Verified`; only an independent verifier may produce `Verified`.
- A contract-level timeout is not process isolation. Do not claim a worker was stopped unless the parent actually terminates and reaps it.
- Do not introduce a production dispatch path in this slice.

### D — Integration and verification

- Integrate the audit contract only with existing bounded fixtures until the production sandbox in Issue #13 is complete.
- Keep raw arguments and sensitive content out of logs and test failure output.
- Run `cargo fmt --all -- --check`, `cargo clippy --locked --all-targets -- -D warnings`, `cargo test --locked --all-targets`, Core Integrity, memory retrieval, and the relevant fixture suite.
- Validate exact PR-head Linux and Windows workflows, including packaging/smoke gates.
- Update `docs/ROADMAP.md`, `docs/PROGRESS.md`, and `docs/AI_HANDOFF.md` only to reflect behavior that is implemented and validated on the PR head.

## Security invariants

1. Empty policy denies all tools.
2. Tool metadata is not execution authority.
3. Model output is never evidence of an effect.
4. Mutating capabilities remain separately granted and approval-gated.
5. No network, shell, subprocess, browser, plugin, or arbitrary filesystem access.
6. No unsandboxed fallback if a future worker backend is unavailable.
7. No XP/RID score from fixture evidence or self-report.
8. Production execution remains disabled until Issue #13's OS isolation and independent-verifier gates pass.

## Local-agent handoff

Start from the latest `origin/main`, inspect `AGENTS.md`, `src/tool_registry.rs`, the existing trace and locking implementations, and Issue #13's sandbox threat model and independent-verifier contract. Implement one coherent slice per PR. Do not stack on historical feature branches. Record exact tested SHA and Linux/Windows CI run IDs before merge.


## Completed implementation record — 2026-11-06

Implemented in `src/tool_audit.rs` and `src/fixture_runtime.rs`:
- Versioned content-free audit schema with strict event-specific metadata validation, safe bounded identifiers/evidence references, unknown-field rejection, and explicit `VerifierReported*` event names that cannot be confused with trusted attestations.
- Append-only JSONL storage with exclusive writer/shared reader OS locks, file/record ceilings, contiguous per-run sequencing, global event-ID uniqueness, and corruption/partial-tail fail-closed behavior that preserves existing bytes.
- Tests for schema/version/content rejection, identifier/field ceilings, event metadata consistency, duplicate/out-of-order sequence, malformed/partial history preservation, concurrent writers and cross-process appends.
- Cooperative invocation deadline, cancellation and atomic call-budget control.
- Audited fixture dispatch writes proposal/authorization/execution outcome events and checks the invocation control before dispatch. Denial, timeout, cancellation and exhausted budget are recorded without fixture-state mutation.
- No new runtime dependency.

Validation evidence:
- PR #69: https://github.com/bjo163/rocksoul/pull/69
- Validated head: `5151e47abfde528e95b4a0ded78e1469aacd7860`
- Merge commit: `c845a9485a9b1b2882691c8dc268d6e6385b6be2`
- Exact-head CI: https://github.com/bjo163/rocksoul/actions/runs/42211510844 — Linux and Windows passed.

## Remaining work — parent Issue #13

- Real OS-isolated worker process and WASM runtime selection under the Rust 1.85 MSRV gate.
- Parent-owned watchdog that terminates and reaps workers; cooperative cancellation alone is insufficient.
- Platform-specific resource/network/filesystem/process isolation on Linux and Windows.
- Independent verifier process/adapters and authenticated task-bound evidence.
- Model-connected held-out task runner, trajectory persistence, leakage controls and trusted RID publication gates.

Do not close Issue #13 or enable production tools based on this completed fixture/audit slice.
