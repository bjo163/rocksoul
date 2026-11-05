# Rocksoul Cognitive Cockpit Architecture

Status: **Implemented increment on task branch; final-head validation and merge pending.**
Scope: opt-in Ratatui presentation in src/tui.rs and portable preferences in src/tui_preferences.rs.

## Boundaries

The Cognitive Cockpit is a read-only local presentation layer over existing Rocksoul Core stores and reports. It does not own cognition, model routing, tool execution, memory writes, task execution, XP, or verification.

- Snapshot::load resolves AppPaths and builds typed in-memory projections for RID, model-configuration presence, memory store/health, bounded task history, latest Core Integrity report, latest memory-retrieval report, and presentation preferences.
- Render functions consume the snapshot and read-only in-memory projections; screen rendering performs no filesystem or store reads. Evaluation suites are never executed during rendering.
- The task monitor reads the existing content-free JSONL workspace trace through task_trace::read_recent, with a 1 MiB file ceiling and density-bounded newest-first record count.
- The memory browser reads schema-v4 memory metadata and uses the existing ranked local search implementation. Content is hidden by default. Explicit Enter-to-reveal passes through the shared memory_proposal::sensitive_content_reason guard and clips/sanitizes displayed text.
- Diagnostics render generic safe states for missing, corrupt or unsupported reports and sanitize RID error details rather than exposing raw filesystem/parser errors.

## Portable presentation preferences

File: AppPaths.config_dir/tui-preferences.v1.json

Schema v1 stores only:
- theme: dark or light.
- density: compact or comfortable.
- locale: english or indonesian.

The preference module applies safe defaults when the file is absent, rejects malformed or unknown schema versions, and preserves existing bytes rather than overwriting a file it cannot understand. Writes use a temporary file and a recoverable backup/replace sequence. Preferences cannot change model provider configuration, tool permissions, remote egress, secrets, memory policy or security boundaries.

## Semantic presentation

The TUI builds a semantic palette from the selected theme: background, panel, text, muted text, accent, success, error and border. Screens and overlays use the same palette. Density bounds task history and memory list sizes. Locale changes navigation labels, screen titles, header/footer and settings labels.

Keyboard interactions are explicit:
- Number keys and Tab/BackTab navigate screens.
- Ctrl+K opens the command palette; ? opens help.
- Memory search uses /; f/c cycle kind/claim filters; Up/Down selects; Enter reveals guarded content; Esc closes detail/search.
- Task history uses Up/Down to select a persisted event.
- Settings use t/d/l to change theme/density/locale and persist the versioned preference file.

## Privacy and safety constraints

- No synthetic active-task state, fake telemetry, live queue, model registry or tool registry is invented.
- Task trace body/detail fields are never rendered.
- Memory content is not rendered in list/search results; explicit reveal is guarded and bounded.
- The TUI does not invoke models, tools, remote APIs, task execution, memory writes, forgetting, or automatic context injection.
- Invalid preference schemas are not silently reset or overwritten.

## Validation

The TestBackend suite renders all eight screens at 80×24, 120×40 and 60×20, in dark/light themes and English/Indonesian locales, including command-palette and help overlays. Keyboard selection/search/detail transitions, privacy guard behavior, preference round-trip/default/future-schema/malformed-file handling and bounded trace reads have focused tests.

Final formatting, Clippy, full test suite, Core Integrity, retrieval evaluation and Windows packaging/smoke tests must be rerun on the exact final PR head before merge.