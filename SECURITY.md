# Security Policy

RockSoul is early-stage research software. Treat all privileged integrations, credentials, model-driven actions, and autonomous execution paths as security-sensitive.

## Reporting a vulnerability

Do **not** open a public GitHub issue for a vulnerability that could expose credentials, enable unauthorized actions, bypass policy, or compromise a host/system.

Use GitHub's private vulnerability reporting / Security Advisory interface for this repository when available. If that channel is unavailable, contact the repository owner privately through an established trusted channel rather than posting exploit details publicly.

## Security principles

- intelligence never implies authorization;
- default toward read-only access;
- credentials never belong in source, prompts, logs, or committed configuration;
- privileged tools require explicit policy gates;
- actions should be reversible where practical;
- consequential claims/actions retain audit evidence;
- autonomous loops are finite and budgeted;
- live experience does not directly modify production model weights.

## Automated checks

The repository uses CodeQL, dependency review, RustSec/cargo-audit, Dependabot, and CI. Automated checks reduce risk but do not replace design review for privileged behavior.
