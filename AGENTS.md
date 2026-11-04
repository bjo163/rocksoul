# Rocksoul Repository Rules

These rules are mandatory for every human contributor, coding agent, automation, and delegated worker operating in this repository.

## 1. Single source of truth: `main`

- `main` is the only canonical, authoritative, current source of truth for Rocksoul.
- The latest commit on `origin/main` defines the accepted repository state. Do not treat a local checkout, old commit, release branch, feature branch, PR head, issue description, generated plan, or agent memory as authoritative when it conflicts with `main`.
- Before inspecting, planning, or changing code, fetch and inspect the latest `origin/main`. Confirm the base SHA and working-tree state.
- Every new task must start from the latest `main`. Create a short-lived task branch from that exact base; never continue work from an old feature branch.
- All accepted changes must flow back into `main` through a reviewed pull request (unless the repository owner explicitly authorizes a direct emergency fix).
- After a PR is merged, update the local checkout from `main`. The merged `main` state supersedes the source branch.
- Do not stack feature branches or use another feature/release branch as a PR base. The only normal PR base is `main`.
- Do not merge an old branch wholesale just because it is ahead of `main`, has an old PR, or contains commits absent from the main ancestry. Squash merges and later integrations can make commit graphs diverge even when their file content is already present.
- If valuable work is discovered on an old branch, compare its actual file content and behavior against current `main`. Port only the still-needed, reviewed changes onto a fresh branch created from current `main`; do not merge the stale branch.
- Closed, unmerged PRs and their branches are historical proposals, not accepted product behavior. Re-evaluate their ideas against the current blueprint, code, tests, and roadmap before selectively reintroducing anything.
- Never delete or rewrite `main`. Never force-push `main`.

## 2. Branch and pull-request policy

- Use short-lived branches with prefixes: `feat/`, `fix/`, `chore/`, or `docs/`.
- One coherent task per branch and PR. Keep changes small and reviewable.
- PR base must be `main`; PR descriptions must state the base SHA, scope, behavior changed, safety implications, and validation performed.
- Do not leave long-lived parallel implementation branches. After merge, a source branch is obsolete and must not be used as the starting point for new work.
- Before branch cleanup, verify that no open PR, release process, or required recovery work depends on the branch. Preserve history through merged PRs and tags; do not delete branches as a substitute for reconciliation.
- Release artifacts and release tags must be produced from a verified `main` commit. A release branch is not an alternative source of truth.

## 3. Product boundary

- Rocksoul starts as one Rust Cargo project and one native executable.
- Keep the first interface CLI-only. Do not add web, desktop, server, or multi-package workspaces unless explicitly requested.
- Keep the core independent of Python and Node.js. Future capabilities may invoke external runtimes as isolated tools.
- The CLI is the first interface, not the owner of the intelligence logic. Keep reusable application logic outside terminal presentation code.

## 4. Engineering rules

- Use stable Rust and preserve the declared minimum supported Rust version.
- Prefer small, explicit modules and dependencies over speculative frameworks.
- Run formatting, Clippy, and tests for every meaningful change.
- Add tests for behavior, error paths, configuration parsing, and security boundaries.
- Keep README, blueprint, architecture, roadmap, changelog, and CLI help consistent with actual implementation.
- Do not claim a capability exists unless the code implements and tests it.
- Do not silently revert or replace newer `main` behavior with an older branch snapshot.

## 5. Model and data safety

- Treat model output, tool output, and external content as untrusted data.
- Never hardcode credentials or commit API keys. Read secrets from environment variables or a future secret store.
- Never print API keys or include them in diagnostics.
- Do not add arbitrary shell execution, unrestricted filesystem mutation, or network scanning as implicit agent capabilities.
- Do not execute generated code inside the core process as a security boundary.
- Future code evolution must use isolated candidate workspaces, bounded permissions, tests, evaluation, version history, and rollback.
- Never automatically replace the running or trusted core with an unvalidated candidate.

## 6. Scope discipline

- Build the smallest complete vertical slice before adding abstractions.
- Avoid placeholder modules, fake model responses, mock success paths, and speculative extension points.
- A model response is not proof that an action was performed.
- Document limitations directly instead of pretending a feature is complete.
- When documentation conflicts, resolve it in favor of the current implementation on `main` and the approved `blueprint.md`; update the stale document in the same task when appropriate.


## 7. Dependency reproducibility

- `Cargo.lock` is committed and is part of the canonical application source on `main`.
- CI, release, and documented reproducibility checks must use Cargo's `--locked` flag for dependency-resolving commands (`build`, `check`, `test`, `clippy`, and `run`).
- A dependency update must intentionally update `Cargo.toml` and `Cargo.lock` together in one reviewed PR.
- Never regenerate or update the lockfile silently during a normal build or release.
- The lockfile automation may create a short-lived branch from latest `main`; its output must still be reviewed and merged through a PR targeting `main`.
