# Template / Fork Guide

RockSoul is designed to be reusable as a GitHub repository template without forcing downstream worlds to preserve the RockSoul product name or repository URL.

Examples:

```text
RockSoul  → StoneSoul
RockSoul  → NovaSoul
RockSoul  → private internal runtime
```

## Template boundary

The **architecture, World contract and automation pattern** are reusable. Product/repository identity is centralized rather than scattered through TUI and Web code.

The first file to change is:

```text
public/brand.env
```

Example:

```dotenv
NAME=StoneSoul
SLUG=stonesoul
GENERATION=1
COGNITIVE_AGE=0
LEVEL=1
XP=0
TRUST=0
WORLD=Internet
STATE=Born
PHASE=Phase A / Birth
TAGLINE=Local-first digital cognitive runtime
NAV=WORLD,MAP,QUESTS,CODEX,SYSTEM
GITHUB_OWNER=your-user
GITHUB_REPO=your-user/stonesoul
PROJECT_NAME=stonesoul
PAGES_URL=https://your-user.github.io/stonesoul/
VERCEL_PROJECT=stonesoul
VERCEL_URL=
ASSETS_REPO=bjo163/rocksoul-assets
ASSETS_REVISION=<accepted revision>
UI_REPO=bjo163/rocksoul-ui
UI_REVISION=<accepted revision>
UI_VERSION=<accepted UI version>
MIND_REPO=your-user/stonesoul-mind
```

Ratatui reads identity/topology from `public/brand.env` and `public/world.json`. The Web World Surface builds repository, Issue, Release, Wiki, Project and Codex links from this shared identity rather than a second JavaScript branding file.

## World topology

[`../public/world.json`](../public/world.json) is intentionally mostly brand-neutral. Special values such as `@repo`, `@project`, `@wiki`, `@pages`, `@assets`, `@ui` and `@mind` are resolved by the Web runtime using `brand.env`.

This means changing the World identity does not require copying a separate Web topology just to change repository URLs.

The asset/UI repositories can remain shared RockSoul ecosystem dependencies, or a derived world may deliberately replace them after establishing its own accepted provenance.

## Existing visual resources

Template reuse follows:

```text
EXISTING
  ↓
REUSE
  ↓
EXTEND
  ↓
CREATE
```

The browser surface uses **brand-neutral** architecture/map primitives from the pinned `rocksoul-assets` revision. It does not depend on a RockSoul wordmark to render a derived product identity.

`rocksoul-ui` remains the canonical production implementation grammar. Do not fork a second design system inside the generated repository merely to rename the product.

## After creating a repository from the template

1. Change `public/brand.env` identity/repository fields.
2. Review `public/world.json`; preserve generic topology and replace only ecosystem dependencies that intentionally differ.
3. Change package/crate names only if the new project needs different Rust package identifiers.
4. Replace public product prose in README/docs when the derivative becomes a distinct project.
5. Review `.github/roadmap-issues.json`; keep only phases relevant to the new world.
6. Add the derivative's own `ROCKSOUL_GITHUB_TOKEN` equivalent if privileged Project/Wiki/Pages/repository sync is desired; rename the secret/workflow only if there is a reason.
7. Run the synchronization workflow so labels, milestones, Project, Wiki and Pages are rebuilt from canonical repository sources.
8. Connect the derivative repository to its **own** Vercel project if a Live World Portal is wanted.
9. Do not copy secrets, cloud credentials, private datasets, runtime databases or runner registration tokens from the source repository.
10. Verify exact-head `rocksoul-gate` before publishing a World Build.

## What remains generic

- `rocksoul-gate` quality/security/release pattern;
- `rocksoul-sync` governance/Project/Wiki/Pages/release pattern;
- Issue forms, PR template and CODEOWNERS;
- label/milestone/roadmap JSON as machine-readable governance;
- `public/world.json` evidence/discovery vocabulary;
- local-first storage rules;
- TUI/Web parity contract;
- age-aligned release packaging strategy.

## GitHub template setting

`rocksoul-sync` can mark the repository as a GitHub template when the privileged token has repository administration permission. This keeps the setting reproducible rather than relying on an undocumented manual toggle.

## Naming rule

Do not introduce a second branding configuration file. Runtime-facing display/repository identity belongs in `public/brand.env`.

Repository prose may mention the project name naturally, but the runtime should not require mass code edits just to boot under another name.
