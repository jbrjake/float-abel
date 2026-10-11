---
id: fl-nwk80qf
title: "Make limb and counter overflow a named panic in every build on Wide and Fixed, so the headroom budgets are checked in release and never wrap"
category: crate/types
labels: [reproducible-floats]
docs:
  - docs/DESIGN.md#fixed-sp-fixed
  - docs/DESIGN.md#wide-sp-wide
verify: "all(lacks src/wide.rs /(?s)debug_assert!\\(\\s*!overflow/, lacks src/fixed.rs /(?s)debug_assert!\\(\\s*!overflow/, run cargo test package=float-abel every_build)"
seq: 43
status: doing
created: 2026-10-11T17:24Z
claimed-by: claude (bd2b6c06-c485-4bc5-9de3-2093f3abcd32)
covers:
  - ref: docs/DESIGN.md#sp-fixed
    sha: 8fa2eadaab9aab115d1312b30e795f024b3b0fcdf571a7817f12502a4690e1ae
handoff: |
  The tree at d33a0ee is green through `./scripts/regression.sh` (78 tests
  in the main package, 4 in
  `tests/noalloc`, workspace clippy, rustdoc, the MSRV build, cargo-deny).
  `Exact` is a 32-byte
  `Copy` plain word (`src/exact/mod.rs`): the compact tier's dyadic
  (`src/exact/compact.rs`) plus
  three `i32` counts, `product(a, b)` total and bit-identical to
  `Wide::deposit_product` through the
  helpers now in `src/ieee.rs`, `Mul<Exact>` in `src/exact/ring.rs` with
  integer special amounts,
  `PastCap { Width, Exponent, Count }`. There is no heap arm and no
  `alloc` feature. The owner's
  rulings are quoted in CLAUDE.md's standing decisions and in each task
  body.
  
  This task is next and is small: every `debug_assert!(!overflow, ...)` in
  `src/fixed.rs` (4 sites:
  `deposit_dyadic`, `AddAssign`, `Neg`, `Mul<i64>`) and `src/wide.rs` (8
  sites: `deposit_finite`,
  `bump`, `AddAssign` limbs and counters, `Neg` both, `Mul<i64>` both)
  becomes an unconditional
  `assert!` with the same named message. The three
  `#[cfg(debug_assertions)] #[should_panic]` tests
  (`tests/fixed.rs` `fixed_exhausting_the_headroom_is_caught_in_debug`,
  `tests/wide.rs`
  `wide_exhausting_the_headroom_is_caught_in_debug`, `tests/wide_group.rs`
  `wide_overflowing_a_special_counter_is_caught_in_debug`) lose the gate
  and are renamed
  `*_is_a_named_panic_in_every_build`; add a `Fixed` scalar-multiply case.
  `scripts/regression.sh`
  gains `cargo test --quiet --release --test main every_build >/dev/null`,
  since only a release run
  shows release semantics. Fix the two doc comments that say "wrap in
  release" (`src/wide.rs` type
  doc, `src/fixed.rs` type doc) and DESIGN.md § Fixed's "a
  `debug_assert!` guards the headroom";
  then `meshwork cover fl-nwk80qf docs/DESIGN.md#sp-fixed` and repin the
  live tasks `lint` names.
  Add the ruling to CLAUDE.md's standing decisions in one line (owner:
  "Yes, check in release").
  
  After it, in order: fl-zs79ct9 (DESIGN.md § ops and § Wide: products
  off the grid are refused by
  name on `Fixed` and `Wide`; § Wide still says "never refuses"),
  fl-nna1djz (`repr(C)` on `Wide`
  and `Fixed`, the `layout_*` tests), fl-te1748m (the conversions, now
  `TryFrom` into `Exact`), then
  Binned. The portfolio ask fl-cfx001b waits on the portfolio amending the
  plan; close it when a
  task there answers it. The sazed and leras tasks that name `Exact`
  (sazed S1/S3, leras L3) assume
  the old shape in their bodies; they live in those stores and are the
  owner's to re-read.
  
  The compact unit tests keep `Compact::from_f64`, `from_product` and
  `Overflow::Special` behind
  `#[cfg(test)]`; `Exact` never hands the tier a special.
  `CARGO_TARGET_DIR` is set outside
  ~/Documents by the repo's `.claude/settings.local.json`; export it by
  hand elsewhere.
relates: [fl-zs79ct9, fl-nna1djz, fl-te1748m, fl-cfx001b]
---
Owner's ruling, 2026-10-11, asked whether the bounds ruling extends to `Wide`'s and `Fixed`'s
headroom budgets and `Wide`'s special counters, which wrap silently in release: "Yes, check in
release". A wrapped sum is silently wrong, and a budget that is only asserted under
`debug_assertions` is not a bound.

In `src/fixed.rs` and `src/wide.rs`, every `debug_assert!(!overflow, ...)` on a limb add, negation
or scalar multiply, and on a special counter's add, negation or multiply, becomes an unconditional
check with a named panic that says which budget was exhausted (`Fixed: headroom exhausted ...`,
`Wide: a special counter overflowed ...`), in every build. The budgets themselves do not move:
`2^64` deposits at the ceiling on `Fixed`, `2^77` deposits of `f64::MAX` on `Wide`, `i64` counters.

Tests: the `#[cfg(debug_assertions)] #[should_panic]` tests in `tests/fixed.rs`, `tests/wide.rs`
and `tests/wide_group.rs` lose the `cfg` gate and are renamed `*_is_a_named_panic_in_every_build`.
`scripts/regression.sh` gains a leg that runs those tests under `--release`, because the point is
release semantics and a debug run cannot show them.

Docs: DESIGN.md § Fixed says "a `debug_assert!` guards the headroom" and § Wide's doc comment says
"past that they wrap in release": both say the check is unconditional. Repin the drifted clauses.

## log
- 2026-10-11T17:24Z created
- 2026-10-11T17:41Z open→doing — claimed by claude (bd2b6c06-c485-4bc5-9de3-2093f3abcd32)
- 2026-10-11T18:12Z cover docs/DESIGN.md#sp-fixed @8fa2eadaab9a
- 2026-10-11T18:12Z handoff by claude (bd2b6c06-c485-4bc5-9de3-2093f3abcd32)
