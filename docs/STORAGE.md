# Storage and Data Placement

RockSoul is **local-first**. Cloud services extend availability and collaboration; they do not become mandatory dependencies for the core runtime.

## Rules

1. One domain has one authoritative store.
2. Local state works without Internet whenever the feature logically can.
3. Git is for small, reviewable, versioned public state — not high-frequency transactions.
4. Secrets, credentials, private memories and personal data never enter the public repository, Wiki, Issues, Releases or public object storage.
5. Large generated objects live outside Git; Git stores their manifest, checksum, provenance and URL.
6. Do not mirror the same relational tables into both Supabase and Neon.

## Placement matrix

| Data class | Primary | Optional cloud | Why |
| --- | --- | --- | --- |
| runtime config / schema / public manifests | local files + Git repository | GitHub | versioned, reviewable, diffable |
| small public snapshots / fixtures | local files + `data/public/` | GitHub | reproducible research/development input |
| compiled EXE / immutable release bundles | local build | GitHub Releases | version/tag aligned download surface |
| temporary CI output | local build | GitHub Actions artifacts | ephemeral evidence, not canonical data |
| large public assets / models / archives | local cache | Cloudflare R2 | object storage, inexpensive, no general Git bloat |
| authenticated web uploads | local cache | Supabase Storage **or** Vercel Blob | choose based on application boundary |
| shared relational application state | local SQLite/Postgres | Supabase Postgres **or** Neon Postgres | choose exactly one per deployment profile |
| auth / realtime application data | local development | Supabase | use when Supabase Auth/RLS/Realtime is actually required |
| web-only edge assets | local `public/` | Vercel CDN / Blob | keep close to the web surface |

## Provider roles

### GitHub

Use the repository as a **versioned public state store** for configuration, schemas, small fixtures, indexes, manifests and reproducibility metadata. Use Releases for binaries and immutable bundles. Use Actions artifacts only for temporary build/test evidence.

Do not commit large generated files just because GitHub is available. GitHub recommends keeping repositories manageable and using object storage/LFS for large files.

### Cloudflare R2 — preferred general object store

Default cloud object storage for large public RockSoul assets, datasets, archives, model artifacts and release mirrors when GitHub Releases is not the right surface.

As of 2026-09-09, R2 Standard includes a monthly free tier of 10 GB-month storage, 1 million Class A operations and 10 million Class B operations, with Internet egress not charged. Re-check provider pricing before depending on a quota.

### Supabase — integrated application backend

Choose Supabase when a deployment needs Postgres plus Auth/RLS/Realtime and/or Storage as one integrated boundary. Do not introduce it merely because an account exists.

As of 2026-09-09, the Free plan advertises 500 MB database storage and 1 GB file storage. Free projects can pause after inactivity; production design must account for that behavior.

### Neon — database-only cloud profile

Choose Neon instead of Supabase Postgres when RockSoul needs a cloud Postgres database without the broader Supabase application stack. Neon is especially suitable for serverless/branchable Postgres workflows.

**Never make Neon and Supabase two writable masters for the same tables.** Migration between them is a controlled operation, not continuous dual-write sync.

### Vercel Blob — web-adjacent objects only

Use Vercel Blob when an object belongs directly to the Vercel web application lifecycle and the convenience is worth the coupling. Cloudflare R2 remains the default general-purpose object store.

The Vercel Hobby plan currently includes limited Blob storage and transfer. Treat that as a convenience tier, not the system archive.

## Local-first profiles

### Profile A — fully local

```text
RockSoul Runtime
├── local files / SQLite
├── local object cache
└── no cloud dependency
```

### Profile B — local + public distribution

```text
RockSoul Runtime
├── local authoritative state
├── GitHub repository → public manifests/config
├── GitHub Releases → binaries
└── Cloudflare R2 → large public objects
```

### Profile C — collaborative cloud

```text
RockSoul Runtime
├── local cache / offline state
├── GitHub → code + public manifests
├── R2 → large objects
└── ONE relational provider
    ├── Supabase, when Auth/RLS/Realtime is useful
    └── Neon, when cloud Postgres alone is preferable
```

## Public repository database convention

`data/public/` may contain only deterministic, non-secret, reviewable data. Every generated object stored elsewhere should be represented by a small manifest similar to:

```json
{
  "id": "artifact-id",
  "sha256": "...",
  "bytes": 0,
  "media_type": "application/octet-stream",
  "source": "cloudflare-r2",
  "url": "https://...",
  "created_at": "RFC3339"
}
```

The manifest is canonical metadata; the external object is canonical bytes.
