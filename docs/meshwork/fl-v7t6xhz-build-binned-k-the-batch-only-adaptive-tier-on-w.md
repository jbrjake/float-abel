---
id: fl-v7t6xhz
title: "Build Binned<K> — the batch-only adaptive tier on Wide's grid, its Add, the conversions, its error bound, and compile-fail tests that keep it out of the engine weight traits"
category: crate/types
labels: [reproducible-floats, rf-tail]
docs: [docs/DESIGN.md#binned-sp-binned]
needs: [fl-at6nh41]
seq: 100
verify: run cargo test package=float-abel binned_
status: open
created: 2026-10-11T15:02Z
covers:
  - ref: docs/DESIGN.md#sp-binned
    sha: 28d2c310aff6a0a29005a96faae1d49e0557c54392112794fee21d28bddbecf2
handoff: |
  The tree is green through `./scripts/regression.sh` at 911bf92 (exit 0,
  sandbox off for
  cargo-deny's advisory fetch: fmt, the three clippy legs, rustdoc, 31 lib
  tests, 98 in
  `tests/main.rs` with `--include-ignored`, the doctest, the release
  headroom leg, `tests/noalloc`,
  the four trybuild cases, the MSRV 1.74.0 build of the default features,
  cargo-deny advisories,
  bans, licenses and sources ok). `CARGO_TARGET_DIR` is set outside
  ~/Documents by the repo's
  `.claude/settings.local.json`.
  
  Since the last handoff the differential feature closed ("Ship the
  differential feature"): `Exact`
  implements differential-dataflow 0.25's `IsZero`, `Semigroup`, `Monoid`,
  `Abelian` and `Multiply`
  in `src/exact/differential.rs`, `From<i8>` sits in `src/exact/ops.rs` on
  every feature set, eleven
  `differential_*` tests run on `ProcessBinary(3)` in
  `tests/differential.rs`, and
  `examples/dd_sum.rs` is the explode + count_total idiom. Two owner
  rulings landed and are in
  CLAUDE.md's standing decisions: the pin is 0.25 on a caret, and the
  closure's two unmaintained
  advisories are ignored by id in deny.toml. `timely = "0.31"` is a
  dev-dependency.
  
  This task is next: `Binned<K>` per DESIGN.md § Binned, in
  `src/binned.rs` with `pub use` from
  `src/lib.rs`, and `tests/binned.rs` as `mod binned;` of `tests/main.rs`
  (tests named `binned_*`,
  the verify filters on that; `scripts/check-test-topics.sh` fails on a
  topic file nobody names).
  What the tree gives you:
  
  - The grid is `Wide`'s: `Wide::LSB` (−1074) and `Wide::LIMBS` (34) in
  `src/wide.rs`;
  `Wide::from_parts(limbs, specials)` is the safe constructor `into_wide`
  wants, and
  `Wide::limbs()` / `Wide::specials()` are the readers
  `Wide::to_binned::<K>()` wants. The limb
  arithmetic helpers are in `src/limbs.rs`; `src/ieee.rs` decomposes a
  double into its
  mantissa, exponent and special class (`ieee::Special`), which the
  deposit's slicing needs.
  - The special counters are `[i64; 3]` as on `Wide`, so `into_wide`
  carries them across unchanged.
  - Zero-copy is the primary constraint: `Copy`, `repr(C)`, no padding, a
  `const` layout assertion
  the way `src/exact/mod.rs` and `src/wide.rs` pin theirs, no allocation
  anywhere.
  - The oracle for the error bound is an `Exact` fold (`Exact::from` +
  `Add`, or
  `extend_from_slice`), read through `to_f64`; `tests/support.rs` has the
  deterministic `Rng`
  (`finite`, `on_grid`, `any`) and `pow2` for the shuffles and the
  adversarial multisets. Do not
  run `Binned` through the laws kit's `Weight`: it is not a group, and the
  kit says so by failing.
  - The trybuild harness is `tests/ui.rs`: one `pass` case must sit beside
  the `compile_fail`
  cases or trybuild only runs `cargo check`, and `TRYBUILD=overwrite cargo
  test ui_` re-blesses
  a `.stderr` after you have read the diff. For the differential half, a
  case that names
  `Binned<3>` as a collection's diff (`scope.new_collection::<u32,
  Binned<3>>()`) should fail on
  `Binned<3>: Semigroup`. trybuild hands a case the crate's
  dev-dependencies, and
  differential-dataflow is an optional normal dependency, not a
  dev-dependency, so the case
  cannot name `differential_dataflow::...` until `differential-dataflow =
  "0.25"` is also listed
  under `[dev-dependencies]` (it unifies with the optional one; timely
  already is).
  - The DBSP half of the compile-fail test needs the `dbsp` feature, which
  does not exist yet
  (fl-06sggdc, on rustc 1.99.0 only). Either land the differential half
  here and leave the DBSP
  case to the dbsp task with a `relates:` edge, or hold this task's close
  on that one; that is a
  shape to read back to the owner, not to decide.
  
  The body's four test groups are the contract: the 1,000-shuffle and
  1–64-partition bit-identity,
  the `n·2^(−60(K−1))·max|x|` bound against `Exact`, the documented
  truncation on the fresh fold of
  `1e30, 1e-10, −1e30`, and the compile-fail cases. DESIGN.md § Binned
  is the spec and `cover`
  pins it on this task; `cover --repin` after any rewording. After it:
  fl-9kp61j0 (the golden
  hash; unlocks three), fl-06sggdc (dbsp; the lockfile-edition caveat is
  on its comments),
  fl-ajqrs9p, fl-c3ahdwc. The portfolio ask fl-cfx001b is still out.
---
`src/binned.rs` per DESIGN.md § Binned.

Tests `binned_*`:
- a fold is bit-identical under 1,000 shuffles and across 1–64 merge partitions;
- `into_wide().to_f64()` is within `n·2^(−60(K−1))·max|x|` of `Exact` on random and adversarial
  multisets;
- the history `1e30`, `1e-10`, `−1e30`, through a *fresh* fold, shows the documented truncation;
- `trybuild` cases under the features: `Binned<3>` fails to compile both as a differential-dataflow
  `Collection` diff and as a DBSP `OrdIndexedWSet` weight.

## log
- 2026-10-11T15:02Z created
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-binned @28d2c310aff6
- 2026-10-12T14:41Z handoff by claude (979ec15b-15d7-418f-857e-2b3284a67604)
