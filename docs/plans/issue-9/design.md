# Issue #9 — Design and implementation boundaries

## Current architecture

`src/tui.rs` currently owns terminal event handling, app navigation, snapshot loading and rendering. It reads existing services directly in a few render functions. Keep the one-Cargo-package architecture; refactor only enough to test read models and isolate presentation.

## Proposed module boundaries

Prefer private types/functions in `tui.rs` initially. Extract a module only if it materially improves testability or ownership.

- `TuiPreferences`: versioned presentation-only settings; load/save under `AppPaths::config_dir`.
- `CockpitSnapshot`: one refreshable, immutable snapshot of model configuration status, RID diagnostics, memory metadata, latest evaluation report, and portable path diagnostics.
- `TaskHistoryView`: bounded projection of `task_trace::read`; content-free and read-only.
- `MemoryBrowserView`: bounded record metadata projection with filters and explicit detail reveal.
- `DiagnosticsView`: health states and sanitized errors; no secret-bearing configuration values.
- `App`: selected screen, focus, overlays, filters and selected IDs; never authoritative data.

## Data-source mapping

| Screen | Authoritative source | Display rule |
|---|---|---|
| Overview | `AppPaths`, `ModelConfig`, RID profile, memory store | Actual readiness and counts only |
| Brains | Current configured chat adapter; future registry when implemented | Legacy adapter shown accurately; specialist roles marked unassigned |
| Tasks | `task_trace::read` | Persisted operation history, not a live queue |
| Memory | `MemoryStore::load/search_ranked` | Metadata by default; content only after explicit reveal |
| RID | `rid_store`, `RidCatalogue` | 99 stable slots; unrated stays distinct from zero |
| Tools | Future capability registry (not present) | Honest empty/not-implemented state |
| Diagnostics | Store validation, model configuration readiness, `eval::latest` | Sanitized diagnostics and latest report; no fake telemetry |
| Settings | Versioned local preference file | Presentation-only settings |

## Memory privacy behavior

- Never render all record content into a list.
- List ID, kind, claim status, source/provenance, timestamps, retention/expiry and approval status.
- Detail view is entered by an explicit key action; hide secrets using the existing privacy/sensitive-content guard where available.
- Do not add memory write, approve, deny, delete, clear, prune or export actions in this issue.
- Search must use bounded query length and bounded result count; no unbounded full-store rendering.

## Task history behavior

- Use existing JSONL trace reader; do not parse raw JSON manually.
- Bound the visible number of events and support selection/detail.
- Display event type and timestamp, safe filename/operation metadata, and status.
- A trace with no entries means “No persisted task events”; it does not mean no task is running elsewhere.
- Malformed/unsupported trace must display a sanitized diagnostic and must not crash the cockpit.

## Diagnostics and evaluation

- Read the existing Core Integrity latest JSON report; do not run evaluations automatically on every render.
- Show suite version, commit if recorded, pass/fail, passed/total checks, and failed check IDs.
- If report is missing, show “Not run”; if corrupt or unsupported, show “Unavailable / invalid report”.
- Never convert absent model latency, token usage, resource usage, specialist health or permission state into fabricated metrics.
- Redact API keys, bearer tokens, URLs with credentials and sensitive file contents in errors.

## Preferences

- Use a small versioned TOML/JSON document under the portable root; never use AppData or user profile storage.
- Defaults: dark theme, comfortable density, Indonesian locale only if translations exist; otherwise report English fallback honestly.
- Dark/light themes must be semantic token sets, not separate widget styling branches.
- Invalid/unknown future schema must fail closed to safe defaults without overwriting the unknown file.
- Preference changes must not mutate model config, network policy, credentials, permissions or evolution settings.

## Testing strategy

- Unit-test pure projections from fixture snapshots.
- Test navigation/focus and overlay escape behavior.
- Test empty, malformed and future-version stores/reports.
- Test memory filter/search bounds and explicit reveal state.
- Test task trace projection with valid, empty, malformed and unsupported records.
- Test preferences default, round-trip, invalid schema and unknown future version.
- Test diagnostic redaction with synthetic credential markers.
- Use Ratatui TestBackend for layout and screen assertions; no brittle screenshot snapshots.
- CI must remain offline; do not download datasets or contact model endpoints.

## Migration and rollback

No existing RID or memory schema migration is required. Preferences are additive and root-local. If preference parsing fails, use safe defaults and preserve the unknown/corrupt file for diagnosis. Reverting the feature removes only the preference file; it must not touch RID, memory, receipts, task traces or evaluation reports.
