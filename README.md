# RockSoul

RockSoul is a Rust-first digital cognitive system inspired by human cognitive structure and Qur'anic principles such as knowledge, verification (tabayyun), trust (amanah), consultation (shura), and reflection (muhasabah).

RockSoul is **not** a claim of artificial soul, consciousness, ruh, or nafs. These concepts are used only as bounded architectural inspiration where appropriate.

## Repository role

This repository contains the RockSoul **life system**: runtime, cognition, memory, world model, senses, skills, guardian, life progression, API, and Ratatui interface.

Model training and checkpoint research remain isolated in [`bjo163/rocksoul-mind`](https://github.com/bjo163/rocksoul-mind).

## Direction

```text
Body        -> Rust runtime
Senses      -> connectors and observations
Attention   -> RockSoul Nano
Memory      -> working / episodic / semantic / procedural
Aql/Cortex  -> reasoning and planning
Tabayyun    -> verification
Yad         -> deterministic tool execution
Amanah      -> permission and risk controls
Muhasabah   -> outcome reflection
World       -> Internet + physical + private digital systems
Life        -> age + level + XP + skills + trust
```

The long-term research direction is general and collective intelligence. **ASI is treated as an empirical research outcome, never a product label assigned without evidence.**

## Status

Phase A — **Birth**.

The initial implementation is intentionally a small modular Rust monolith with Ratatui. No microservices, permanent multi-agent swarm, or unnecessary infrastructure is introduced at this stage.
