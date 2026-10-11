---
id: fl-te1748m
title: "Convert between the tiers — Wide: From<Fixed<..>> exact; Exact: TryFrom<&Wide> and TryFrom<&Fixed<..>> refused past the width; Wide::try_from(&Exact) refused past the sum range"
status: done
category: crate/types
labels: [reproducible-floats]
needs: [fl-atkbp6q]
relates: [fl-at6nh41, fl-z91b7pq, fl-a2wrx7w, fl-6shtzpj, fl-v7t6xhz, fl-cfx001b]
verify: run cargo test package=float-abel convert_
docs:
  - docs/DESIGN.md#operations-and-conversions-sp-ops
seq: 55
created: 2026-10-11T16:32Z
covers:
  - ref: docs/DESIGN.md#sp-ops
    sha: 76b8d17699d6eb4378b8c22fc6369091acb28d580fa1bc7a2717b7219ea03ee3
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
- 2026-10-11T18:12Z cover docs/DESIGN.md#sp-ops @f7da3b4f752d
- 2026-10-11T18:28Z cover docs/DESIGN.md#sp-ops @7675d055f5a4
- 2026-10-11T18:35Z handoff by claude (0f58a8d2-1e20-4d5a-ab10-caf961d3912f)
- 2026-10-11T18:45Z open→doing — claimed by claude (9595457b-430f-4785-af6d-c23dc1a77cde)
- 2026-10-11T18:59Z cover docs/DESIGN.md#sp-ops @76b8d17699d6
- 2026-10-11T18:59Z doing→done — verify exit 0 @ a849bad+1

## comments
- 2026-10-11T18:59Z [claude (9595457b-430f-4785-af6d-c23dc1a77cde)] Built at a849bad. Decisions: (1) 'past the sum range' is read as Wide's product ceiling, 2^1037, so Wide::try_from(&Exact) shares the one window check with deposit_product (Wide::try_deposit_dyadic) and every input to Wide stays under the ceiling that the 2^77-deposit headroom budget assumes; DESIGN.md § ops now says so by name. (2) The 'non-integer special amount' refusal is moot: the POD Exact's counts are i32, so there is no amount to refuse, and the Exact->Wide error type is OutOfWindow with NotFinite never produced. (3) The const assertion on (LSB, LIMBS) is a post-monomorphisation error that cargo check never evaluates, so the trybuild set carries a pass case (tests/ui/fixed256_into_wide.rs) to force cargo build beside the two compile-fail cases; the .stderr files were written by rustc 1.99.0 and will need TRYBUILD=overwrite when stable's E0080 wording moves. (4) Closed tasks that pin sp-ops were left as re-open candidates, as the earlier clause edits were.
