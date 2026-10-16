---
id: fl-cwwnyas
title: Draw the limb cap's edge in the laws kit's Exact strategy
status: open
category: crate/types
labels: [reproducible-floats]
discovered-from: fl-dchdzd4
relates: [fl-dchdzd4]
verify: run cargo test package=float-abel laws_closure_exact_at_the_cap
docs:
  - docs/DESIGN.md#exactness-bounds-sp-bounds
created: 2026-10-16T18:26Z
---

tests/laws.rs's `exact_full()` is the closure law's strategy for `Exact`: it draws every binade with
specials and products, but never a value within a bit of `EXACT_MAX_LIMBS`, so the closure law
(fl-x8fwfe3) could not have caught fl-dchdzd4's defect, a value at the cap's exact width that
could not be subtracted from itself; the wire tests found it by hand. The law is only as strong as
its strategy.

Add a strategy that lands at the edge: the eleven shapes of tests/exact_cap.rs's `edge_shapes()`
(`2^(C−1) + 3`, `2^C − 1`, `2^(C−1) − 1`, the powers at and above the cap, small integers) and
small sums of them, each a value the operations hold, and run `check_closure` on it with
`small_int()` and with itself in a test named `laws_closure_exact_at_the_cap_*`, so a regression
at the edge is the kit's to catch, not a hand-written case's. The sweep must stay cheap under miri:
a few hundred draws of 537-limb values, or `cfg(not(miri))` like the rest of tests/laws.rs.

## log
- 2026-10-16T18:26Z created
