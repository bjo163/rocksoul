# Changelog

All notable Rocksoul changes are documented here.

## [0.2.0] — 2026-11-04

### Added
- Versioned persistent local memory stored in `data/memory/store.v1.json`.
- Episodic, semantic, procedural, working, and derived memory record types.
- Explicit user-approved memory writes; no automatic conversation ingestion or model write access.
- Memory list, search, delete, confirmed clear, working-memory expiry, and root-local JSON export.
- Real memory status in CLI, REPL, diagnostics, and the Cognitive Cockpit.
- Core evaluation checks for memory schema integrity and explicit approval.

### Storage and privacy
- All memory records and exports remain under the Rocksoul installation root.
- No AppData, Documents, user-profile storage, remote sync, vector database, or external database server.
- Deletion is permanent in the local store; existing exports are separate files and must be deleted separately if no longer wanted.

## [0.1.0] — 2026-11-04

### Added
- Rust-native Windows x64 CLI seed runtime and interactive REPL.
- Keyboard-first Rocksoul Cognitive Cockpit.
- RID identity, progression ledger, and 99 unrated technical capability dimensions.
- Deterministic cognitive turn lifecycle, bounded planning, model-only reasoning steps, and structured reflection.
- Per-turn limits for model calls, output tokens, and elapsed time.
- Evidence-gated XP verification and offline Core Integrity evaluation.
- Portable root storage: configuration, identity, evaluation reports, workspace, cache, evolution data, and logs stay beside the executable.
- Inno Setup Windows x64 installer and portable ZIP distribution.
- GitHub Release workflow with executable, SHA-256 checksum, installer, and portable package.

### Safety and limitations
- No arbitrary shell, filesystem tool, plugin, or autonomous self-modification capability.
- Reflector output is model review, not independent evidence.
- Persistent cross-session memory and external tool adapters are not part of this release.
- This release makes no claim of AGI or consciousness.
