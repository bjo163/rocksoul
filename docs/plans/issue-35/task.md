# Issue #35 — Autonomous task breakdown

**Base:** `main` at `b76bf36740056e41933f8510c22a8376f49d997a` (verified before implementation).  
**Branch:** `feat/stage1-crash-window-recovery`  
**PR base:** `main` only.

## Phase A — Audit and test map

- [x] A1. Refresh latest main and record exact SHA; verify no newer PR/issue supersedes #35.
- [x] A2. Read AGENTS, blueprint, this plan, issue #35, workspace artifact service, command orchestration, RID store, core verifier, task trace, and eval.
- [x] A3. Map current operation ID, XP event ID, receipt path, trace events, and replay branches.
- [x] A4. Identify the existing test for crash after RID save but before receipt write; preserve it.
- [x] A5. Write down the precise new failure boundary before changing code.

## Phase B — Crash-window recovery

- [x] B1. Add a private deterministic test failpoint after artifact publication and independent read-back but before trusted verification/XP mutation.
- [x] B2. Ensure the failpoint is unreachable from production CLI/config/environment.
- [x] B3. Add a real-filesystem test that injects the crash and confirms the artifact exists with exact bytes.
- [x] B4. Drop in-memory objects, reload RID from the portable root, and prove no XP event was persisted by the failed attempt.
- [x] B5. Replay the same operation ID and exact approval; assert the artifact is returned as AlreadyPresent and read back again.
- [x] B6. Verify the replayed observed evidence through the trusted core verifier and persist the RID.
- [x] B7. Persist the receipt and verified trace; assert one XP event and one receipt.
- [x] B8. Replay a third time; assert no duplicate artifact, XP event, or receipt.
- [x] B9. Keep the existing crash-after-RID-before-receipt recovery test green.

## Phase C — Durable receipt conflict handling

- [x] C1. Add a test for identical immutable receipt replay.
- [x] C2. Add a test for conflicting immutable receipt fields; fail closed and preserve the existing receipt bytes.
- [x] C3. Add a test for malformed/truncated receipt JSON; fail closed and preserve the corrupt bytes for diagnosis.
- [x] C4. Add a test for interrupted temp-file/rename publication if the current helper has a deterministic injection seam.
- [x] C5. Confirm no recovery path silently overwrites a corrupt or conflicting durable receipt.

## Phase D — Typed, versioned acceptance contract

- [x] D1. Audit `VerificationEvidence` and `VerificationRequest`; avoid duplicating fields already represented.
- [x] D2. Define the smallest versioned acceptance-check set contract (schema version + typed checks).
- [x] D3. Validate schema version, unique check IDs, bounds, required-check presence, and evidence references.
- [x] D4. Convert the validated contract into the existing verifier input without adding generic execution.
- [x] D5. Add positive, invalid-version, duplicate-ID, empty-required-set, oversized-value, and missing-evidence tests.
- [x] D6. Keep workspace artifact verification as the first adapter using the shared contract.

## Phase E — Evaluation and documentation

- Evaluation suite is now v1.3.0 and reports dedicated crash-window, replay/idempotency, immutable receipt-conflict, and malformed-receipt check IDs.
- The temp-file/rename interruption subcase (C4) was reviewed: the current create-only hard-link publication has no existing deterministic seam for injecting an interrupted temp publication. No production failpoint or extra abstraction was added; this subcase is deferred unless a future refactor introduces a suitable private seam.

- [x] E1. Add offline Core Integrity checks for crash-after-publication, replayed XP idempotency, and receipt conflicts/corruption.
- [x] E2. Include machine-readable check IDs and concise explanations in the evaluation report.
- [ ] E3. Update COGNITIVE_CORE recovery semantics and distinguish tested guarantees from deferred conversation resume.
- [ ] E4. Update ARCHITECTURE, ROADMAP, README, PROGRESS, and AI_HANDOFF.
- [ ] E5. Update this plan with actual source paths, tests, evidence and remaining limitations.

## Phase F — Verification and merge

- First PR CI run #42012045477: Windows release build/tests/package/smoke passed; Linux job stopped at `cargo fmt --check` due to an extra blank line in `src/workspace_artifact.rs`. The formatting issue was corrected, and additional acceptance-bound tests plus Core Integrity coverage were added afterward. A fresh full CI run is required for the current head.

- [ ] F1. Run `cargo fmt --all -- --check`.
- [ ] F2. Run `cargo clippy --locked --all-targets -- -D warnings`.
- [ ] F3. Run `cargo test --locked --all-targets`.
- [ ] F4. Run `cargo run --locked -- eval run core`.
- [ ] F5. Verify CI Linux formatting/Clippy/tests/Core Integrity/memory retrieval.
- [ ] F6. Verify CI Windows build/tests/portable packaging/installer smoke.
- [ ] F7. Inspect the complete diff and ensure no arbitrary tools, environment failpoints, or unrelated refactors.
- [ ] F8. Open a PR targeting main with base SHA, behavior, safety implications, test evidence and recovery limitations.
- [ ] F9. Merge only after required checks pass; verify main SHA and close #35 with evidence.

## Completion output

Update `docs/PROGRESS.md` and `docs/AI_HANDOFF.md` with PR URL, merge SHA, CI run IDs, test count, issue closure, and next task. Do not mark a task complete before it is merged and main is verified.
