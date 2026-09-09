# Contributing to RockSoul

Thanks for helping build RockSoul.

Before opening a PR, read:

- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)
- [`docs/ROADMAP.md`](docs/ROADMAP.md)
- [`docs/DEVELOPMENT.md`](docs/DEVELOPMENT.md)
- [`docs/GOVERNANCE.md`](docs/GOVERNANCE.md)

## Workflow

1. Search existing issues first.
2. For architecture changes, open an Architecture / RFC issue before a large implementation.
3. Branch from the appropriate current integration ref.
4. Keep the change focused.
5. Run all Rust quality gates.
6. Update documentation when behavior or architecture changes.
7. Open a PR using the repository template.

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

## Security

Do not report exploitable vulnerabilities in a public issue. Follow [`SECURITY.md`](SECURITY.md).
