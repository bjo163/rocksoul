# Ruh Information (RID)

RID is the persistent developmental profile for one Rocksoul instance.

## Three independent concepts

- **Age** is elapsed wall-clock time since the instance creation timestamp. It is not human age, consciousness, or sentience.
- **XP and level** represent verified progression events. The current level curve is version 1: level N is reached at 50 × (N - 1) × N cumulative XP (100 XP for level 2, 300 for level 3, 600 for level 4).
- **99 stats** represent measured technical capabilities. They are independent from XP and level.

## 99 Asmaul Husna-inspired stat slots

The domain model creates exactly 99 stable slots, linked by IDs asmaul_husna:01 through asmaul_husna:99. The display catalogue (Arabic text, transliteration, gloss, source, variants, and technical rubric) must be curated separately and versioned. A slot is unrated until reproducible evaluation evidence exists.

Asmaul Husna is an inspirational naming layer only. Rocksoul does not possess, embody, or measure divine attributes. A stat is an engineering metric with an operational definition, evidence, confidence, and rubric version.

The commonly used enumerations have variant histories. The catalogue must name its selected source and version and must not present a chosen ordering as the only undisputed enumeration. See:
- Sahih Muslim 2677: https://sunnah.com/muslim/48/5
- Bulugh al-Maram 1382: https://sunnah.com/bulugh:1382
- Indonesian context on the number and enumeration: https://islamqa.info/id/answers/41003

## XP integrity

XP is an append-only event ledger. Each event has a unique ID, bounded amount, source category, timestamp, evidence reference, and policy version. Replaying an identical event is idempotent; reusing an ID with different data is rejected. The current per-event maximum is 100 XP.

Only a trusted kernel verifier may authorize XP awards or stat evaluations. A model cannot self-award XP or write its own measured scores. Raw token count, uptime, message count, and unverified self-report are not valid XP sources.

## Current implementation boundary

The Rust domain contracts and validation rules exist. Persistent storage, instance birth initialization, CLI commands, API read endpoints, and the reviewed 99-entry display catalogue are subsequent implementation work; do not describe them as shipped yet.
