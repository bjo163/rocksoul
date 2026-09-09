# Development

## Requirements

- Rust 1.88.0, pinned by `rust-toolchain.toml`
- Git
- a terminal supported by Crossterm for the Ratatui app

## Local checks

Run the same quality gates used by CI:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Run the TUI:

```bash
cargo run -p rocksoul-tui
```

## Branch model

- `main` — stable integration branch; merge only through reviewed/validated PRs.
- `dev` — integration branch for ongoing phase work when needed.
- short-lived branches — `feat/*`, `fix/*`, `docs/*`, `chore/*`, `refactor/*`, `security/*`.

Avoid permanent phase branches. The roadmap belongs in issues/milestones, not stale branches.

## Crate policy

RockSoul starts as a modular monolith. A new crate must have a real ownership boundary and implementation need.

Do not create empty future crates solely to mirror the architecture diagram.

Expected evolution is approximately:

```text
rocksoul-core
rocksoul-life
rocksoul-tui
  ↓ as phases become real
rocksoul-world
rocksoul-memory
rocksoul-brain
rocksoul-senses
rocksoul-skills
rocksoul-cognition
rocksoul-guardian
rocksoul-api
```

This is a direction, not a requirement to split every subsystem.

## Dependency rules

- Prefer Rust standard library and existing workspace dependencies when sufficient.
- Explain new infrastructure dependencies in the PR.
- Do not add a database, queue, service, framework, or model runtime only for anticipated future use.
- `rocksoul-core` should stay dependency-light and must not depend on UI or higher-level subsystems.
- Dependency direction should flow from application layers toward core abstractions, not cyclically.

## Commit and PR style

Prefer Conventional Commit-like prefixes:

```text
feat:
fix:
docs:
refactor:
test:
ci:
chore:
security:
```

A PR should state what changed, why, validation performed, architectural impact, and any follow-up work.

## Definition of done

A change is done only when:

- behavior is implemented rather than mocked without explicit reason;
- tests cover meaningful invariants where practical;
- format/check/clippy/tests pass;
- docs are updated if architecture or operator behavior changes;
- consequential behavior is deterministic or explicitly bounded;
- new work left behind is represented as an issue rather than an anonymous TODO.
