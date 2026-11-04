# Rocksoul Roadmap

The roadmap is capability-driven. Each stage must deliver a working, tested increment. The stages are not promises of AGI.

## Stage 0 — Seed CLI

- [x] One Rust Cargo project and native executable.
- [x] Interactive CLI with help, status, clear, and exit.
- [x] OpenAI-compatible chat-completions adapter.
- [x] Local bootstrap and diagnostics commands.
- [x] Windows release build in CI.
- [ ] Generate and commit the dependency lockfile after the first successful CI resolution.
- [ ] Verify release artifact on a clean Windows machine.

## Stage 1 — Reliable cognition loop

- [x] Define explicit request, response, and turn state contracts.
- [x] Add validated structured planning and reflection contracts.
- [x] Add model-call and elapsed-time budgets with explicit lifecycle errors.
- [x] Add provider-reported completion-token accounting when usage is available.
- [x] Keep model-independent core interfaces.
- [x] Add deterministic tests around state transitions, budgets, and validation.
- [ ] Generate plans with a dedicated model planning pass and execute bounded plan steps.
- [ ] Generate a separate reflection pass and evaluate acceptance checks against runtime evidence.
- [ ] Wire cancellation to terminal input/signals and implement retry backoff/idempotency.
- [ ] Enforce output-token budgets for providers that do not return usage metadata (requires a documented estimator or provider-specific tokenizer).

Current status: the REPL is integrated with a single-action turn (one model call, deterministic non-empty-response verification). This is a foundation, not the complete multi-step cognition loop.

## Stage 2 — Persistent memory

- Add a local durable store with versioned schemas.
- Separate session context, user-approved facts, working memory, and derived knowledge.
- Support inspection, export, deletion, and migrations.
- Make memory retrieval observable and testable.

## Stage 3 — Skills and tools

- Define a typed tool contract and capability registry.
- Start with read-only, narrowly scoped tools.
- Add explicit permission checks, audit events, and timeouts.
- Keep external runtimes optional rather than core dependencies.

## Stage 4 — Application builder

- Create projects from explicit templates.
- Work only inside an explicitly selected workspace.
- Track generated files and changes.
- Run formatters, compilers, tests, and project-specific checks.
- Report what changed and what was not completed.

## Stage 5 — Isolated evolution

- Create candidate branches/worktrees and immutable baselines.
- Define measurable task suites and regression tests.
- Compare candidate results with the baseline.
- Preserve candidate lineage, logs, artifacts, and evaluation results.
- Require approval before promoting core changes.
- Support rollback and reproducible rebuilds.

## Stage 6 — Multi-agent capabilities

- Add delegation only when single-agent workflows show a measurable need.
- Give each worker bounded tasks, tools, context, and budgets.
- Add coordination, shared artifacts, conflict handling, and evaluation.

## Stage 7 — Additional interfaces

- Add desktop, web, API, or automation interfaces as separate presentation/adaptor layers.
- Keep the same core contracts and identity.
- Do not duplicate cognition or memory logic per interface.

## Release gate

A stage is complete only when implementation, tests, documentation, security review, and reproducible build evidence are present. Roadmap checkboxes must reflect actual repository state.
