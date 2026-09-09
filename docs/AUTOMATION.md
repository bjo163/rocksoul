# Automation

RockSoul automation follows a strict rule: **one automation owner per responsibility**. The repository intentionally keeps only two GitHub Actions control planes plus GitHub-native Dependabot.

## 1. `rocksoul-gate`

File: `.github/workflows/gate.yml`

The gate owns deterministic validation and distributable build evidence:

- branch/repository policy validation;
- Rust format, check, clippy and tests;
- RustSec dependency audit;
- dependency review on pull requests;
- Markdown link integrity;
- static Web/Vercel surface smoke tests;
- CodeQL for Rust;
- tag-triggered creation of a GitHub Release;
- Linux, Windows `.exe`, and macOS release binaries.

It runs on `main`, `dev`, pull requests, release tags and a low-frequency scheduled security check.

A release tag is **not** allowed to skip the same gate used for ordinary code.

## 2. `rocksoul-sync`

File: `.github/workflows/sync.yml`

The sync plane owns metadata and presentation surfaces:

- path-based PR labels;
- labels from `.github/labels.json`;
- milestones from `.github/milestones.json`;
- canonical issues from `.github/roadmap-issues.json`;
- public GitHub Project `rocksoul`;
- Project Priority/Phase fields and views;
- GitHub Wiki mirror from `/docs`;
- repository description/topics/template/discussion settings;
- Release Drafter;
- manual age-aligned tag creation;
- stale issue/PR hygiene.

Issue and milestone governance uses the normal repository `GITHUB_TOKEN`. Project/Wiki/admin synchronization requires the optional repository secret:

```text
ROCKSOUL_GITHUB_TOKEN
```

The token must have enough repository administration/Wiki access and GitHub Projects `project` scope. If it is absent, the job exits successfully with a notice; deterministic repo governance continues to work.

## 3. Dependabot

Dependabot is the **only dependency update bot** and owns Cargo + GitHub Actions update PRs. `dependabot/*` is the only non-human branch-prefix exception to the `main`, `dev`, `feature/*` policy.

Renovate is deliberately absent.

## Responsibility matrix

| Concern | Owner |
| --- | --- |
| build/test/lint | `rocksoul-gate` |
| code/security scanning | `rocksoul-gate` |
| dependency change review | `rocksoul-gate` |
| binary packaging | `rocksoul-gate` |
| dependency update PRs | Dependabot |
| labels/milestones/issues | `rocksoul-sync` |
| Project/Wiki/repo metadata | `rocksoul-sync` |
| release draft + age tag | `rocksoul-sync` |
| stale hygiene | `rocksoul-sync` |
| Vercel deployment | Vercel Git integration; GitHub `main` remains source |

## AI automation

No AI bot is currently installed. Triage, labels, release versioning, security checks and project synchronization are deterministic and do not benefit from an LLM yet.

Add an AI automation only after a measurable gap appears, for example:

- semantic duplicate-issue detection beyond GitHub search;
- architecture review summaries for very large RFCs;
- release-note synthesis that deterministic labels cannot express;
- research evidence classification.

If that happens, use **one** AI gateway job behind the sync plane rather than multiple competing review bots. An OpenAI-compatible base URL and API key can be supplied then.

## Permissions and safety

- Workflows receive the smallest practical job-level permissions.
- Project/admin PAT use is isolated to the synchronization job.
- Bots do not automatically merge arbitrary human code.
- Release publication happens only through an explicit age-release request/tag.
- No automation grants RockSoul runtime authority over external systems.

## No shell-script sprawl

Repository automation logic stays in `gate.yml` and `sync.yml` while it remains readable. A helper script is extracted only when logic is reused by both local development and Actions or becomes independently testable. Do not create one-off `scripts/*.sh` wrappers merely to hide a few workflow lines.
