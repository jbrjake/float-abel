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
status: open
created: 2026-10-11T17:24Z
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
