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

The canonical technical taxonomy is `data/rid/universal_stats.v2.json` (catalogue ID `rocksoul.rid.universal_stats.v2`, version `2.1.0`). It defines **99 independent first-class universal capability domains**, each with a unique `rid_domain:NN` ID, a unique key and definition, and exactly one linked stat dimension with a stable `rid_stat:NN` ID. There is no mandatory nine-domain parent hierarchy and no fixed 11-stats-per-category grouping.

The complete human-readable catalogue, inclusion/exclusion criteria, and overlap guards are documented in [RID — Universal 99 Domains & Stats](rid-universal-stats.md). The former `data/rid/universal_stats.v1.json` (nine broad groups × eleven stats) is retained only as a historical snapshot and is superseded by v2.

Every domain now declares a primary measurement target, inclusion criteria, exclusion criteria, and overlap guards. The same observed outcome must not be double-counted across scores. Correlated capabilities may coexist, but require separate observable outcomes and separate rubric checks. If the evaluator cannot separate them, leave the affected dimensions unrated. The labels are universal capability vocabulary, not a claim that every entity or runtime supports every modality. A domain may be `not_applicable` when its modality or capability is absent. A score requires a versioned held-out evaluation, explicit acceptance criteria or ground truth, sufficient samples, confidence, evidence references, evaluator identity, timestamp, and rubric version. Initial state is `unrated`; `not_applicable` must remain distinct from zero. XP, level, age, consciousness, personhood, and spiritual worth are independent concepts and must never be used as proxies for stat scores.

### Religious inspiration catalogue — separate reference layer

`data/rid/asmaul_husna.v1.json` is retained as a separate religious/inspirational reference catalogue. It is not the source of universal stat identities, labels, scores, or evaluation. Its Arabic names, transliterations, glosses, enumeration variant, and editorial associations remain subject to independent qualified religious and language review. The former one-to-one ordinal pairing must not be interpreted as semantic or theological equivalence. No technical stat represents or measures a divine attribute.

The selected legacy sequence is the popular enumeration reproduced in Jami at-Tirmidhi 3507, whose grading is displayed as Da'if (Darussalam) on Sunnah.com. Jami at-Tirmidhi 3508 is displayed as Sahih (Darussalam), but explicitly does not enumerate the names. This is a documented catalogue variant, not a claim that its ordering is the only undisputed enumeration. References:
- Sahih Muslim 2677: https://sunnah.com/muslim/48/5
- Bulugh al-Maram 1382: https://sunnah.com/bulugh:1382
- Indonesian context on the number and enumeration: https://islamqa.info/id/answers/41003

### Compatibility and migration

The Rust runtime now embeds the universal v2.1.0 catalogue and new profile schema v2 uses stable `rid_stat:01` through `rid_stat:99` identifiers. When a schema-v1 profile is loaded, migration preserves the exact original profile bytes in `profile.json.v1.bak`, stores all 99 legacy stat records in an immutable archive inside the migrated profile, preserves instance identity and XP, and initializes all 99 universal stats as `unrated`. Historical scores are never mapped by ordinal position because the constructs changed. The separate Asmaul Husna file remains a religious/inspirational reference only. Migration code and fixture coverage are now on main; CI verification is pending.

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
