---
id: fl-bvt5k3t
title: Name the oracle lib in both gates' fuzz test leg so cargo 1.100's unused_dependencies lint stops warning on libfuzzer-sys and postcard
status: done
category: crate/hardening
discovered-from: fl-gjxces5
verify: "all(contains scripts/regression.sh /cargo test --quiet --locked --manifest-path fuzz\\/Cargo\\.toml --lib/, contains scripts/smoke.sh /cargo test --quiet --locked --manifest-path fuzz\\/Cargo\\.toml --lib/)"
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
- 2026-11-24T23:54Z open→doing — claimed by claude (4aebe618-581f-48ef-9ee8-943721730ef9)
- 2026-11-25T00:10Z doing→done — verify exit 0 @ 12c59f4+4

## comments
- 2026-11-25T00:10Z [claude (4aebe618-581f-48ef-9ee8-943721730ef9)] 2026-11-24 at 11e3c9a: the leg passes `--lib` in both gates, not `--all-targets`, and the title and verify are repointed to say so. `--all-targets` was measured here only with `--no-run`; run for real on cargo 1.100.0 it builds the three fuzz targets as test binaries and runs them (`test = false` in fuzz/Cargo.toml does not hold against an explicit selection), and `decode_robustness-… --quiet` fuzzed at one core for 10 min 12 s with the lib's four tests already passed, until it was stopped. A gate with that flag never returns.
  
  `--lib` instead, from cargo 1.100.0's own lint reference (doc/book/src/reference/lints.md at 3acc79c0c): "The cargo target selection flags, independent of which packages are selected, determine which dependencies tables are checked", with `cargo check --bin foo` given as a selection that "will not lint `[dependencies]`". Observed on aarch64 macOS: the leg as it stood prints the two warnings on 1.100.0 and none on 1.99.0; with `--lib` it prints none on either; `4 passed` all four ways. What `--lib` drops is the doctest pass, which reports 0 tests: fuzz/src/lib.rs has no doctest, and one added later would need `--doc` beside it.
  
  Not observed: that the clippy leg's `--all-targets` reports a dependency that really went unused, which the same reference says it does ("`cargo check --all-targets` will still only lint `[build-dependencies]` and `[dependencies]`"). A probe that added `num-integer` to `[dependencies]` drew no warning from either leg, and proves nothing: the rule skips a direct dependency that is also a transitive one (`is_transitive_dep` in src/diagnostics/rules/unused_dependencies.rs), and num-bigint brings num-integer in. The leg's wall clock is unchanged, under a second warm.
