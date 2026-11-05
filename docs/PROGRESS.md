# Rocksoul Project Progress

**Last updated:** 2026-11-05 (Asia/Jakarta, PR #53 validation passed; merge pending)  
**Repository:** [bjo163/rocksoul](https://github.com/bjo163/rocksoul)  
**Canonical branch:** `main`  
**Current principle:** One Core. Infinite Evolution.

This is the durable progress snapshot for maintainers and delegated/local AI agents. It records what is merged, what is only proposed, what remains blocked, and the next safe actions. Always refresh GitHub state and latest `main` before execution.

## Executive status

| Area | State | Evidence / note |
|---|---|---|
| Product foundation | **In place** | One Rust Cargo package, one portable native executable, CLI/REPL and optional Ratatui cockpit |
| Stage 0 — Seed CLI | **Implemented** | Windows release build and automated installer/portable smoke tests; manual clean-device upgrade test remains |
| Stage 1 — Reliable cognition | **Vertical slice merged** | PR #34, merge commit `0f5ca4bc4d1c65322cb02963fcc23c53fa990f5d`; CI #248 passed |
| Stage 1 follow-through | **Implementation + CI complete; merge pending** | Issue #35 / PR #53: crash recovery, receipt integrity, versioned acceptance checks, Core Integrity v1.3.0; Linux and Windows CI #42043396826 passed |
| Stage 2 — Persistent memory | **Complete on main** | Schema v4, locking, retention/forgetting, evaluated retrieval and human-reviewed proposals; see PRs #43/#46/#47/#49/#50/#51 |
| Stage 2 claim integrity | **Merged** | PR #51 merged as `260d29852bb30f2f63129271a07cd368fafccc3f`; issue #44 closed; CI #42008862735 passed on Linux and Windows |
| Stage 3 — Skills/tools | **Not started** | No general tool registry or arbitrary tool execution |
| Stage 4 — Application builder | **Not started** | Future bounded project-generation workflow |
| Stage 5 — Evaluation/evolution | **Partial foundation** | Offline Core Integrity exists; candidate benchmarking, isolated evolution and promotion/rollback remain |
| Stage 6 — Multi-agent | **Not started** | Deliberately deferred until single-agent foundations justify delegation |
| Stage 7 — Additional interfaces | **Not started** | CLI remains primary; do not add web/desktop/server prematurely |

> A badge describes the progress snapshot, not a guarantee that a live workflow is currently green. Check the linked PR and CI before merging.

## 1. Accepted baseline on main

Verified `main` base SHA used for the current Issue #35 branch:

`b76bf36740056e41933f8510c22a8376f49d997a`

This is the PR #53 base, not a promise that `main` has not advanced since. Refresh the remote ref before every new task.

### Stage 0 — Seed CLI

Delivered:
- One Rust Cargo package and one native executable (Rust 1.85 MSRV).
- CLI/REPL, OpenAI-compatible model adapter, local init/status/doctor.
- Portable executable-root configuration and runtime storage.
- RID identity, age, XP ledger, deterministic levels and 99 unrated technical stat slots.
- Ratatui Cognitive Cockpit.
- Offline Core Integrity evaluation.
- Windows x64 release packaging and clean-runner installer/portable smoke tests.

Remaining:
- Manual clean-device Windows installation and upgrade/data-preservation verification.

### Stage 1 — Reliable cognition loop

PR #34 is merged. The bounded workspace-artifact slice includes:
- Structured plan and reflection contracts; model-only reasoning steps.
- A single typed create-only text artifact operation.
- Exact interactive human approval.
- Independent read-back verification and persisted receipt.
- Bounded retry/cancellation behavior and content-free JSONL traces.
- XP award only after trusted verification, with replay idempotency.

Issue #35 implementation is in draft PR #53. The branch now contains a private test-only crash seam after real artifact read-back, exact replay recovery with one XP event/receipt, immutable receipt conflict and malformed-receipt preservation tests, bounded `AcceptanceCheckSet` schema v1, and Core Integrity v1.3.0 machine-readable checks. Full planner/reflection conversation resume and independent acceptance adapters for additional typed services remain deferred. Do not call #35 complete until fresh Linux and Windows CI pass, PR #53 merges, `main` is verified, and the issue is closed.

### Stage 2 — Persistent memory

Merged baseline:
- Root-local versioned JSON memory and migrations through schema v4 (v1/v2/v3 -> v4).
- Backup recovery and cross-process OS locking.
- Typed episodic, semantic, procedural, working and derived records.
- Retention/expiry, append-only content-free forgetting tombstones and managed backup/export scrubbing.
- Deterministic lexical retrieval, provenance filters and offline retrieval evaluation.
- Human-reviewed model proposals with sensitive-data guard, exact fingerprint approval and post-decision candidate scrubbing.
- Structured claim provenance and exact task-bound receipt verification; fresh single-use receipts are required for verified task outcomes, revocation downgrades claims, and correction history is append-only.

Still disabled by design:
- Automatic conversation capture.
- Silent model memory writes/deletes.
- Automatic memory context injection.
- Automatic training from conversation history.

## 2. Completed pull request — #51

**PR:** https://github.com/bjo163/rocksoul/pull/51  
**Issue:** #44 — closed after merge  
**Merge commit:** `260d29852bb30f2f63129271a07cd368fafccc3f`  
**Validated PR head:** `817c651f23e3e74bd59c25c94585a7e2d3bf108b`  
**CI:** [run #42008862735](https://github.com/bjo163/rocksoul/actions/runs/42008862735) — all Linux and Windows jobs succeeded.

Delivered:
1. Schema v4 claim provenance and v1/v2/v3 migration without promoting legacy model proposals.
2. User assertion, observed evidence, inference, unverified task outcome, verified task outcome and contradicted/superseded statuses.
3. Exact normalized claim and expected task/operation ID bound to a kernel-issued receipt.
4. Receipt-bearing proposals require both receipt ID and expected task ID; the ID is bound by the proposal fingerprint and must match the receipt's task ID and persisted operation ID.
5. Seven-day freshness, one-time use, revocation ledger and downgrade of revoked claims.
6. Append-only correction/supersession history.
7. CLI/search visibility for claim status and receipt provenance.
8. Proposal store v1-to-v2 migration preserving exact human approval.

CI evidence:
- Linux formatting and Clippy passed.
- 105 Linux tests passed.
- Offline Core Integrity and memory retrieval evaluation passed.
- Windows release build and tests passed.
- Windows installer/portable packaging, clean portable smoke test, and installer install/init/uninstall-preservation smoke test passed.

The receipt digest is tamper-evident integrity metadata, not a digital signature against a local actor with write access to the executable root. Automatic conversation capture and memory context injection remain disabled.

## 3. Active implementation — Issue #35 / PR #53

**PR:** https://github.com/bjo163/rocksoul/pull/53 (ready for review; not merged)  
**Issue:** https://github.com/bjo163/rocksoul/issues/35 (open)  
**Base SHA:** `b76bf36740056e41933f8510c22a8376f49d997a`  
**Latest branch head at this checkpoint:** `937b986fb54a0db82b3e3a2ee8519aaeee1fa2ad`  
**Final CI:** [#42043396826](https://github.com/bjo163/rocksoul/actions/runs/42043396826) — all Linux and Windows jobs passed on source head `8a0876fe9a3020477f52de5bab7451754acc8f62`. Linux: formatting, Clippy, 108 tests, Core Integrity 23/23 and memory retrieval (precision@2 1.000, recall@2 1.000). Windows: release build, 107 tests, installer/portable packaging, clean portable smoke, installer install/init/uninstall-preservation smoke, and artifact uploads passed. The final `937b986f` branch commit only updates task-plan validation evidence; it does not change source code.

Earlier CI diagnosis: run #42012045477 found a formatting issue, which was fixed. Run #42042937489 passed Linux but exposed a Windows-only collision in the existing memory v1 migration test: the helper named temporary roots with PID + timestamp, and Windows clock precision allowed parallel tests to reuse a path. `src/memory.rs` now adds a per-process atomic sequence to guarantee unique test roots. The final Windows run #42043396826 passed after that fix.

### Implemented on the PR branch (not yet accepted by main)

- `src/commands.rs`: shared workspace completion boundary, private test-only post-read-back hook, create-only receipt persistence exposed internally to offline evaluation.
- `src/core.rs`: bounded `AcceptanceCheckSet` schema v1 (max 64 checks, required-check presence, unique/non-empty IDs, bounded values and evidence references, unsupported-version rejection); validation occurs before XP mutation.
- `src/eval.rs`: Core Integrity v1.3.0 adds machine-readable checks for crash-after-publication/before-XP, replayed XP/receipt/trace idempotency, conflicting immutable receipts, and malformed receipt preservation.
- `src/commands.rs` tests: process-loss simulation with real temporary filesystem, reload from portable RID store, exact replay, one XP event, stable receipt, conflict and corruption preservation.
- `src/core.rs` tests: unsupported version, duplicate/blank IDs, missing evidence reference, optional-only/empty set, oversized values/references and more than 64 checks fail before XP.
- `docs/plans/issue-35/task.md`: audit and implementation phases A–D plus evaluation checks E1–E2 recorded; documentation and final CI/merge tasks remain in progress.

### Deferred by design

- Full planner/turn/reflection conversation resume.
- Generalized execution or arbitrary tool dispatch; the contract only validates evidence supplied by trusted adapters.
- A deterministic temp-file/rename interruption hook; current receipt publication uses create-only temp + sync + hard-link, and no production failpoint or extra abstraction was introduced just for this optional subcase.
- Power-loss durability guarantees beyond the filesystem operations explicitly performed.

## 4. Documentation included in this update

This documentation update reconciles:
- `docs/AI_HANDOFF.md` — local-agent onboarding, architecture map, mandatory workflow, build/test commands, safety invariants, active PR state, next-task selection and known deferred work.
- `docs/PROGRESS.md` — this progress snapshot.
- `docs/plans/issue-35/` — execution specification, design notes, small autonomous task breakdown, and agent safety rules for the next issue.
- `docs/ARCHITECTURE.md` — describes the merged schema v4 baseline and task-bound receipt policy.
- `docs/COGNITIVE_CORE.md` — distinguishes model-only planner/reflection from trusted verification and documents the merged claim-provenance gate.
- `docs/ROADMAP.md` — marks schema v4 claim provenance and task-bound receipt gating complete while keeping context injection gated.
- `README.md` — links to the handoff and progress documents.

This progress snapshot was refreshed after PR #51 merged. Keep it synchronized with future merges, CI evidence, issue state and the next selected task.

## 5. Next execution order

### Immediate

1. Continue the existing [PR #53](https://github.com/bjo163/rocksoul/pull/53); do not create a duplicate branch or PR.
2. Re-run full CI on the latest PR head after the formatting and evaluation/documentation commits.
3. Fix any Linux/Windows failures, inspect the full diff and review threads, and verify Core Integrity v1.3.0 reports all new checks.
4. Update the PR body with exact current head and CI evidence; mark ready only after checks pass.
5. Merge to `main`, verify the resulting main SHA, then close issue #35 with the evidence.
6. Keep arbitrary shell, unrestricted tools, automatic memory capture and memory context injection out of scope.

### Current task: finish PR #53 / issue #35

Issue #35 is already in progress through PR #53. Stage 2 umbrella issue #36 and its work packages #37–#41 plus follow-up #44 are closed. Issue #13 remains the owner of the AGI-oriented evaluation harness; do not create a duplicate evaluation subsystem. Do not start automatic memory injection until a separate policy, privacy/contamination test matrix, forgetting/deletion behavior and regression gates are reviewed.

## 6. Release and merge gates

A feature is complete only when:
- The code implements the documented behavior.
- Tests cover success, invalid input, failure, replay, migration/corruption and relevant platform behavior.
- Linux formatting, Clippy, full tests, Core Integrity and memory retrieval evaluation pass.
- Windows release build, full tests, portable packaging/smoke and installer smoke pass.
- Docs and roadmap match the merged implementation.
- PR is merged into main and main is verified.
- Related issue is closed with evidence.
- Next task has small, executable acceptance criteria.

Never label an in-progress workflow as passed. Never close an issue because a branch or PR merely exists.
