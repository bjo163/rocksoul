# Ruh Information (RID)

RID is the persistent developmental profile for one Rocksoul instance.

## Three independent concepts

- **Age** is elapsed wall-clock time since the instance creation timestamp. It is not human age, consciousness, or sentience.
- **XP and level** represent verified progression events. The current level curve is version 1: level N is reached at 50 × (N - 1) × N cumulative XP (100 XP for level 2, 300 for level 3, 600 for level 4).
- **99 stats** represent measured technical capabilities. They are independent from XP and level.

## Local profile

The profile is stored as JSON at the application data directory under `identity/rid.json`. `rocksoul init` creates it once and preserves the existing instance ID and creation timestamp on subsequent runs. A temporary-file and backup replacement strategy is used for writes. Invalid profiles are reported, never silently reset.

CLI:
- `rocksoul rid status` — age, level, XP, number of rated stats.
- `rocksoul rid stats` — all 99 slots and current measurement state.
- `rocksoul rid history` — auditable XP event list.
- REPL shortcut `:rid` — RID status.

The domain model and persistence layer are local-first. HTTP API exposure will be added through the same core contracts later.

## 99 Universal Stats

The canonical technical taxonomy is `data/rid/universal_stats.v1.json` (catalogue ID `rocksoul.rid.universal_stats.v1`, version `1.0.0`). It defines exactly 99 stable universal capability dimensions in nine domains, with eleven stats per domain. Each stat has a stable `rid_stat:NN` ID, a concise action-word label (such as `SEE`, `HEAR`, `REMEMBER`, `REASON`, `CREATE`, and `TRUST`), an operational definition, and an evidence-oriented evaluation protocol.

The nine domains are:
1. Perception
2. Memory & Learning
3. Reasoning
4. Execution
5. Communication
6. Safety & Integrity
7. Creativity & Transfer
8. Metacognition
9. Collaboration

The labels are universal capability vocabulary, not a claim that every entity or runtime supports every modality. A score requires a versioned held-out evaluation, explicit acceptance criteria or ground truth, sufficient samples, confidence, evidence references, evaluator identity, timestamp, and rubric version. Initial state is `unrated`; `not_applicable` must remain distinct from zero. XP, level, age, consciousness, personhood, and spiritual worth are independent concepts and must never be used as proxies for stat scores.

### Religious inspiration catalogue — separate reference layer

`data/rid/asmaul_husna.v1.json` is retained as a separate religious/inspirational reference catalogue. It is not the source of universal stat identities, labels, scores, or evaluation. Its Arabic names, transliterations, glosses, enumeration variant, and editorial associations remain subject to independent qualified religious and language review. The former one-to-one ordinal pairing must not be interpreted as semantic or theological equivalence. No technical stat represents or measures a divine attribute.

The selected legacy sequence is the popular enumeration reproduced in Jami at-Tirmidhi 3507, whose grading is displayed as Da'if (Darussalam) on Sunnah.com. Jami at-Tirmidhi 3508 is displayed as Sahih (Darussalam), but explicitly does not enumerate the names. This is a documented catalogue variant, not a claim that its ordering is the only undisputed enumeration. References:
- Sahih Muslim 2677: https://sunnah.com/muslim/48/5
- Bulugh al-Maram 1382: https://sunnah.com/bulugh:1382
- Indonesian context on the number and enumeration: https://islamqa.info/id/answers/41003

### Compatibility and migration

Existing RID profiles and the current Rust domain still use legacy slot identifiers `asmaul_husna:01` through `asmaul_husna:99`. Preserve those records and their history. The new catalogue reserves stable universal IDs `rid_stat:01` through `rid_stat:99`; runtime migration must be explicit, versioned, tested against real profile fixtures, and must not silently reinterpret old scores. Until that migration is implemented and verified, the new catalogue is the canonical product taxonomy while the legacy catalogue remains the runtime compatibility source.

## XP integrity

XP is an append-only event ledger. Each event has a unique ID, bounded amount, source category, timestamp, evidence reference, and policy version. Replaying an identical event is idempotent; reusing an ID with different data is rejected. The current per-event maximum is 100 XP.

Only a trusted kernel verifier may authorize XP awards or stat evaluations. A model cannot self-award XP or write its own measured scores. Raw token count, uptime, message count, and unverified self-report are not valid XP sources.

## Current implementation boundary

Implemented now:
- RID schema and validation.
- 99 stable, initially unrated stat slots.
- Instance age calculation, XP ledger, and deterministic level curve.
- Local JSON profile initialization, read, validation, and recoverable file replacement.
- CLI status/stats/history and REPL status shortcut.
- Embedded, validated 99-entry catalogue; rid stats displays each name label alongside its separate technical capability.

Still pending:
- Independent religious/language review of the 99-name glosses and editorial mapping.
- Evaluator integration that can produce trusted evidence.
- Kernel-mediated XP awards tied to verified task and evaluation events.
- Native API read endpoints and richer memory/evolution integration.
