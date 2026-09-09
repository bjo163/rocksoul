# Public Repository Data

This directory is the only intended location for small, deterministic, non-secret data that benefits from normal Git versioning.

Allowed examples:

- public fixtures and examples;
- schemas and registries;
- compact evaluation metadata;
- manifests that point to external immutable objects;
- reproducibility snapshots small enough to review as diffs.

Do **not** store:

- credentials, API keys, cookies or tokens;
- private user/runtime memory;
- personal or regulated data;
- database dumps;
- model checkpoints, large datasets or generated binaries;
- high-frequency mutable state.

Large objects belong in GitHub Releases or Cloudflare R2 according to [`docs/STORAGE.md`](../../docs/STORAGE.md). Keep a checksum/provenance manifest here only when that metadata is useful to reproduce or verify the object.
