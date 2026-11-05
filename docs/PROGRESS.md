# Rocksoul Project Progress

**Last updated:** 2026-11-06 (Asia/Jakarta, Universal 99 domains third-pass overlap reconciliation v2.3.3 published)  
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

## Completed implementation — Issue #9 Cognitive Cockpit

**Task branch:** feat/tui-cockpit-completion  
**Validated PR head:** `383d8cfa3998105d34f6897b93bc14bce5329a55`  
**PR:** [#65](https://github.com/bjo163/rocksoul/pull/65) — merged  
**Main merge SHA:** `6474cd130bd8146f03dcacb757d9c341d4b1161a`  
**Issue:** [#9](https://github.com/bjo163/rocksoul/issues/9) — closed

Merged to main:
- Typed snapshot projections load bounded persisted task trace and memory store once; render functions consume read-only in-memory projections. Task history supports newest-first selection/detail and safe metadata-only rendering.
- Memory browser with local search, kind/claim/source filters, provenance/approval/expiry metadata, and explicit Enter-to-reveal through the shared sensitive-content guard.
- Snapshot-based memory store health, Core Integrity and memory retrieval report diagnostics, plus sanitized RID error details.
- Versioned portable TUI preferences for dark/light theme, compact/comfortable density and English/Indonesian locale; unknown or malformed preference files are preserved rather than overwritten.
- Semantic palette tokens applied to screens and overlays; locale applied to navigation, header, footer, settings and command palette.
- TestBackend coverage for all eight screens across 80×24, 120×40 and 60×20, both themes/locales, and palette/help overlays.
- Updated task plan and product/technical handoff documents.

Exact-head validation:
- CI [#42071047960](https://github.com/bjo163/rocksoul/actions/runs/42071047960) passed on validated PR head `383d8cfa3998105d34f6897b93bc14bce5329a55` for both Linux and Windows.
- Linux: `cargo fmt --check`, Clippy, 130 tests, Core Integrity 23/23, memory retrieval precision@2 1.000 / recall@2 1.000.
- Windows: release build, 129 tests, Core Integrity 22/22, memory retrieval precision@2 1.000 / recall@2 1.000, installer and portable ZIP packaging, clean portable smoke, installer install/init/uninstall-preservation smoke; artifacts uploaded.
- Full 13-file PR diff and safety boundaries reviewed. No model calls, tool dispatch, remote egress, task execution, memory writes/deletes or context injection were added to the Cockpit.

Integration complete: PR #65 merged; `main` verified at `6474cd130bd8146f03dcacb757d9c341d4b1161a`; Issue #9 closed. No further Issue #9 work remains.

## 1. Accepted baseline on main

Verified `main` SHA after PR #65 merge:

`6474cd130bd8146f03dcacb757d9c341d4b1161a`

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

1. Keep Issue #9's merged Cockpit behavior stable; any follow-up must be a new, narrowly scoped issue.
2. Next dependency gate: [Issue #8](https://github.com/bjo163/rocksoul/issues/8) requires independent human religious/language review of the 99-entry catalogue.
3. Do not let an AI model self-approve religious/language correctness; record reviewer scope, findings, disagreements and accepted corrections.
4. After Issue #8 review is complete, reassess [Issue #13](https://github.com/bjo163/rocksoul/issues/13), whose evaluation-harness dependencies include #4, #5, #8 and #9.
5. Keep arbitrary shell, generic tool execution, automatic memory capture and memory context injection out of scope until separately designed and gated.

### Current next task: Issue #8 — independent catalogue review

Issue #9 is merged and closed. The next dependency is not another UI implementation: it is an independent human review of the existing 99-entry catalogue's religious and language presentation. AI self-review is not an acceptable substitute. Issue #13 must remain blocked until #4, #5, #8 and #9 are all audited and closed.

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


## RID Universal 99 Domains & Stats — third-pass overlap reconciliation v2.3.3 (runtime migration implemented; CI pending)

**Canonical taxonomy:** `data/rid/universal_stats.v2.json`  
**Version:** `2.3.3`  
**Second-pass taxonomy commit:** `653fc83cf2f087ee81431840fa48c459ff30710f`  
**Third-pass taxonomy commit:** `01a171d09fbea7aa8a278ecbccb8bf88277c22c8`  
**Runtime validator commit:** `af7d4d9c831f7960ad9516214dd0d6bcdcce6ada`  
**Human-readable audit:** [docs/rid-universal-stats.md](rid-universal-stats.md)

Delivered:
- Exactly 99 independent first-class domains and 99 linked stat slots: **99 capability dimensions, not 198**.
- Exactly 99 unique domain IDs, keys, labels, measurement targets, stat IDs, stat keys, stat labels, and domain-specific separation rules.
- **322 directed overlap guards / 161 reciprocal pairs**, plus 161 canonical pairwise contrast records (`rid_overlap:001`–`rid_overlap:161`).
- Second-pass review added 12 missing adjacency guards: READ/UNDERSTAND, READ/TRANSLATE, TRACK/DIALOGUE, RETAIN/ADAPT, LEARN/INTEGRATE, OPTIMIZE/ITERATE, COMPLETE/SELF_CHECK, EXPRESS/STRUCTURE, TRANSLATE/TUNE, BOUNDARY/SECURE, SECURE/RESIST, and GOAL_TRACK/HANDOFF.
- Each pair record names both constructs, both measurement targets, and an explicit separate-outcome rule; runtime validates exact graph/registry parity.
- Third-pass review added 12 additional adjacency guards: NOTICE/ERROR_DETECTION, TRACK/MONITOR, TRACK/GOAL_TRACK, UPDATE/SYNTHESIZE, ANALYZE/REFLECT, PRIORITIZE/ESCALATE, EXECUTE/TRANSFER, EXECUTE/MONITOR, ALIGN/VIEWPOINT_MODEL, ALIGN/INCLUDE, INVENT/REFRAME, and EXPERIMENT/REFLECT.
- All 99 domain separation rules state primary target, inclusion, exclusion, adjacent construct targets, and no-double-counting requirements.
- Runtime validator checks unique metadata, normalized keys, sorted/unique reciprocal adjacency, exact graph counts, pairwise contrast IDs and targets, domain/stat metadata parity, and the one-to-one measurement contract.
- Stat 42 remains RESUME (stable ID retained) to distinguish interruption recovery from long-run reliability.
- Runtime embeds v2.3.3; schema-v1 archival migration preserves exact original bytes, legacy records, identity, and XP while initializing all universal scores as unrated.
- Evaluation manifest v1.1.3 binds adapters to RID v2.3.3; benchmark families are navigation tags only, not additional scores.
- Religious inspiration data remains separate from technical capability taxonomy.

Audit conclusions and remaining gates:
- No duplicate IDs, keys, labels, primary measurement targets, or separation rules were found in the current 99-entry catalogue.
- The lexical screen is a candidate-discovery aid, not proof of independence. The second-pass and third-pass semantic reviews added 24 adjacency guards after reviewing candidate pairs that were not explicit in the prior graph.
- Correlation between capabilities is expected; the control is distinct measurement targets and non-reused outcome evidence, not a claim that capabilities are statistically independent.
- Structural validation and editorial boundary review do **not** constitute independent psychometric validation. Trusted evaluator design, held-out benchmark quality, inter-rater reliability, and empirical discriminant validity remain pending.
- Exact-head Linux/Windows CI must be green before claiming verified completion.
- Previous CI diagnosis and repairs: `cargo fmt --check` reported formatting drift in `commands.rs`, `rid.rs`, `rid_catalogue.rs`, `rid_store.rs`, and `tui.rs`; formatting is corrected. Windows tests exposed a fixture type mismatch (`Vec<&str>` vs `Vec<String>`), now fixed with owned strings. The next Windows run exposed `Access is denied` during legacy migration because a read-only archive handle was redundantly flushed after the staged file had already been synced; that read-only `sync_all` call is removed. The latest exact-main CI run is queued; do not mark it green until both Linux and Windows jobs finish.

Next:
1. Verify exact-head Linux and Windows CI for v2.3.3 and fix any failures.
2. Design trusted per-domain evaluator adapters with held-out benchmark suites, negative controls, calibrated confidence, and auditable evidence.
3. Run independent expert review and empirical discriminant-validity studies before treating scores as comparable capability measurements.
4. Keep all 99 dimensions `unrated` until trusted evidence exists; do not map historical Asmaul Husna scores by ordinal slot.