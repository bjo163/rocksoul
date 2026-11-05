# Issue #35 — Design notes

**Design status:** Proposed implementation direction. Validate against the latest `main` before coding.

## Current flow observed on main

Relevant code:
- `src/workspace_artifact.rs`: `WorkspaceArtifactProposal::operation_id`, exact `UserApproval`, create-only publication, independent final-path read-back, and `ArtifactOutcome::{Created, AlreadyPresent}`.
- `src/commands.rs`: workspace command orchestration, trusted `VerificationRequest`, RID save, receipt persistence, and task trace.
- `src/core.rs`: `VerificationEvidence`, `VerificationRequest`, `VerificationReceipt`, deterministic verification and XP event idempotency.
- `src/rid_store.rs`: portable RID profile persistence.
- `src/task_trace.rs`: content-free operation lifecycle records.
- `src/eval.rs`: offline Core Integrity evaluation.

Observed durability order:
1. User-approved create-only artifact publication and independent read-back.
2. Construct exact claim and required evidence.
3. Run the trusted verifier; the XP event is added to the in-memory RID profile.
4. Persist the RID profile.
5. Persist the verification receipt.
6. Append the verified task trace event.

Existing coverage includes the later crash window (RID saved, receipt absent). The missing test boundary is after step 1 and before step 3/4.

## Crash-injection seam

Prefer a private, test-only hook at the orchestration boundary immediately after successful publication/read-back and before verification begins. The hook should return a typed simulated-crash error in tests.

Do not:
- Add a public CLI flag or production environment variable for crash injection.
- Put a failpoint after XP is mutated in memory and then accidentally persist that mutated profile.
- Treat a trace event or model narration as proof of artifact state.
- Replace the real filesystem read-back with a mock success result.

The test should use a temporary portable root and real filesystem operations:
1. Persist an initial RID profile.
2. Approve one exact proposal.
3. Execute and publish the artifact; trigger the test failpoint.
4. Drop all in-memory state to model process loss.
5. Reload RID from disk and assert XP is unchanged.
6. Reconstruct the same operation and approval from the same exact proposal.
7. Replay through the normal service; expect `AlreadyPresent` with byte-for-byte read-back.
8. Run the trusted verifier with stable task/verification/event IDs.
9. Persist RID, then persist receipt and trace.
10. Replay again and assert the XP event is duplicate/idempotent and receipt comparison accepts only immutable equality.

## Receipt conflict behavior

The current receipt persistence helper uses a create-only destination and compares immutable receipt fields on replay while allowing verification time/receipt digest to differ as designed. Add direct tests for:
- Existing valid receipt with identical immutable fields: replay accepted.
- Existing valid but conflicting task/evidence/claim fields: fail; do not overwrite.
- Malformed/truncated JSON: fail; do not overwrite or silently replace.
- Temp-file/rename failure: preserve the prior receipt and report an actionable error.

Keep the receipt integrity limitation explicit: the digest detects mutation but is not a digital signature against a local actor with write access.

## Typed versioned acceptance-check contract

Before implementation, inspect whether `VerificationEvidence` already covers all reusable fields and whether a wrapper is sufficient. Prefer the smallest addition.

Candidate contract:
- `AcceptanceCheckSet` with explicit `schema_version` and a bounded list of typed checks.
- Each check has a stable unique ID, expected value, observed value, evidence reference, and required/optional flag.
- Validation rejects unsupported schema versions, duplicate/empty IDs, empty/oversized/control-character values, malformed evidence references, and a set with no required checks.
- `VerificationRequest` consumes a validated check set or a narrow conversion from it.
- Versioning describes the check data contract; it does not create a plugin or arbitrary execution framework.

Do not add service-specific execution callbacks, arbitrary predicates, scripting, network fetching, or generic tool dispatch. A future service supplies observed values through its own trusted adapter and then submits the typed evidence set to the same verifier.

## Durability claims

Document the actual guarantees precisely:
- Create-only artifact publication is replay-safe for an identical operation.
- RID XP is persisted independently of the receipt file.
- If the process stops before RID persistence, replay re-verifies the existing artifact and awards XP once.
- If it stops after RID persistence but before receipt persistence, replay recognizes the XP event as duplicate and reconstructs the receipt.
- Full model conversation, planner state, and reflection history are not resumed.
- Do not claim fsync/power-loss guarantees beyond the operations explicitly performed by the implementation.
