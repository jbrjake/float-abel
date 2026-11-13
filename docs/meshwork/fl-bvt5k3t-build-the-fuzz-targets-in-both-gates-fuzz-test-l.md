---
id: fl-bvt5k3t
title: Build the fuzz targets in both gates' fuzz test leg so cargo 1.100's unused_dependencies lint stops warning on libfuzzer-sys and postcard
status: open
category: crate/hardening
discovered-from: fl-gjxces5
verify: "all(contains scripts/regression.sh /cargo test --quiet --locked --manifest-path fuzz\\/Cargo\\.toml --all-targets/, contains scripts/smoke.sh /cargo test --quiet --locked --manifest-path fuzz\\/Cargo\\.toml --all-targets/)"
docs:
  - docs/DESIGN.md#hardening-sp-hardening
seq: 95
created: 2026-11-13T19:04Z
---

Found 2026-11-13 at 35c0b77 while running scripts/regression.sh under `RUSTUP_TOOLCHAIN=1.100.0`
for fl-gjxces5, and present in the runner's smoke.sh output in ci run 43414233153: cargo 1.100.0
(3acc79c0c) has a lint of its own, `cargo::unused_dependencies`, warn by default, and both gates'
`cargo test --quiet --locked --manifest-path fuzz/Cargo.toml` leg prints it twice:

    warning: unused dependency `libfuzzer-sys`   --> Cargo.toml:19:1
    warning: unused dependency `postcard`        --> Cargo.toml:24:1

Both are used, by the three fuzz targets under fuzz/fuzz_targets/ (`libfuzzer_sys::fuzz_target`
in each, `postcard::from_bytes` in decode_robustness.rs and sub_roundtrip.rs). The test leg
builds the oracle lib and its tests and none of the bins, so cargo sees a build that never
reaches them. It is a warning from cargo, not from rustc or clippy, so `-- -D warnings` does not
see it and neither gate fails; it is noise in both gates' output on every run from 1.100 on.

Measured on this machine: the leg prints the two warnings on `cargo +1.100.0` and none on
`cargo +1.99.0`; the fuzz clippy leg, which passes `--all-targets`, prints none on either; and
the test leg with `--all-targets` added prints none on 1.100.0 and exits 0 on both (`--no-run`).

Wanted: the fuzz test leg in scripts/regression.sh and scripts/smoke.sh builds the targets it
tests beside, by `--all-targets`, rather than an `allow` in fuzz/Cargo.toml, which would also
hide a dependency that really went unused. Check the leg's wall clock in smoke.sh against the
commit gate's 60 s before and after, since it now links three bins.

## log
- 2026-11-13T19:04Z created
