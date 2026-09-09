# Reproducible cognitive runtime release

`scripts/certify-release.ps1` is the local release evidence command. It runs
format checking, the complete workspace test suite, warning-denied Clippy, and
a release build. It then records the source commit, Rust toolchain, tracked
`Cargo.lock` SHA-256, binary path, binary SHA-256, and gate results in a JSON
report under `artifacts/`.

The report is evidence only; it does not promote model output, observations,
or derived graph data to canonical research. Release publication remains a
separate protected operation with rollback evidence.
