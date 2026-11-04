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

## 99 Asmaul Husna-inspired stat slots

The domain model creates exactly 99 stable slots, linked by IDs `asmaul_husna:01` through `asmaul_husna:99`. The versioned display catalogue now lives at data/rid/asmaul_husna.v1.json. It contains 99 Arabic labels, transliterations, Indonesian/English draft glosses, and 99 unique technical dimensions grouped into nine domains. Each dimension has an operational definition, an evaluation protocol, a minimum sample requirement (20 by default; 30 for safety/governance), and a scoring-rubric ID. The Rust domain refuses a score below the catalogue's sample threshold. Glosses and technical pairings remain editorial drafts pending independent review. A slot remains unrated until a trusted evaluator produces reproducible evidence.

Asmaul Husna is an inspirational naming layer only. Rocksoul does not possess, embody, or measure divine attributes. A stat is an engineering metric with an operational definition, evidence, confidence, and rubric version.

The selected sequence is the popular enumeration reproduced in Jami at-Tirmidhi 3507, whose grading is displayed as Da'if (Darussalam) on Sunnah.com. Jami at-Tirmidhi 3508 is displayed as Sahih (Darussalam), but explicitly does not enumerate the names. The catalogue records both references and does not claim that its chosen ordering is the only undisputed enumeration. See:
- Sahih Muslim 2677: https://sunnah.com/muslim/48/5
- Bulugh al-Maram 1382: https://sunnah.com/bulugh:1382
- Indonesian context on the number and enumeration: https://islamqa.info/id/answers/41003

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
