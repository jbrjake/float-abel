---
id: fl-v112c79
title: "Run the four older laws on the full-range Exact strategy — group, ring, canonical and consolidation checks on every binade 2^±300 with specials and on row products, timed first, so the kit's sweep covers the heap form and not the compact word alone"
status: open
category: crate/laws
labels: [reproducible-floats]
discovered-from: fl-x8fwfe3
verify: run cargo test package=float-abel laws_exact_holds_every_law_on_every_binade
docs:
  - docs/DESIGN.md#the-laws-kit-sp-laws-kit
seq: 65
created: 2026-10-16T17:02Z
---

Found while adding the closure law (fl-x8fwfe3). tests/laws.rs runs the four older laws on
`Exact` through `exact()`, doubles within 56 binades of each other, "inside the width by
construction", which the spike (fl-h41w0tr) named as the reason the kit could not see the
compact form's cap. The heap arm holds every sum and product now, and the closure law runs on
`exact_full()`, every binade 2^±300 with the specials in, and on `exact_products()`; the group,
ring, canonical and consolidation laws still run on the narrow strategy alone, so nothing asserts
that a fold of 32 heap values lands the same bits under 1,000 shuffles, that `a + b − b == a`
across a promotion and a demotion, that structural `Eq` and `Hash` agree across both forms on
drawn data, or that random partitions with pruning land the plain fold when most partials are
heap values. tests/exact_heap.rs covers each on fixed inputs; the kit's proptest sweep over the
same laws is what a stranger would run, and `Exact` should pass it on its whole range.

To build: `laws_exact_holds_every_law_on_every_binade` in tests/laws.rs running the four older
checks on `exact_full()` (with `small_int()`, `exact_integer()` and `exact_full()` as the ring's
multipliers where the amounts stay integers, and the specials' dyadic amounts where they do not),
timed first: the group check folds 32 weights 1,000 times per case over 256 cases, so a heap
value per step may put the test over the suite's budget, in which case the shuffles or the cases
for this one strategy are reduced with the reason in the test's doc. The closure test took 4.4 s
with the whole laws topic at 16db257, so the budget is minutes, not seconds.

## log
- 2026-10-16T17:02Z created
