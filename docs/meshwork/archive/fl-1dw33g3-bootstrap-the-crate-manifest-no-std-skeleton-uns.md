---
id: fl-1dw33g3
title: "Bootstrap the crate — manifest, no_std skeleton, unsafe policy, MSRV 1.74, the Rust gate scaffold, cargo-deny and the CI skeleton"
category: crate/build
labels: [reproducible-floats, rf-critical-path]
docs: [docs/DESIGN.md#crate-posture-sp-posture, docs/DESIGN.md#features-sp-features, CLAUDE.md]
seq: 10
verify: "all(exists Cargo.toml, contains Cargo.toml /rust-version = .1\\.74./, contains Cargo.toml /line-tables-only/, exists src/lib.rs, contains src/lib.rs /deny.unsafe_code./, exists tests/main.rs, exists deny.toml, exists scripts/smoke.sh, exists scripts/regression.sh)"
status: done
created: 2026-10-11T15:02Z
covers:
  - ref: docs/DESIGN.md#sp-posture
    sha: 4f94eb664c7ecd323b64de98af7089f1877907ad81e11ee1d8f2be1fa6020c8d
  - ref: docs/DESIGN.md#sp-features
    sha: a938e9e4c72ed22866f0bf4632c81cc3793c0e3efb99b0c78005824dfe64707e
---
Lay the crate down per DESIGN.md § Crate posture and § Features.

- `Cargo.toml`: `license = "MIT OR Apache-2.0"`, edition 2021, `rust-version = "1.74"`, zero required
  dependencies. Declare every feature in the table; the adapter and kit features stay empty until
  their tasks wire them. The crate lists itself as a dev-dependency with `serde`, `differential`,
  `laws`, `simd` and `arbitrary` on, so a bare `cargo test` runs every feature-gated test.
- `src/lib.rs`: `#![no_std]`, `#![deny(unsafe_code)]`, the `alloc` and `std` gates.
- `tests/main.rs` is the one integration target, with topics as `mod`s.
- The Rust scaffold from CLAUDE.md's baseline: `[profile.dev]` debug settings, lints in the manifest,
  `scripts/smoke.sh` (commit gate), `scripts/regression.sh` (push gate, clippy and rustdoc with
  `-D warnings`), `.githooks/`. Warnings fail the gate only, so `.cargo/config.toml` carries no
  `-D warnings`.
- `deny.toml` for cargo-deny (licenses, advisories, duplicate majors).
- A CI workflow skeleton: the MSRV job on exactly the default features, the test job, and the dbsp job
  pinned to rustc 1.96.1 (empty until the DBSP adapter lands).
- A README note: keep `CARGO_TARGET_DIR` outside any Spotlight- or sync-covered tree.

Not this task: reserving the crates.io name. Any publish is the owner's act.

## log
- 2026-10-11T15:02Z created
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-posture @4f94eb664c7e
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-features @a938e9e4c72e
- 2026-10-11T15:32Z open→doing — claimed by claude (85c2609c-6abc-4455-b7f3-6c1c79eda92c)
- 2026-10-11T15:37Z doing→done — verify exit 0 @ d2f29de+1
