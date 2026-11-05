# Rocksoul Project Progress

**Last updated:** 2026-11-05 (Asia/Jakarta)  
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
| Stage 2 claim integrity | **Validation in progress** | PR #51 proposes schema v4, claim provenance, kernel receipt gate, revocation and supersession for issue #44 |
| Stage 3 — Skills/tools | **Not started** | No general tool registry or arbitrary tool execution |
| Stage 4 — Application builder | **Not started** | Future bounded project-generation workflow |
| Stage 5 — Evaluation/evolution | **Partial foundation** | Offline Core Integrity exists; candidate benchmarking, isolated evolution and promotion/rollback remain |
| Stage 6 — Multi-agent | **Not started** | Deliberately deferred until single-agent foundations justify delegation |
| Stage 7 — Additional interfaces | **Not started** | CLI remains primary; do not add web/desktop/server prematurely |

> A badge describes the progress snapshot, not a guarantee that a live workflow is currently green. Check the linked PR and CI before merging.

## 1. Accepted baseline on main

At the start of this progress update, the main SHA observed was:

`3489b9641083530c6e6d4a4f3bc554c17e10a638`

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
- Root-local versioned JSON memory and migrations through schema v3.
- Backup recovery and cross-process OS locking.
- Typed episodic, semantic, procedural, working and derived records.
- Retention/expiry, append-only content-free forgetting tombstones and managed backup/export scrubbing.
- Deterministic lexical retrieval, provenance filters and offline retrieval evaluation.
- Human-reviewed model proposals with sensitive-data guard, exact fingerprint approval and post-decision candidate scrubbing.

Still disabled by design:
- Automatic conversation capture.
- Silent model memory writes/deletes.
- Automatic memory context injection.
- Automatic training from conversation history.

## 2. Active pull request — #51

**Title:** `feat(memory): require kernel receipts for verified task outcomes`  
**URL:** https://github.com/bjo163/rocksoul/pull/51  
**Branch:** `feat/verified-task-outcome-claims`  
**Issue:** #44  
**Latest branch head observed during this update:** `1e2cd107b50bf55ac99f90b366eb54379169ce70`  
**Latest code fix:** `b29dc8136013ebf3bfcf1fbdbfef116b1aaf6e28`  
**Latest observed workflow for current branch head:** [run #42007687101](https://github.com/bjo163/rocksoul/actions/runs/42007687101)  
**State at snapshot:** PR open; current head's CI still running. Not merged.

Scope proposed:
1. Add structured claim statuses: user assertion, observed evidence, inference, unverified task outcome, verified task outcome, contradicted/superseded.
2. Migrate memory schema v3 to v4 without promoting old model proposals to verified status.
3. Bind exact normalized task-outcome claims and expected task/operation IDs to kernel-issued receipts; the expected ID must match the receipt's integrity-bound task ID.
4. Require a fresh (seven-day), exact, unused and non-revoked receipt.
5. Keep invalid/missing/stale/mismatched-claim/mismatched-task/tampered/replayed/revoked receipts unverified.
6. Keep ordinary user assertions receipt-free and explicitly labeled.
7. Add append-only claim-status history for correction/supersession.
8. Add append-only receipt revocation and downgrade revoked verified claims.
9. Expose claim status/receipt provenance in CLI and memory retrieval.
10. Migrate proposal store v1 to v2 without weakening exact human approval.

### CI diagnosis and current evidence

Earlier run #41982853154 failed on Linux and Windows because two receipt-gating tests remained unverified. Formatting and Clippy had passed.

Root cause identified: receipt lookup attempted to parse every file in the receipt directory, including OS lock files and unrelated non-receipt files. The lookup now accepts only `.json` receipt files. A second review added an expected task ID to each receipt-bearing proposal; the approval fingerprint binds it, and it must match the receipt's integrity-bound task ID and persisted operation ID. Regression tests cover a different task and modified operation metadata. The lookup fix was committed as `b29dc813...`; task-binding work followed in commits after it.

Observed after fix:
- Workflow run #42007315585 for code fix commit `b29dc813...`: Linux formatting, Clippy, tests, Core Integrity and memory retrieval evaluation all passed.
- Windows build and tests passed at the time of the last poll, but Windows packaging/portable/installer smoke steps were still running.
- Subsequent documentation commits triggered fresh CI. Latest branch head at this snapshot is `1e2cd107...`, with run #42007687101 still in progress. The latest-head run must be fully green before merge.

**Do not merge based only on the older green Linux run.** Check the workflow attached to the exact latest PR head and require every required job to complete successfully.

### PR #51 safety boundaries

- A model cannot mint receipts or call the trusted verifier.
- Receipt integrity digest is not a digital signature against a local actor with write access to the runtime root.
- A receipt proves only a bounded verification event, not broad real-world truth.
- One receipt cannot verify multiple claims.
- Revocation downgrades previously verified claims.
- Automatic conversation capture and memory context injection remain disabled.

## 3. Documentation updated in active branch

The PR branch now includes:
- `docs/AI_HANDOFF.md` — local-agent onboarding, architecture map, mandatory workflow, build/test commands, safety invariants, active PR state, next-task selection and known deferred work.
- `docs/PROGRESS.md` — this progress snapshot.
- `docs/ARCHITECTURE.md` — distinguishes merged schema v3 baseline from proposed schema v4 work.
- `docs/COGNITIVE_CORE.md` — distinguishes model-only planner/reflection from trusted verification and labels PR #51 as proposed until merged.
- `docs/ROADMAP.md` — corrected Stage 2 status and kept the claim-gate task incomplete until merged.
- `README.md` — links to the handoff and progress documents.

When PR #51 merges, refresh this file and the handoff document to record the actual merge SHA, CI run IDs, closed issue state, and next selected task. Do not leave the current pending status in docs after merge.

## 4. Next execution order

### Immediate

1. Poll workflow run #42007687101 and any newer workflow attached to the latest PR #51 head.
2. If Linux or Windows fails, inspect full job logs, isolate the root cause, add regression coverage, and push a focused fix.
3. If both required jobs pass on the latest head, inspect the final PR diff and mergeability.
4. Merge PR #51 into `main`.
5. Verify PR merged state, merge commit SHA, latest main contents and required CI.
6. Close issue #44 only after the merge is verified.
7. Update `docs/PROGRESS.md`, `docs/AI_HANDOFF.md`, README, architecture, cognitive core and roadmap so all say the same thing.

### After #51

Refresh all open issues and choose based on dependencies, not guessed issue numbers:

- **First reliability candidate:** issue #35 (crash-window recovery and generalized acceptance gates). Its scope is independent of memory injection and is already specified with deterministic offline tests.
- **Stage 2 parent:** issue #36 is the memory umbrella. Verify its checklist and close only when all child scopes and evidence are complete.
- **Evaluation:** issue #13 owns the AGI-oriented evaluation harness. Keep synthetic memory failure fixtures and capability evaluation there rather than creating a duplicate evaluation subsystem.
- **Do not start automatic memory injection** until a separate policy, privacy/contamination test matrix, forgetting/deletion behavior and regression gates are reviewed.

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
