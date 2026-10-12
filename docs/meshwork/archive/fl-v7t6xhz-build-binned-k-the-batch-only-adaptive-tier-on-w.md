---
id: fl-v7t6xhz
title: "Build Binned<K> — the batch-only adaptive tier on Wide's grid, its Add, the conversions, its error bound, and compile-fail tests that keep it out of the engine weight traits"
category: crate/types
labels: [reproducible-floats, rf-tail]
docs: [docs/DESIGN.md#binned-sp-binned]
needs: [fl-at6nh41]
seq: 100
verify: run cargo test package=float-abel binned_
status: done
created: 2026-10-11T15:02Z
covers:
  - ref: docs/DESIGN.md#sp-binned
    sha: bccae53a11b777c62dccc53b1eb5e9dca63b0b21e4909a0ec0a26a17bf03225d
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
- 2026-10-12T14:56Z open→doing — claimed by claude (745519d7-d30a-4336-9fa7-1462279793db)
- 2026-10-12T15:08Z cover docs/DESIGN.md#sp-binned @bccae53a11b7
- 2026-10-12T15:10Z doing→done — verify exit 0 @ d98adf7+3

## comments
- 2026-10-12T15:10Z [claude (745519d7-d30a-4336-9fa7-1462279793db)] Built at d98adf7, with two things for the owner to rule on; neither is ratified.
  
  1. The bins keep their own carries, and the word is 16·K + 32 bytes (80 at K=3), not the
  design's 8·K + index + counters (56 at K=3). The design's sketch, "carry-save with 4 headroom
  bits, so a carry propagates once per 16 deposits", is not order-independent once the index has
  moved: a carry that crossed into the bin above before a rise survives the drop of the bin it came
  from, while the same inputs deposited after the rise lose it. Deposit sixteen values whose low
  slices overflow bin 0, let the carry cross, then a 1e30 raises the index: the carry is in bin 1,
  kept. Deposit the 1e30 first: the sixteen low slices are each dropped at the door, no carry. Two
  results from one multiset. ReproBLAS, the model the design names, keeps each bin's overflow in
  that bin's own carry word, which is what carry-save means, so that is what was built. The shuffle
  and partition tests fail on the crossing variant (watched: seven of the fourteen tests, both
  bit-identity ones among them). The byte cost is a question, not a decision: with `i32` carries and
  an `i32` index the word is 64 bytes at K=3 and padding-free only for odd K, with a budget of 2³¹
  wraps; with the `i64` carries it is uniform for every K and the budget is 2⁶³. The slices are 64
  bits wide, on Wide's literal limb grid, so the bound tightened to n·2^(−64(K−1))·max|x| and the
  spec clause was reworded and repinned.
  
  2. The DBSP half of the compile-fail case ("Binned<3> fails to compile … as a DBSP
  OrdIndexedWSet weight") cannot be written before the `dbsp` feature exists, so it is filed on
  "Ship the dbsp feature" (fl-06sggdc) as a comment with a relates edge, and the differential half
  landed here (tests/ui/binned_as_differential_diff.rs, failing on `Binned<3>: Semigroup`, beside
  the Exact pass case that proves the feature is on in the harness). The previous handoff said that
  shape was the owner's to choose; this session's instruction was to continue, so the choice that
  blocks nothing was made and is reversible by moving one file.
  
  Not built, filed separately: `deposit_product` and `deposit_products` on Binned, which § ops says
  every tier has.
