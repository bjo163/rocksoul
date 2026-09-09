# RockSoul Documentation

This directory is the canonical engineering documentation for RockSoul. GitHub issues and milestones track executable work; docs define architecture, invariants, and operating rules.

## Start here

| Document | Purpose |
| --- | --- |
| [ARCHITECTURE.md](ARCHITECTURE.md) | System boundary, cognitive model, world model, life model, invariants |
| [ROADMAP.md](ROADMAP.md) | Phase order and exit criteria |
| [DEVELOPMENT.md](DEVELOPMENT.md) | Local development and crate rules |
| [GOVERNANCE.md](GOVERNANCE.md) | Branches, labels, issues, milestones, PR and release policy |
| [AUTOMATION.md](AUTOMATION.md) | GitHub Actions and bot ownership; overlap prevention |
| [PROJECT_MANAGEMENT.md](PROJECT_MANAGEMENT.md) | Canonical issue/milestone model and optional GitHub Project v2 sync |

Repository-level policies:

- [../CONTRIBUTING.md](../CONTRIBUTING.md)
- [../SECURITY.md](../SECURITY.md)
- [../CODE_OF_CONDUCT.md](../CODE_OF_CONDUCT.md)

## Source-of-truth rules

- **Architecture and invariants:** `docs/ARCHITECTURE.md`
- **Phase intent and ordering:** `docs/ROADMAP.md`
- **Current executable work:** open GitHub issues
- **Phase grouping:** GitHub milestones
- **Code state:** repository `main` and exact CI evidence
- **Model research:** `bjo163/rocksoul-mind`, not this repository

When docs and implementation disagree, open an issue and fix the inconsistency explicitly rather than silently treating one as correct.
