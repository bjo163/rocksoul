# Automation

RockSoul automation follows a strict rule: **one automation owner per responsibility**. The repository intentionally keeps only two GitHub Actions control planes plus GitHub-native Dependabot.

## 1. `rocksoul-gate`

File: `.github/workflows/gate.yml`

The gate owns deterministic validation and distributable build evidence:

- branch/repository policy validation;
- machine validation of labels, milestones, roadmap Issues and `public/world.json`;
- shared `brand.env` ↔ `world.json` asset/UI provenance consistency;
- Rust format, check, clippy and tests;
- TUI shared-world contract tests;
- RustSec dependency audit;
- capability-aware GitHub dependency review on pull requests;
- Markdown link integrity;
- World Portal JavaScript syntax + local HTTP smoke tests;
- CodeQL for Rust;
- tag-triggered creation of a GitHub Release;
- Linux, Windows `.exe`, and macOS release binaries.

It runs on `main`, `dev`, pull requests, release tags and a low-frequency scheduled security check.

A release tag is **not** allowed to skip the same gate used for ordinary code.

## 2. `rocksoul-sync`

File: `.github/workflows/sync.yml`

The sync plane owns metadata and public presentation surfaces:

- path-based PR labels;
- labels from `.github/labels.json`;
- milestones from `.github/milestones.json`;
- canonical Issues from `.github/roadmap-issues.json`;
- public GitHub Project `rocksoul`;
- Project Priority/Phase fields and Kanban/Roadmap/Backlog views;
- active `main` ruleset and repository settings;
- GitHub Wiki mirror from `/docs`;
- GitHub Pages **Codex** deployment from `public/`;
- repository description/topics/template/discussion settings;
- Release Drafter;
- opt-in age-aligned tag creation (`release:ready` or manual dispatch);
- stale Issue/PR hygiene.

Issue and milestone governance uses the normal repository `GITHUB_TOKEN`.

Project/Wiki/repository-settings/ruleset/Pages activation uses one optional repository secret:

```text
ROCKSOUL_GITHUB_TOKEN
```

It should have the least permissions necessary for:

- user-owned GitHub Project management (`project` scope or equivalent fine-grained access);
- repository Administration write for metadata and repository rulesets;
- Wiki write;
- Pages/Admin write needed to enable Pages with workflow publishing.

If the secret is absent, privileged jobs report a notice instead of claiming activation. Deterministic repository governance and `rocksoul-gate` continue to work.

Pages deployment itself uses the normal job-scoped `GITHUB_TOKEN` with `pages: write` and `id-token: write` **after Pages has actually been enabled**.

### Workflow-definition validation boundary

`rocksoul-sync` must be valid as a GitHub Actions workflow before any token/API capability can be evaluated. Keep YAML block scalars self-contained: do not place raw shell heredoc bodies outside the indentation of `run: |`. Prefer shell strings, `printf`, or generated JSON for embedded Project/Wiki/ruleset payloads.

A sync failure with **zero created jobs** is treated as a workflow-definition failure, not evidence that `ROCKSOUL_GITHUB_TOKEN` is invalid.

## 3. Dependabot

Dependabot is the **only dependency update bot** and owns Cargo + GitHub Actions update PRs. `dependabot/*` is the only non-human branch-prefix exception to the `main`, `feature/*` policy.

Renovate is deliberately absent.

## Responsibility matrix

| Concern | Owner |
| --- | --- |
| build/test/lint | `rocksoul-gate` |
| world/brand/TUI/Web contract validation | `rocksoul-gate` |
| code/security scanning | `rocksoul-gate` |
| dependency change review | `rocksoul-gate` |
| binary packaging | `rocksoul-gate` |
| dependency update PRs | Dependabot |
| labels/milestones/Issues | `rocksoul-sync` |
| Project/Wiki/repo metadata | `rocksoul-sync` |
| main ruleset | `rocksoul-sync` |
| GitHub Pages Codex | `rocksoul-sync` |
| release draft + age tag | `rocksoul-sync` |
| stale hygiene | `rocksoul-sync` |
| Vercel Live World Portal | Vercel Git integration; GitHub `main` remains source |
| Cloudflare Tunnel | no automation until a real private-service gateway is required |

## Runner strategy

The current RockSoul gate is verified on GitHub-hosted runners. Other repositories in the RockSoul ecosystem contain self-hosted runner contracts, but that is **not evidence** that those nodes are registered or online for `bjo163/rocksoul`.

Do not change `runs-on` to `self-hosted` merely because such runners exist elsewhere. First obtain authenticated runner inventory/capability evidence, then route a workload only when the labels/tools match.

This prevents a World Node from being presented as `ONLINE/READY` without evidence.

## AI automation

No AI bot is currently installed. Triage, labels, release versioning, security checks, world topology validation and Project synchronization are deterministic and do not benefit from an LLM yet.

Add an AI automation only after a measurable gap appears, for example:

- semantic duplicate-Issue detection beyond GitHub search;
- architecture review summaries for very large RFCs;
- release-note synthesis that deterministic labels cannot express;
- research evidence classification.

If that happens, use **one AI gateway job behind `rocksoul-sync`** rather than multiple competing review bots. An OpenAI-compatible base URL and API key can be supplied then.

## Permissions and safety

- Workflows receive the smallest practical job-level permissions.
- Project/admin PAT use is isolated to privileged synchronization steps.
- Bots do not automatically merge arbitrary human code.
- Release publication happens only via `release:ready` or explicit manual age-release.
- Pages/Vercel presentation never grants RockSoul runtime authority.
- No automation grants RockSoul authority over external systems merely because it can observe them.

## No shell-script sprawl

Repository automation logic stays in `gate.yml` and `sync.yml` while it remains readable. A helper script is extracted only when logic is reused by local development and Actions or becomes independently testable.

Do not create one-off `scripts/*.sh` wrappers merely to hide a few workflow lines. Two control planes remain the architectural gate.
