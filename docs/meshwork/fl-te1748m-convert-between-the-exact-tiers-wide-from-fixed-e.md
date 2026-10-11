---
id: fl-te1748m
title: "Convert between the tiers — Wide: From<Fixed<..>> exact; Exact: TryFrom<&Wide> and TryFrom<&Fixed<..>> refused past the width; Wide::try_from(&Exact) refused past the sum range"
status: open
category: crate/types
labels: [reproducible-floats]
needs: [fl-atkbp6q]
relates: [fl-at6nh41]
verify: run cargo test package=float-abel convert_
docs:
  - docs/DESIGN.md#operations-and-conversions-sp-ops
seq: 55
created: 2026-10-11T16:32Z
covers:
  - ref: docs/DESIGN.md#sp-ops
    sha: 63e9572f1cf144a73be825703e1b020e3cdbf1f4882dd181f76ef89dadfb91d9
---

DESIGN.md § Operations and conversions names four conversions between the exact tiers that no
other task implements: `Wide: From<Fixed<LSB, LIMBS>>`, `Exact: From<Wide>` and
`Exact: From<Fixed<LSB, LIMBS>>`, all exact, and `Wide::try_from(&Exact)`, which returns `Err`
past the sum range. Nothing lossy gets a `From`; the `Binned` conversions belong to the Binned task.

`From<Fixed>` into `Wide` is exact only when `LSB ≥ -1074` (a finer grid holds values below
`Wide`'s unit) and `Fixed`'s ceiling lies inside `Wide`'s range, so the impl carries a `const`
assertion on `(LSB, LIMBS)` rather than a runtime refusal. The shift is `LSB + 1074` limbs-and-bits
onto `Wide`'s grid, sign-extended. `Wide::try_from(&Exact)` refuses a finite part below the unit or
at the ceiling, and a special amount that is not an integer count, by name.

Tests `convert_*` in `tests/convert.rs`: every `Fixed256` state round-trips through `Wide` and
`Exact` bit for bit on random multisets; `Wide` states with specials round-trip through `Exact`;
`try_from(&Exact)` refuses by name past the range and accepts at the edge; the `const` assertion is
a `trybuild` compile-fail case for `Fixed<-1076, 20>`.

## log
- 2026-10-11T16:32Z created
- 2026-10-11T16:32Z cover docs/DESIGN.md#sp-ops @cf4faaa1f100
- 2026-10-11T17:41Z cover docs/DESIGN.md#sp-ops @63e9572f1cf1
