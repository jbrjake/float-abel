---
id: fl-pwq0h8n
title: "Give Wide and Fixed differential's and DBSP's weight traits — IsZero, Semigroup, Monoid, Abelian, Multiply<isize>/<i64> and the DBSP set — for an engine that needs a Copy diff or a declared window, with the 3-worker test and the criterion row"
category: crate/differential
discovered-from: fl-h41w0tr
verify: "all(contains src/wide.rs /differential/, run cargo test package=float-abel differential_wide_)"
seq: 70
labels: [reproducible-floats]
docs:
  - docs/OVERFLOW.md#options
  - docs/DESIGN.md#wide-sp-wide
  - docs/DESIGN.md#fixed-sp-fixed
status: open
created: 2026-10-15T15:15Z
needs: [fl-th7g69s]
---

With the heap arm (owner, 2026-10-15: "yes let's do heap and allow product fols inside a
dataflow") `Exact` holds every sum and every product fold as a diff, so `Wide` is no longer the
carrier for product folds inside an engine. What remains for this task: `Wide` as the `Copy` diff
for an engine whose weight slot must be `Copy` or castable from a mapping (the role the plan gave
it: leras's `Monoid` is `Copy`), and `Fixed<LSB, LIMBS>` as the 32-byte declared window with a
per-value door check and no allocation ever. Neither has an engine trait today; the scratch
crates gave each a newtype with five of differential's.

To build, after the heap arm lands: the differential impls on `Wide` and on `Fixed<LSB, LIMBS>`
in the shape of src/exact/differential.rs (`Multiply<Wide> for isize`/`i64` too; no
`Wide × Wide`, stated in the rustdoc with the bit count), the DBSP set in the shape of
src/exact/dbsp.rs behind `dbsp`; `tests/differential.rs` tests named `differential_wide_*` and
`differential_fixed_*`: the five histories on both pairs complete on 3 workers, a product fold
through `deposit_product` at the door, and `Fixed256` refusing 2¹⁰⁰⁰ at the door; a
`benches/dd_sum.rs` row for `Wide` on the same-scale fixture recorded in BENCH.md § The
end-to-end SUM (the rough figure is 198 ns an update against `isize`'s 12.8, docs/OVERFLOW.md
§ Cost). DESIGN.md § Wide and § Fixed and the trilemma row say which engine traits each carries.

## log
- 2026-10-15T15:15Z created
