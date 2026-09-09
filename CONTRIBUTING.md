# Contributing to RockSoul

Thanks for helping build RockSoul.

Before opening a PR, read:

- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)
- [`docs/ROADMAP.md`](docs/ROADMAP.md)
- [`docs/DEVELOPMENT.md`](docs/DEVELOPMENT.md)
- [`docs/GOVERNANCE.md`](docs/GOVERNANCE.md)
- [`docs/STORAGE.md`](docs/STORAGE.md) when adding persistence/cloud services

## Workflow

1. Search existing issues first.
2. For architecture changes, open an Architecture / RFC issue before a large implementation.
3. Branch from `dev` for integrated feature work, or the appropriate current base explicitly agreed by maintainers.
4. Use a short-lived `feature/<purpose>` branch for all human contributions.
5. Keep the change coherent and avoid unrelated cleanup.
6. Run the local Rust checks.
7. Update documentation when behavior or architecture changes.
8. Open a PR using the repository template; `rocksoul-gate` is the deterministic quality gate.

Examples:

```text
feature/persistent-birth
feature/world-map
feature/fix-state-recovery
feature/docs-deployment
```

Do not create new `feat/*`, `fix/*`, `docs/*`, `chore/*`, `release/*` or `hotfix/*` namespaces; labels and commit prefixes already encode those distinctions.

## Quality expectations

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Do not introduce hidden hardcoding, fake-success paths, unbounded loops, or production behavior that exists only in comments/mocks without being explicitly identified.

## Architecture changes

Changes that introduce a new runtime service, database, queue, model role, security boundary, external write capability, or crate should explain why the existing modular monolith cannot satisfy the requirement cleanly.

For storage, do not make Supabase and Neon dual writable masters for the same data domain. Preserve the local-first placement rules in `docs/STORAGE.md`.

## TUI / Web parity

Changes to operator-visible identity or state should keep Ratatui and Web aligned. `public/brand.env` is the single display-brand source until a shared persistent runtime-state contract supersedes static defaults.

## Security

Do not report exploitable vulnerabilities in a public issue. Follow [`SECURITY.md`](SECURITY.md).
