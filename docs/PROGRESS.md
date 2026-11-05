# Rocksoul Project Progress

**Last updated:** 2026-11-05 (Asia/Jakarta, after PR #51 merge)  
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
| Stage 1 follow-through | **Open** | Issue #35: crash injection after artifact publication/before XP persistence and generalized typed acceptance checks |
| Stage 2 — Persistent memory | **Core baseline merged** | Schema v3, locking, retention/forgetting, retrieval evaluation and human-reviewed proposals merged through PRs #43/#46/#47/#49/#50 |
| Stage 2 claim integrity | **Merged** | PR #51 merged as `260d29852bb30f2f63129271a07cd368fafccc3f`; issue #44 closed; CI #42008862735 passed on Linux and Windows |
| Stage 3 — Skills/tools | **Not started** | No general tool registry or arbitrary tool execution |
| Stage 4 — Application builder | **Not started** | Future bounded project-generation workflow |
| Stage 5 — Evaluation/evolution | **Partial foundation** | Offline Core Integrity exists; candidate benchmarking, isolated evolution and promotion/rollback remain |
| Stage 6 — Multi-agent | **Not started** | Deliberately deferred until single-agent foundations justify delegation |
| Stage 7 — Additional interfaces | **Not started** | CLI remains primary; do not add web/desktop/server prematurely |

> A badge describes the progress snapshot, not a guarantee that a live workflow is currently green. Check the linked PR and CI before merging.

## 1. Accepted baseline on main

Verified main SHA after PR #51 merge:

`260d29852bb30f2f63129271a07cd368fafccc3f`

Re-check latest main before using this as a base.

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

Remaining in issue #35:
- Crash injection after file publication but before XP persistence.
- Conflicting/corrupt durable receipt failure tests and evaluation coverage.
- Reusable typed/versioned acceptance-check contract for future services.
- Clear recovery boundary docs: operation/receipt/XP replay is durable; planner/reflection conversation history is not resumed.

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

## 3. Documentation updated in active branch

The PR branch now includes:
- `docs/AI_HANDOFF.md` — local-agent onboarding, architecture map, mandatory workflow, build/test commands, safety invariants, active PR state, next-task selection and known deferred work.
- `docs/PROGRESS.md` — this progress snapshot.
- `docs/ARCHITECTURE.md` — distinguishes merged schema v3 baseline from proposed schema v4 work.
- `docs/COGNITIVE_CORE.md` — distinguishes model-only planner/reflection from trusted verification and labels PR #51 as proposed until merged.
- `docs/ROADMAP.md` — corrected Stage 2 status and kept the claim-gate task incomplete until merged.
- `README.md` — links to the handoff and progress documents.

This progress snapshot was refreshed after PR #51 merged. Keep it synchronized with future merges, CI evidence, issue state and the next selected task.

## 4. Next execution order

### Immediate

1. Start issue #35 from the latest `main` SHA `260d29852bb30f2f63129271a07cd368fafccc3f` (refresh main first).
2. Add deterministic crash injection after artifact publication but before XP persistence; prove artifact and XP replay are idempotent and durable.
3. Add conflicting/corrupt receipt recovery fixtures and a typed, versioned acceptance-check contract for future services.
4. Update evaluation, architecture, cognitive-core, roadmap, README and AI handoff docs.
5. Run Linux and Windows CI, then merge through a PR to main.
6. Keep arbitrary shell, unrestricted tools, automatic memory capture and memory context injection out of scope.

### Current next task: #35

Issue #35 is the next selected implementation task. The Stage 2 memory umbrella #36 is ready to be reconciled and closed after updating its stale #41 checkbox and recording #44 as complete. Issue #13 remains the owner of the AGI-oriented evaluation harness; do not create a duplicate evaluation subsystem. Do not start automatic memory injection until a separate policy, privacy/contamination test matrix, forgetting/deletion behavior and regression gates are reviewed.

## 5. Release and merge gates

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
