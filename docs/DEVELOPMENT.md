# Development

## Requirements

- Rust 1.88.0, pinned by `rust-toolchain.toml`
- Git
- a terminal supported by Crossterm for the Ratatui app
- Python 3 only if you want a trivial local static server for the Web mirror

No Node.js/frontend toolchain is required by the Phase A Web surface.

## Local checks

Run the same Rust checks owned by `rocksoul-gate`:

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

Run the Web mirror:

```bash
python -m http.server 3000 -d public
```

## Branch model

Only these branch namespaces are intentional:

- `main` — releasable source.
- `main` — current protected integration and release branch. Use short-lived `feature/*` branches for changes.
- `feature/*` — all short-lived human work, regardless of whether the change is feature/fix/docs/refactor/maintenance/security.
- `dependabot/*` — GitHub-native bot exception.

Use labels and Conventional Commit prefixes for work type; do not multiply branch prefixes for the same metadata.

Examples:

```text
feature/persistent-birth
feature/world-graph
feature/fix-resume-state
feature/docs-storage
```

Avoid permanent phase/release branches. The roadmap belongs in issues/milestones and releases belong in tags.

## Shared TUI/Web surface

`public/brand.env` is the single display identity configuration used by both Ratatui and the Vercel Web mirror.

Do not add a second branding file or frontend-specific product identity. When runtime state becomes persistent, extract a shared serializable state contract rather than implementing cognition twice.

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
- Do not add a JavaScript framework merely to mirror a terminal screen that static HTML can represent.

## Storage rules

Use `docs/STORAGE.md` before introducing persistence or cloud storage. One data domain has one authoritative store; local-first is the default.

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
- `rocksoul-gate` passes;
- docs are updated if architecture or operator behavior changes;
- TUI/Web state contracts remain aligned when applicable;
- consequential behavior is deterministic or explicitly bounded;
- new work left behind is represented as an issue rather than an anonymous TODO.
