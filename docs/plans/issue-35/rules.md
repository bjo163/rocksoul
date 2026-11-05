# Issue #35 — Agent execution rules

These rules supplement `AGENTS.md`; they do not override it.

## Source-of-truth rules

- Latest `main` is the only accepted baseline. The base SHA in this plan is historical context; refresh before coding.
- Do not reuse PR #51 or PR #52 branches. Start a new short-lived branch from current main.
- Do not stack branches. Target PRs at `main` only.
- Inspect current issue #35, open PRs, CI and docs before editing. Avoid duplicate or superseded work.

## Safety and architecture rules

- Keep one Cargo package and one native Rust executable.
- Preserve Rust 1.85 MSRV and locked dependency reproducibility.
- No arbitrary shell, unrestricted filesystem, network/browser/plugin access, background autonomy, self-modification, or generic tool executor.
- The only current side effect is the explicitly approved create-only workspace artifact operation.
- A model response, reflection, trace message, or tool narration is not evidence that an artifact exists.
- Only trusted Rust code may construct verifier evidence from independently observed filesystem state.
- XP is gated on verified observed effects and uses stable idempotent event IDs.
- Do not award XP before verification; do not persist an in-memory XP mutation after a simulated crash.
- Do not overwrite conflicting artifacts, corrupt receipts, or unknown schema versions.
- Test failpoints must be private/test-only. No production CLI flag, environment variable, config option, or public crash injection endpoint.

## Test rules

- Use deterministic temporary directories and real filesystem read-back.
- No network, model provider, wall-clock sleep, or flaky timing dependencies in the offline test path.
- Test both first execution and replay from reloaded on-disk state.
- Test negative paths: conflicting artifact, malformed receipt, conflicting receipt, unsupported acceptance schema, duplicate check IDs, missing required evidence.
- Preserve existing cancellation, retry, symlink, approval, trace and receipt replay tests.
- Run Linux and Windows CI. A local green run is not sufficient to merge.

## Documentation rules

- Update code comments and docs to state exact durability boundaries.
- Explicitly state that conversation/planner/reflection resume is not implemented.
- Do not claim power-loss durability beyond tested filesystem and sync behavior.
- Update roadmap checkboxes only after tests pass and merge is verified.
- Record exact PR, merge SHA, CI run IDs, and issue closure in progress/handoff docs.

## Stop conditions

Stop and report a blocker instead of broadening scope if:
- The recovery behavior requires unrestricted execution or hidden background work.
- A typed acceptance contract cannot be validated without speculative generic dispatch.
- A platform cannot provide the claimed atomicity/locking behavior.
- A receipt/artifact conflict would require silent overwrite or ambiguous authority.
