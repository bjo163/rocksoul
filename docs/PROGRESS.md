# Rocksoul Project Progress

**Last updated:** 2026-11-05 (Asia/Jakarta, Issue #9 final cross-platform validation)  
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
| Stage 1 follow-through | **Merged and verified** | PR #53 merged as `458df81f`; issue #35 closed; CI #42044132321 passed on Linux and Windows |
| Stage 2 — Persistent memory | **Complete on main** | Schema v4, locking, retention/forgetting, evaluated retrieval and human-reviewed proposals; see PRs #43/#46/#47/#49/#50/#51 |
| Stage 2 claim integrity | **Merged** | PR #51 merged as `260d29852bb30f2f63129271a07cd368fafccc3f`; issue #44 closed; CI #42008862735 passed on Linux and Windows |
| Stage 3 — Skills/tools | **Not started** | No general tool registry or arbitrary tool execution |
| Stage 4 — Application builder | **Not started** | Future bounded project-generation workflow |
| Stage 5 — Evaluation/evolution | **Partial foundation** | Offline Core Integrity exists; candidate benchmarking, isolated evolution and promotion/rollback remain |
| Stage 6 — Multi-agent | **Not started** | Deliberately deferred until single-agent foundations justify delegation |
| Stage 7 — Additional interfaces | **Not started** | CLI remains primary; do not add web/desktop/server prematurely |

> A badge describes the progress snapshot, not a guarantee that a live workflow is currently green. Check the linked PR and CI before merging.

## Active implementation — Issue #9 Cognitive Cockpit

**Task branch:** feat/tui-cockpit-completion  
**Latest validated implementation/documentation head:** `e6cac1fbee4f99bbbdef3f7190161c0e9a8941f1`  
**Target:** main (no Cockpit completion changes merged yet)

Delivered on the task branch:
- Typed snapshot projections load bounded persisted task trace and memory store once; render functions consume read-only in-memory projections. Task history supports newest-first selection/detail and safe metadata-only rendering.
- Memory browser with local search, kind/claim/source filters, provenance/approval/expiry metadata, and explicit Enter-to-reveal through the shared sensitive-content guard.
- Snapshot-based memory store health, Core Integrity and memory retrieval report diagnostics, plus sanitized RID error details.
- Versioned portable TUI preferences for dark/light theme, compact/comfortable density and English/Indonesian locale; unknown or malformed preference files are preserved rather than overwritten.
- Semantic palette tokens applied to screens and overlays; locale applied to navigation, header, footer, settings and command palette.
- TestBackend coverage for all eight screens across 80×24, 120×40 and 60×20, both themes/locales, and palette/help overlays.
- Updated task plan and product/technical handoff documents.

Exact-head validation:
- CI [#42070438679](https://github.com/bjo163/rocksoul/actions/runs/42070438679) passed on `e6cac1fbee4f99bbbdef3f7190161c0e9a8941f1` for both Linux and Windows.
- Linux: `cargo fmt --check`, Clippy, 130 tests, Core Integrity 23/23, memory retrieval precision@2 1.000 / recall@2 1.000.
- Windows: release build, 129 tests, Core Integrity 22/22, memory retrieval precision@2 1.000 / recall@2 1.000, installer and portable ZIP packaging, clean portable smoke, installer install/init/uninstall-preservation smoke; artifacts uploaded.
- Full eight-file PR diff and safety boundaries reviewed. No model calls, tool dispatch, remote egress, task execution, memory writes/deletes or context injection were added to the Cockpit.

Remaining: merge PR #65 to main, verify main contains the merge SHA, and close Issue #9 with exact evidence.

## 1. Accepted baseline on main

Verified `main` SHA after PR #53 merge:

`458df81f87ae0b328dbc12c2b2829ca3a0ff75d9`

Refresh the remote ref before every new task; this is the handoff baseline, not a permanent pointer.

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

Issue #35 is complete. PR #53 merged to `main` as `458df81f87ae0b328dbc12c2b2829ca3a0ff75d9`; issue #35 is closed. The merged code contains a private test-only crash seam after real artifact read-back, exact replay recovery with one XP event/receipt, immutable receipt conflict and malformed-receipt preservation tests, bounded `AcceptanceCheckSet` schema v1, and Core Integrity v1.3.0 machine-readable checks. Full planner/reflection conversation resume and independent acceptance adapters for additional typed services remain deferred.

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

## 3. Completed implementation — Issue #35 / PR #53

**PR:** https://github.com/bjo163/rocksoul/pull/53 (merged)  
**Issue:** https://github.com/bjo163/rocksoul/issues/35 (closed)  
**Merge SHA:** `458df81f87ae0b328dbc12c2b2829ca3a0ff75d9`  
**Validated PR head:** `c9c605d6896930e5a36f30bb8d3fbe2ecfdd5484`  
**Final CI:** [#42044132321](https://github.com/bjo163/rocksoul/actions/runs/42044132321) — all Linux and Windows jobs passed on validated PR head `c9c605d6896930e5a36f30bb8d3fbe2ecfdd5484`. Linux: formatting, Clippy, 108 tests, Core Integrity 23/23 and memory retrieval (precision@2 1.000, recall@2 1.000). Windows: release build, 107 tests, installer/portable packaging, clean portable smoke, installer install/init/uninstall-preservation smoke, and artifact uploads passed. Squash merge verified on main as `458df81f87ae0b328dbc12c2b2829ca3a0ff75d9`; issue #35 closed.

Earlier CI diagnosis: run #42012045477 found a formatting issue, which was fixed. Run #42042937489 passed Linux but exposed a Windows-only collision in the existing memory v1 migration test: the helper named temporary roots with PID + timestamp, and Windows clock precision allowed parallel tests to reuse a path. `src/memory.rs` now adds a per-process atomic sequence to guarantee unique test roots. The final Windows run #42043396826 passed after that fix.

### Merged implementation

- `src/commands.rs`: shared workspace completion boundary, private test-only post-read-back hook, create-only receipt persistence exposed internally to offline evaluation.
- `src/core.rs`: bounded `AcceptanceCheckSet` schema v1 (max 64 checks, required-check presence, unique/non-empty IDs, bounded values and evidence references, unsupported-version rejection); validation occurs before XP mutation.
- `src/eval.rs`: Core Integrity v1.3.0 adds machine-readable checks for crash-after-publication/before-XP, replayed XP/receipt/trace idempotency, conflicting immutable receipts, and malformed receipt preservation.
- `src/commands.rs` tests: process-loss simulation with real temporary filesystem, reload from portable RID store, exact replay, one XP event, stable receipt, conflict and corruption preservation.
- `src/core.rs` tests: unsupported version, duplicate/blank IDs, missing evidence reference, optional-only/empty set, oversized values/references and more than 64 checks fail before XP.
- `docs/plans/issue-35/task.md`: audit, implementation, evaluation, documentation and CI/merge tasks recorded with final evidence; issue #35 closed.

### Deferred by design

- Full planner/turn/reflection conversation resume.
- Generalized execution or arbitrary tool dispatch; the contract only validates evidence supplied by trusted adapters.
- A deterministic temp-file/rename interruption hook; current receipt publication uses create-only temp + sync + hard-link, and no production failpoint or extra abstraction was introduced just for this optional subcase.
- Power-loss durability guarantees beyond the filesystem operations explicitly performed.

## 4. Documentation and next-task handoff

This documentation update reconciles:
- `docs/AI_HANDOFF.md` — local-agent onboarding, architecture map, mandatory workflow, build/test commands, safety invariants, completed PR state, next-task selection and known deferred work.
- `docs/PROGRESS.md` — this progress snapshot.
- `docs/plans/issue-35/` — completed execution specification, design notes, task breakdown, and agent safety rules.
- `docs/plans/issue-9/` — next TUI completion specification, design boundaries, autonomous task breakdown, and local-agent rules.
- `docs/ARCHITECTURE.md` — describes the merged schema v4 baseline and task-bound receipt policy.
- `docs/COGNITIVE_CORE.md` — distinguishes model-only planner/reflection from trusted verification and documents the merged claim-provenance gate.
- `docs/ROADMAP.md` — marks schema v4 claim provenance and task-bound receipt gating complete while keeping context injection gated.
- `README.md` — links to the handoff and progress documents.

This progress snapshot was refreshed after PR #53 merged and `main` was verified. Keep it synchronized with future merges, CI evidence, issue state and the next selected task.

## 5. Next execution order

### Immediate

1. Continue the active [Issue #9 — Cognitive Cockpit completion](https://github.com/bjo163/rocksoul/issues/9) work in [active draft PR](https://github.com/bjo163/rocksoul/pulls); refresh its head and CI before making changes.
2. Audit the existing eight-screen TUI before editing; preserve the one-core/read-only boundary and the current terminal lifecycle.
3. Implement real read models for persisted task trace, memory browsing, diagnostics/evaluation report, and truthful Brains/Tools status.
4. Add portable presentation preferences, accessibility/responsive tests, and correct the stale memory store path shown in the cockpit.
5. Run Linux and Windows CI, update docs, merge through a PR to main, verify main and close issue #9.
6. Keep arbitrary shell, generic tool execution, automatic memory capture and memory context injection out of scope.

### Current next task: #9 — Cognitive Cockpit completion

Issue #9 is actively implemented in active draft PR. Typed task/memory snapshot projections, guarded memory detail, diagnostics/evaluation reporting, portable preferences, semantic dark/light palettes, density, locale and responsive TestBackend coverage are on the branch but not yet merged. Final-head CI and merge gates remain open. The detailed plan is in `docs/plans/issue-9/`.

Issue #13 owns the broader AGI-oriented evaluation harness and explicitly depends on #4, #5, #8 and #9. Do not start or duplicate that harness before its dependencies are audited/closed. Issue #8's catalogue is implemented but still requires independent religious/language review; do not mark it complete through AI self-review. Stage 2 umbrella #36 and work packages #37–#41 plus follow-up #44 are closed. Do not start automatic memory injection until separate policy, privacy/contamination, forgetting/deletion and regression gates are reviewed.

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
