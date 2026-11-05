# Issue #35 — Crash-window recovery and generalized acceptance gates

**Status:** Execution specification prepared from `main` at `260d29852bb30f2f63129271a07cd368fafccc3f`.  
**Issue:** https://github.com/bjo163/rocksoul/issues/35  
**Owner:** Rocksoul core / workspace execution  
**Product boundary:** One Rust Cargo package, one portable native executable, CLI-first.

## Problem

The existing create-only workspace operation is idempotent and independently reads back the published artifact. The command then constructs a trusted `VerificationRequest`, runs the verifier, persists the RID profile (including XP), persists a kernel receipt, and appends a verified trace event.

The code already tests recovery when the RID XP profile has been persisted but the receipt file has not yet been written: replay must not award XP twice and can reconstruct the receipt.

The remaining crash window is earlier: **after the artifact is published and read back, but before XP is durably persisted**. The file may exist while the RID still has no corresponding XP event or durable verification receipt. Replay must recognize the exact same artifact, verify its current bytes again, then persist XP exactly once and write the receipt.

A second requirement is to make acceptance checks a reusable, typed, versioned contract for future bounded services without introducing a generic executor or widening tool permissions.

## Goals

1. Prove crash recovery after publication but before RID/XP persistence.
2. Prove replay of the exact operation is idempotent across artifact, XP ledger, receipt, and task trace.
3. Prove conflicting artifacts and malformed/conflicting receipts fail closed and are never silently overwritten.
4. Add deterministic offline evaluation coverage for the new recovery boundary.
5. Define a typed, versioned acceptance-check contract reusable by narrowly scoped future services.
6. Document exactly what is durable and what is not.

## Non-goals

- Persisting/resuming full planner, model conversation, or reflection history.
- Generalized arbitrary shell execution or unrestricted filesystem access.
- Network, browser, plugin, background, or remote tool execution.
- Automatic retry of non-transient failures.
- Automatic memory ingestion or memory context injection.
- Changing the current one-Cargo-package architecture.
- Claiming that a simulated crash test proves power-loss durability beyond the filesystem guarantees actually used.

## Required invariants

- The workspace operation ID is stable for the exact filename/content pair and remains the replay identity.
- Publication remains create-only. Existing identical bytes mean replay; conflicting bytes mean refusal.
- A receipt is issued only after fresh independent read-back evidence passes.
- XP is awarded only after trusted verification and remains idempotent by stable event ID.
- A crash before RID save leaves no durable XP award; replay can finish the same operation.
- A crash after RID save but before receipt persistence does not duplicate XP; replay reconstructs the receipt.
- Corrupt or conflicting durable receipts are not overwritten to manufacture success.
- Model reflection and tool narration are never acceptance evidence.
- Failpoints are test-only; do not add a production environment variable or user-facing crash switch.

## Acceptance criteria

- A deterministic test injects failure immediately after artifact publication/read-back and before verification/XP persistence.
- After the injected failure: artifact exists with exact bytes; persisted RID XP is unchanged; no verified receipt is published.
- Replaying the same approved operation returns the existing artifact, re-reads bytes, verifies the effect, persists one XP event and one receipt.
- Replaying again does not duplicate the artifact, XP event, or receipt.
- A different artifact at the same target is rejected without overwrite or XP.
- A malformed receipt and an immutable-field-conflicting receipt both fail closed and preserve the original bytes.
- Versioned acceptance-check contracts reject unsupported schema versions, duplicate check IDs, malformed required evidence, and invalid bounds.
- Offline evaluation reports the new crash/replay/conflict cases.
- Linux and Windows CI, release build, portable smoke, and installer smoke pass.
- README, architecture, cognitive core, roadmap, progress and handoff docs match the implementation.

## Evidence to record

Record the exact base SHA, branch/PR, test names, Linux/Windows CI run IDs, resulting merge SHA, issue closure, and any remaining durability limitations in `docs/PROGRESS.md`.
