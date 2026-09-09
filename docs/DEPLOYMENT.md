# Deployment

RockSoul exposes the same operator identity through two surfaces:

- **Ratatui** — native local operator interface.
- **Web/Vercel** — lightweight browser mirror.

Both surfaces read branding from `public/brand.env`. The web mirror intentionally has no frontend framework or package dependency in Phase A.

## Local TUI

```bash
cargo run -p rocksoul-tui
```

Release binary:

```bash
cargo build --release -p rocksoul-tui
```

## Local web mirror

Serve `public/` with any static server:

```bash
python -m http.server 3000 -d public
```

Open `http://localhost:3000`.

## Vercel

The repository contains `vercel.json` with `public/` as the output directory. Import `bjo163/rocksoul` into Vercel with the repository root as the project root and framework preset **Other**.

There is no build command in Phase A. Vercel serves the same static interface from `public/`.

Preview deployments should follow `feature/*`; production should follow `main`. `dev` remains the integration branch and may be connected to a stable preview environment when useful.

## Surface parity contract

TUI and Web must keep these concepts aligned:

```text
Name
Generation
Cognitive Age
Level
XP
Trust
World
State
Phase
```

Today the web surface reads Phase A defaults from `public/brand.env`. Once persistent runtime/API state exists, both surfaces should consume the same serialized runtime state contract rather than duplicate business logic.

## Releases

Release publication is deliberately lightweight:

1. Release Drafter continuously prepares human-readable notes from merged PR labels.
2. Run `rocksoul-sync` manually with `publish_age_release=true` when a public build is worth publishing.
3. The sync workflow calculates the next tag as `v0.<COGNITIVE_AGE>.<revision>`.
4. `rocksoul-gate` validates the tag and builds downloadable binaries.
5. GitHub Release receives Linux x86_64, Windows x86_64 `.exe`, and macOS arm64 archives.

Example while cognitive age is `0`:

```text
v0.0.1
v0.0.2
v0.0.3
```

When cognitive age advances to `1`, the release line becomes:

```text
v0.1.1
```

This is an age-aligned distribution version, not a claim about capability maturity.

## Deployment authority

- GitHub `main` is the release source.
- Vercel is a presentation/deployment surface, never a second code source.
- GitHub Releases are the canonical binary distribution surface.
- Cloudflare R2 may mirror large immutable release assets when useful.
