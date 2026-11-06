---
id: fl-2fmfyz3
title: "Wire examples/short-table into the gates: fmt, clippy and its tests in both, and the push gate re-runs it and fails when OUTPUT.md differs from what it prints"
category: crate/proof
parent: fl-9p8yw9v
needs: [fl-0ntk3eb]
seq: 340
docs:
  - docs/DESIGN.md#verification
verify: 'all(contains scripts/regression.sh examples/short-table/OUTPUT.md, contains scripts/smoke.sh examples/short-table/, contains docs/DESIGN.md examples/short-table)'
status: open
created: 2026-11-06T14:19Z
---
Mirror the fuzz package's legs.

- `scripts/regression.sh`: `cargo fmt --manifest-path examples/short-table/Cargo.toml -- --check`;
  clippy on `--all-targets` with `-D warnings`; its tests; then `cargo run --quiet --locked` into a
  temporary file and `diff -u examples/short-table/OUTPUT.md` against it, failing with the command
  that regenerates the file. `--locked`, so a lockfile a release left stale fails loudly instead of
  being rewritten. Every leg passes `--target-dir "${CARGO_TARGET_DIR:-target}/short-table"`, so
  they share one build inside the root `target/`, which git ignores and CI's gate job caches.
- `scripts/smoke.sh`: a flag set when the diff reaches `examples/short-table/`, running the same
  legs, the way its `fuzz` flag does.
- CI needs no new job: the gate job runs both scripts on `ubuntu-latest`, so every nightly that
  runs and every release pull request re-derives the table on x86_64 Linux against the copy blessed
  on aarch64 macOS.
- `docs/DESIGN.md` § Verification gains a row naming the example and what the gate checks.

Watch the diff leg fail on a deliberate one-character edit to `OUTPUT.md` before trusting it.

## log
- 2026-11-06T14:19Z created
