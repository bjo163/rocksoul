# Automation

RockSoul automation follows one rule: **one owner per responsibility**. Avoid overlapping bots that produce duplicate PRs, labels, or decisions.

## Active automation

| Automation | Responsibility | Trigger |
| --- | --- | --- |
| `ci.yml` | fmt, check, clippy, tests | push / PR |
| `governance-sync.yml` | canonical labels, milestones, roadmap issues | governance file change / manual |
| `pr-labeler.yml` | path-based PR labels | PR |
| `dependency-review.yml` | reject newly introduced vulnerable dependencies | PR |
| `codeql.yml` | Rust code scanning | push / PR / weekly |
| `security-audit.yml` | RustSec advisory audit | dependency change / weekly |
| `docs.yml` | Markdown link integrity | docs change / weekly |
| `stale.yml` | age out abandoned issues/PRs with exemptions | daily |
| `release-drafter.yml` | maintain draft release notes | merge/push to main |
| Dependabot | Cargo and GitHub Actions version updates | weekly |
| `project-sync.yml` | optional GitHub Project v2 creation/item sync | manual; requires project-scoped PAT |

## Deliberately absent

- **Renovate:** overlaps with Dependabot.
- **automatic code merge bot:** too much authority for the current maturity level.
- **permanent AI review bot:** not required until a concrete review gap is measured.
- **auto-release/publish:** release publication stays deliberate.
- **multiple stale bots / label bots:** one workflow owns each concern.

## Permissions

Every workflow uses the smallest practical `GITHUB_TOKEN` permissions.

Project v2 is exceptional: GitHub Projects requires a token with `project` scope. The optional project sync workflow expects `ROCKSOUL_PROJECT_TOKEN`; the default repository `GITHUB_TOKEN` is not treated as sufficient.

## Version maintenance

Dependabot tracks GitHub Actions as an ecosystem, so major/minor action updates arrive as PRs instead of silently changing runtime behavior.

Current automation was initially aligned with contemporary major releases supporting GitHub-hosted Node 24-era runners, including `actions/labeler@v7`, `actions/stale@v11`, `actions/dependency-review-action@v5`, CodeQL Action `v4`, and Release Drafter `v7`.
