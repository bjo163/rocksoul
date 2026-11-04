# Rocksoul Roadmap

The roadmap is capability-driven. Each stage must deliver a working, tested increment. The stages are not promises of AGI.

## Stage 0 — Seed CLI

- [x] One Rust Cargo project and native executable.
- [x] Interactive CLI with help, status, clear, and exit.
- [x] OpenAI-compatible chat-completions adapter.
- [x] Local bootstrap and diagnostics commands.
- [x] Windows release build in CI.
- [ ] Commit the resolved dependency lockfile.
- [ ] Verify the release artifact on a clean Windows machine.

## Stage 1 — Reliable cognition loop

- [x] Define explicit request, response, and turn state contracts.
- [x] Add validated structured planning and reflection contracts.
- [x] Add model-call and elapsed-time budgets.
- [x] Track provider-reported completion tokens when available.
- [x] Generate a structured plan with a dedicated model call.
- [x] Execute at most three ordered model-only steps.
- [x] Enforce per-turn model-call, elapsed-time, and reported-token budgets.
- [ ] Add a separate model reflection pass with independently evaluated acceptance checks.
- [ ] Wire terminal cancellation and implement retry backoff/idempotency.
- [ ] Add token estimation for providers that omit usage metadata.

Current status: a turn performs one planning call and one to three model-only execution calls. No shell, filesystem, or external tools are exposed. Verification confirms non-empty outputs, not factual completion of plan-specific acceptance checks.

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

## Stage 5 — Isolated evolution

- Create candidate branches/worktrees and immutable baselines.
- Define measurable task suites and regression tests.
- Compare candidate results with the baseline.
- Require approval before promoting core changes.
- Support rollback and reproducible rebuilds.

## Stage 6 — Multi-agent capabilities

- Add delegation only when single-agent workflows show a measurable need.
- Give each worker bounded tasks, tools, context, and budgets.
- Add coordination, shared artifacts, conflict handling, and evaluation.

## Stage 7 — Additional interfaces

- Add desktop, web, API, or automation interfaces as separate presentation/adaptor layers.
- Keep the same core contracts and identity.

## Release gate

A stage is complete only when implementation, tests, documentation, security review, and reproducible build evidence are present.
