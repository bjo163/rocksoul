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

- Define explicit request, response, and turn state.
- Add structured planning and reflection contracts.
- Add cancellation, token/time budgets, retry policy, and clear error states.
- Keep model-independent core interfaces.
- Add deterministic tests around state transitions.

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
