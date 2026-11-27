# Repository Consolidation Audit — 2026-11-27

## Scope and snapshot

Repository: `bjo163/rocksoul`  
Canonical branch: `main`  
Audited main SHA: `f2eb06390bb9bf89c8146cc3f4159357ae846fa3`  
Last observed main commit: 2026-11-14 03:07:37 UTC  
Audit date: 2026-11-27

This is a GitHub metadata, pull-request, branch-ref, workflow, and canonical-documentation audit. It is not a local clean-room build or a complete file-by-file semantic comparison of every surviving ref.

## Executive result

- `main` is the repository default branch and is explicitly defined by `AGENTS.md` as the only canonical source of truth.
- The repository has 52 branch refs including `main` (51 non-main refs).
- The PR history contains 54 PRs: 32 merged, 22 closed without merge, and zero open PRs.
- The latest observed CI run for the audited `main` SHA is run [#43549977459](https://github.com/bjo163/rocksoul/actions/runs/43549977459). Both Linux quality checks and Windows build/package/smoke tests completed successfully.
- The matching Release workflow run [#43549977462](https://github.com/bjo163/rocksoul/actions/runs/43549977462) was skipped, not failed.
- The project is not currently ready to enable production tool dispatch. The open sandbox and independent-verifier gates remain explicit blockers.

## Canonical source and workflow policy

The current `AGENTS.md` policy is correct and should remain authoritative:

1. Start every task from the latest `main` SHA.
2. Use a short-lived branch based on that exact SHA and submit one coherent PR targeting `main`.
3. Never merge an old branch wholesale merely because it appears ahead. Squash merges and historical branch divergence make commit counts insufficient evidence.
4. Reintroduce useful old work only by comparing actual files/behavior and selectively porting still-needed changes onto a fresh branch from current `main`.
5. Closed, unmerged PRs are historical proposals, not accepted product behavior.
6. Do not use release/feature branches as alternate sources of truth.

No open PRs currently need re-targeting or conflict resolution.

## Pull-request classification

### Merged PRs — 32

`#14, #17, #19, #20, #21, #22, #23, #24, #25, #27, #34, #43, #46, #47, #48, #49, #50, #51, #52, #53, #54, #65, #68, #69, #70, #72, #74, #76, #77, #95, #96, #97`

These merges are accepted history on `main`. Their surviving source refs are cleanup candidates, not work to merge again. Before removing a ref, confirm it is not used by a release/recovery process and preserve the PR history. A merged PR alone is not evidence that unrelated later commits on a similarly named ref are safe to discard.

### Closed without merge — 22

`#4, #6, #7, #10, #11, #12, #15, #16, #42, #55, #56, #57, #58, #59, #60, #61, #62, #63, #64, #73, #78, #94`

Do not reopen or merge this set wholesale.

Notable clusters:

- **Cognitive Cockpit:** repeated incremental attempts `#55–#64` used the same `feat/tui-cockpit-completion` head. The accepted completion is PR [#65](https://github.com/bjo163/rocksoul/pull/65), which is merged. The older attempts remain historical.
- **RID/core foundations:** `#4, #6, #7, #11` were closed without merge and some used other feature branches as their base. Current RID definitions and runtime implementation must be taken from `main`, not these historical branch snapshots.
- **Planner/evaluation:** `#15` and `#16` were closed without merge. Reconcile their ideas against current Issue [#13](https://github.com/bjo163/rocksoul/issues/13), the current evaluation manifest, and implemented evaluation code before reintroducing anything.
- **Documentation handoffs:** `#42, #73, #78, #94` were superseded or left unmerged. PR [#97](https://github.com/bjo163/rocksoul/pull/97) is the latest merged worker-protocol roadmap/handoff reconciliation.

## Branch-ref cleanup assessment

The surviving refs are materially more numerous than active work requires.

### Merged-source refs

The source refs for the 32 merged PRs listed above are candidates for eventual deletion after the repository owner verifies release/recovery dependencies. They should not be used as task bases. No source ref was deleted during this audit.

### Unmerged and duplicate refs

Preserve pending content-level review:

- Historical Cargo lockfile attempts: `chore/commit-cargo-lock`, `chore/commit-cargo-lock-41858824534`, `chore/commit-cargo-lock-41967686279`, and `chore/commit-cargo-lock-42123805396`. The accepted lockfile policy/change is already represented by merged PRs #24 and #25; these residual refs must be checked for unique file changes before cleanup.
- Memory outcome naming overlap: `feat/memory-verified-task-outcome-claims` versus merged `feat/verified-task-outcome-claims` (PR #51).
- Reviewed-memory proposal naming overlap: `feat/stage-2e-reviewed-memory-proposals` versus merged `feat/stage-2e-human-reviewed-memory-proposals` (PR #50).
- Worker/RID/documentation branch variants associated with closed-unmerged PRs must remain historical until their actual tree changes are checked.

### `release-production`

PR #17 that introduced this branch was merged, but the surviving ref is still present. GitHub's graph comparison reports it as diverged from current `main` (5 commits ahead and 214 behind). The comparison also reports differences across application, data, installer, workflow, and documentation files. Because the merge history is not a reliable content-equivalence test, do **not** merge this branch into `main`, use it as a release base, or delete it based on graph counts alone. Compare the exact tree/content against current `main` and any release tags first.

## Current implementation and open work

The canonical README and roadmap describe a Rust 1.85, edition-2024, single-package application. The CI workflow checks formatting, locked Clippy/tests, deterministic Core Integrity and memory-retrieval evaluation on Linux, and Windows release packaging plus clean portable/installer smoke tests.

Priorities from the current open issue set:

1. **Worker supervisor and authenticated IPC — Issue #75.** PR #72 provides bounded protocol framing/session contracts; it is not a production OS sandbox. Parent-owned start/cancel/terminate/wait/reap behavior, authenticated OS-bound IPC, and process-tree termination proof remain required.
2. **OS isolation and WASM gate — Issue #71.** Linux and Windows enforcement, negative tests, and fail-closed behavior remain required. No in-process fixture executor is a sandbox.
3. **Independent evaluation/verifier — Issue #13.** This is a hard gate before production dispatch. Do not describe the project as AGI; the roadmap treats AGI as a research direction.
4. **Request-grounded authorization — Issue #92.** PR #95 implements the first deterministic contract slice (typed authority/payload/control roles, lexical request grounding, immutable write-set commitment, and adversarial tests). The issue remains open correctly: semantic ambiguity/negation, explicit confirmation grants/receipts, audit and schema integration, broader attacks, and benign-utility metrics remain. Production dispatch stays disabled.
5. **Universal 99 taxonomy — Issue #5.** The v2.3.3 catalogue and migration are present on `main`; the issue remains open for exact-head validation and trusted-evaluator/measurement acceptance. Do not score stats from smoke tests.
6. **Asmaul Husna reference review — Issue #8.** This is separate from technical RID scoring and requires qualified human Arabic/source/language review. AI must not self-approve it.

The open issue list also contains dated Research Radar records. Treat them as research evidence/input, not as proof that an implementation task is complete. Promote a research finding into implementation only through a scoped issue and a main-based PR.

## Documentation drift

`docs/PROGRESS.md` says it was last updated on 2026-11-06, while the audited `main` includes later merges through 2026-11-14, including worker protocol/security and handoff updates (#72, #77, #95, #96, #97). Refresh that snapshot in a separate, focused docs PR and cite exact main commits/CI runs. Do not rewrite historical implementation evidence as though an unmerged proposal shipped.

## Repository safeguards

- Branch metadata reported `protected: false` for `main`. The connected integration could not retrieve the detailed branch-protection endpoint (HTTP 403), so ruleset/required-review configuration was not independently verified.
- Repository owner should verify GitHub Settings manually and enable protection for `main`: block force pushes/deletion, require PRs, and require the current Linux and Windows CI checks before merge.
- Keep release creation tied to a verified `main` commit. The Release workflow being skipped on an ordinary push is expected and is not a CI failure.
- Keep Rust MSRV 1.85 until an explicit compatibility review approves a change.

## Recommended consolidation sequence

1. Treat audited `main` SHA `f2eb0639` as the baseline for this report; refresh from remote before any new task.
2. Update `docs/PROGRESS.md` in a focused PR to reconcile the November 14 merges and the current open gates.
3. Continue #75/#71 in small, main-based PRs; require real Linux and Windows adversarial process-isolation evidence.
4. Keep #13 as the independent-verifier/evaluation gate before production dispatch.
5. Reconcile the remaining duplicate/orphan refs by exact file/tree comparison. Delete only refs with no unique work and no release/recovery dependency.
6. Triage Research Radar issues separately from implementation issues.
7. Verify and enforce branch protection on `main`.

## Audit limitations

This audit verified GitHub branch names/count, PR states/merge timestamps, selected branch comparisons, current-main files, and latest observed main CI jobs. It did not execute a fresh local Rust build, inspect every historical branch's complete tree, validate release artifact provenance, or independently confirm repository branch rulesets. No old branch was merged, force-updated, or deleted as part of this audit.
