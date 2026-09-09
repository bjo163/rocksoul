# Repository Governance

RockSoul uses repository automation to keep project state reproducible and reduce manual GitHub administration.

## Sources of truth

| Concern | Source of truth |
| --- | --- |
| Code | `main` |
| Architecture | `docs/ARCHITECTURE.md` |
| Phase intent | `docs/ROADMAP.md` |
| Work items | GitHub issues |
| Phase grouping | GitHub milestones |
| Label taxonomy | `.github/labels.json` |
| Canonical roadmap issues | `.github/roadmap-issues.json` |
| Automation behavior | `.github/workflows/*` and `docs/AUTOMATION.md` |

## Branch policy

`main` is the stable integration branch. It should require the `rocksoul-ci` check before merge. Direct pushes should be avoided once branch protection is enabled.

`dev` may be used for integrated phase development, but feature branches should remain short-lived.

Recommended repository ruleset for `main`:

- require pull request before merging;
- require `rocksoul-ci` status check;
- require branch to be up to date before merging when appropriate;
- block force pushes and branch deletion;
- allow squash merge as the default history style;
- optionally require CodeQL/security checks after they have proven stable.

The initial repository connection cannot mutate branch protection/rulesets, so this remains an explicit GitHub admin setting rather than pretending it is configured.

## Labels

Labels are namespaced so their meaning is obvious:

- `type:*` — nature of work;
- `area:*` — subsystem ownership;
- `priority:*` — urgency;
- `status:*` — workflow state;
- `phase:*` — roadmap phase;
- meta labels such as `dependencies`, `breaking-change`, `stale`.

`.github/workflows/governance-sync.yml` idempotently creates/updates the canonical labels and milestones.

## Issues

Each roadmap issue should define:

- goal;
- scope;
- non-goals;
- acceptance criteria;
- dependencies/blockers where known;
- relevant phase milestone.

Issue forms cover bugs, features, and architecture/RFC proposals.

## Pull requests

PRs should be small enough to validate coherently. Path-based labeling is automatic. A PR must not bypass deterministic quality checks simply because it is documentation-heavy; only jobs whose scope is genuinely irrelevant may be skipped by path filters.

## Releases

Release Drafter maintains an unpublished draft release based on merged PR labels. Publishing a release remains deliberate until the project has a stable release cadence.

Version impact conventions:

- `breaking-change` → major;
- `type:feature` → minor;
- everything else → patch by default.

## Automation safety

Bots may classify, check, draft, and maintain metadata. Bots do **not** automatically merge arbitrary code or grant runtime authority.

No two dependency bots are enabled: Dependabot is canonical, so Renovate is intentionally absent.
