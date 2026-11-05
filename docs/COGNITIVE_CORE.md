# Cognitive Core Contracts

**Status:** Describes current implementation on `main`. The interactive planner is model-only; independent verification is currently a separate core contract and deterministic test path.

## Current interactive turn flow

The REPL routes each user message through a bounded planner, model-only step executor, and structured reflector. Model outputs are untrusted proposals.

1. Validate the turn request and initialize turn state, cancellation token, and budget meter.
2. Ask the configured model for a JSON `TurnPlan`.
3. Validate the plan: non-empty objective, unique step IDs, one to three ordered steps, and at least one required acceptance check.
4. Execute each step through a separate model call, preserving order and conversation context. These are reasoning calls—not shell commands, tools, or external actions.
5. Reject missing or empty step outputs.
6. Ask the model to reflect on the plan, required checks, and generated outputs.
7. Validate the structured reflection. A successful reflection completes the in-memory turn; failed or unresolved reflection fails it.
8. On successful completion, append the assistant response to the session-local conversation history.

The conversation history is not persisted across process sessions. The planner and reflector are model calls; the reflector is not independent evidence and can accept unsupported claims or miss errors.

## Budgets

Each turn permits at most five model calls: one planner, up to three step calls, and one reflector. The aggregate output-token ceiling is 16,000 and the wall-clock budget is 120 seconds.

The request's `max_tokens` is conservatively reserved as one fifth of the total budget per call. This keeps the sum of request caps within 16,000 even when a provider omits usage metadata. When usage is returned, the meter also checks the reported aggregate. The HTTP client has a 120-second timeout per request, and the turn meter checks the overall elapsed budget between calls.

## Lifecycle, cancellation, and retries

The core defines deterministic lifecycle phases and validates transitions, retry limits, terminal-state immutability, and budget enforcement. Ctrl+C cancels model-driven turns, including in-flight HTTP requests, cooperatively. Provider requests retry only connection/timeout failures and HTTP 408/425/429/5xx, at most three attempts; every attempt consumes the shared turn call budget. Workspace execution retries only transient filesystem I/O errors for at most three attempts with 20/40 ms backoff. A stable operation ID binds the exact filename/content pair and is reused for verifier XP idempotency. Ctrl+C interrupts async terminal approval and signals the blocking executor; filesystem syscalls are not forcibly interrupted, and cancellation is honored at safe pre-publication checkpoints. Automatic re-planning remains unimplemented.

## Independent verification and RID XP

The core exposes a verification contract that accepts evidence records, checks required expected/observed values, records evidence references, and gates XP awards. XP event IDs provide replay idempotency; reusing an ID with different event data is rejected. Failed required checks award no XP.

This verifier is not yet wired to the interactive planner's acceptance checks or to external tool execution. The deterministic Core Integrity suite exercises the verifier using synthetic test evidence. Therefore, a successful model reflection or ordinary chat turn does not award RID XP and must not be described as independently verified task completion.

## Memory and other runtime boundaries

Persistent local memory is implemented separately from the REPL:
- The memory store is schema v3 JSON under the executable root. Existing v1 and v2 stores are migrated on first load to v3 with per-record retention policy metadata; originals remain as rollback sources until an explicit forget operation requires scrubbing managed legacy snapshots.
- CLI commands provide explicit user-controlled add, list, ranked search with kind/source/time filters, export, delete, confirmed clear, and dry-run-first expiry pruning.
- Ranked retrieval is deterministic lexical scoring with source IDs, record kind, provenance, timestamps, source confidence (when supplied), and a plain-language relevance explanation. The numeric lexical score is not a relevance probability.
- The model cannot silently write or delete memory.
- User forgetting is recorded in an append-only `forgetting.jsonl` ledger containing record IDs, timestamps, and reason codes only—not deleted content. The ledger is applied during load and backup recovery; managed backup, legacy snapshot, and latest export files are scrubbed after forgetting.
- Expired records are excluded from active reads and exports. `memory prune-expired` previews candidates; `--apply` explicitly commits deletion. Ordinary file deletion is not forensic secure erasure; external copies, filesystem snapshots, and backups outside the managed root remain outside this guarantee.
- Conversation history is session-only; it is not automatically ingested into memory or injected from memory into prompts.
- Schema v1/v2-to-v3 migration, versioned retention metadata, an append-only content-free forgetting ledger, multi-process locking, deterministic lexical ranking, provenance filters, and offline retrieval evaluation are implemented. Tombstones are applied during load and backup recovery; managed backups, legacy snapshots, and latest exports are removed after a forget operation. Ordinary file deletion is not a forensic secure-erasure guarantee. CLI read-modify-write transactions use a stable OS lock file with a bounded five-second wait; network filesystems and storage layers that do not honor OS advisory locks are unsupported for concurrent access. Automatic memory context injection remains disabled; it requires separate policy approval and additional contamination/regression gates.

The current runtime exposes no shell executor, arbitrary function tools, plugin marketplace, or autonomous filesystem mutation. Model-only reasoning steps do not perform external actions. Self-modification and autonomous code execution are not enabled.

All configuration and runtime data paths resolve beneath the executable directory; no AppData or user-profile data path is intentionally used.

## Validation

Unit tests cover plan/reflection JSON parsing, fenced JSON, malformed output rejection, unique plan-step validation, lifecycle transitions, retry bounds, cancellation contracts, and budget enforcement. The offline Core Integrity suite additionally covers verifier gating, XP replay idempotency, RID catalogue invariants, level thresholds, and explicit-memory baseline.

Core Integrity suite v1.2 additionally exercises the Stage 1 workspace path offline: unsafe/mismatched approval rejection, approved create/read-back, verifier-gated XP, duplicate replay idempotency, pre-cancel, retry exhaustion/cancellation during backoff, real read-back mismatch rejection, versioned trace round-trip/corruption checks, and Unix target-symlink rejection. Failure-injection now covers real read-back mismatch, retry exhaustion, cancellation during retry backoff and at the pre-publication boundary, plus Windows target-symlink rejection when supported. Recovery is tested for the crash window after RID XP persistence but before receipt write: replay is idempotent and reconstructs the receipt without new XP. Crash injection immediately after file publication but before XP persistence remains outstanding.

CI runs formatting, locked dependency resolution, Clippy, tests, offline Core Integrity evaluation, and Windows release packaging.


## First approved workspace capability

The CLI exposes one narrow create-only workspace artifact operation:
- Only a flat ASCII `.txt` filename is accepted; content is limited to 16 KiB.
- The user sees the proposed filename and full content, then must type the exact `WRITE <filename>` phrase.
- The service rejects detected workspace/target symlinks, refuses overwrites, stages content in the workspace, publishes without replacing an existing target, and independently reopens the final file to compare bytes.
- Identical same-name/same-content replay is idempotent. Conflicting content is rejected.
- The trusted command path submits observed read-back checks to the RID verifier; only a verified receipt can add XP.
- Cooperative cancellation is checked before execution and immediately before publication. Ctrl+C cancels model-driven REPL turns between model calls; terminal workspace execution and blocking filesystem cancellation are not yet signal-wired.
- The REPL planner may now return an optional typed artifact intent, which is validated and passed to the same interactive approval command. The planner cannot approve or execute it. Model reflection remains non-evidence for the filesystem effect.
- The CLI appends content-free version-1 JSONL lifecycle events under `logs/task-trace.v1.jsonl`; `rocksoul workspace trace` displays them.
- Complete turn/reflection persistence and end-to-end planner orchestration remain unfinished. Verification receipts are persisted and inspectable; cross-process trace locking and bounded retry/backoff are implemented. Adversarial filesystem TOCTOU protection is not claimed.
