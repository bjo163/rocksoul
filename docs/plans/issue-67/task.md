# Issue #67 — Tool Audit and Deadline Handoff

**Parent:** [Issue #13](https://github.com/bjo163/rocksoul/issues/13)  
**Implementation issue:** [Issue #67](https://github.com/bjo163/rocksoul/issues/67)  
**Target:** `main`  
**Status:** Execution contract only; no production tool execution is enabled by this plan.

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
