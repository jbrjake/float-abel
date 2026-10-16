---
id: fl-pwq0h8n
title: "Give Wide and Fixed differential's and DBSP's weight traits — IsZero, Semigroup, Monoid, Abelian, Multiply<isize>/<i64> and the DBSP set — for an engine that needs a Copy diff or a declared window, with the 3-worker test and the criterion row"
category: crate/differential
discovered-from: fl-h41w0tr
verify: "all(contains src/wide.rs /differential/, run cargo test package=float-abel differential_wide_)"
seq: 70
labels: [reproducible-floats]
docs:
  - docs/OVERFLOW.md#what-fits
  - docs/DESIGN.md#wide-sp-wide
  - docs/DESIGN.md#fixed-sp-fixed
status: done
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
- 2026-10-16T17:06Z handoff by claude (84727215-1ab8-4972-b9d9-cd4dd0eb435e)
- 2026-10-16T17:24Z handoff by claude (84727215-1ab8-4972-b9d9-cd4dd0eb435e)
- 2026-10-16T17:25Z handoff by claude (84727215-1ab8-4972-b9d9-cd4dd0eb435e)
- 2026-10-16T17:26Z open→doing — claimed by claude (cd1130c2-56bf-432b-bcb1-8ed1c474e8fa)
- 2026-10-16T18:10Z doing→done — verify exit 0 @ ac9c361+2

## comments
- 2026-10-16T17:59Z [claude (cd1130c2-56bf-432b-bcb1-8ed1c474e8fa)] Landed in four commits on main: 7e472de moves Wide's and Fixed's operators into src/wide/ops.rs and src/fixed/ops.rs (pure move, room under the file-length target); 89df467 the differential traits (IsZero, Semigroup, Monoid, Abelian, Multiply<i64>/<isize> on either side, From<i8>; Fixed's From<i8> pinned by a const assertion HoldsIntegers to grids at or below one with a ceiling above 2^7, with tests/ui/fixed_from_i8_above_one.rs blessed on 1.99.0) and tests/differential_words.rs (13 tests, differential_wide_* and differential_fixed_*: the five histories on 1 and 3 workers with Wide on all three pairs and Fixed256 on the 1e12 pair, Fixed256 refusing 2^1000 and 1e-300 at the door, a product fold through deposit_product against num-bigint with a retraction epoch, retraction nets to nothing, join/semijoin/antijoin with isize both orders, count_core/distinct_core From<i8>, specials on Wide); 45c9115 the DBSP set (HasZero, MulByRef<i64>/<isize>, SizeOf, IsNone, SupportsRoaring, by-ref Add/Sub/Neg/AddAssign/SubAssign on both words for dbsp's blankets) with tests/dbsp_words.rs (12 tests, all 22 dbsp_ green on 1.99.0); 440ac8c the wide/{1,8}w/{serialized,typed} bench rows. Two decisions the handoff left open, decided from dbsp 0.364's source, not ruled by the owner: (1) no HasOne on either word, because HasOne reaches nothing without Mul<Self>: dbsp's RingValue = GroupValue + Mul<Output=Self> + MulByRef<Output=Self> + HasOne, and neg/minus/differentiate on a custom-weight stream bound the weight on ZRingValue through the leaf's Neg (trace/layers/leaf.rs), so those do not compile for Wide or Fixed and the docs say so; the weighted batch, merge and aggregate_linear (DBWeight + MulByRef<ZWeight>) do. (2) The archived form is the word itself through rkyv's derive with #[archive(as = "Wide")] / #[archive(as = "Fixed<LSB, LIMBS>")] and the Serialize derive, Deserialize/OrdRepr hand-written as a copy and Ord's cmp: no hand-written unsafe fn resolve, so #![deny(unsafe_code)] stays lifted only in src/simd/ and src/exact/heap.rs; the derive's unsafe is an external macro's and the lint does not fire. Fixed's limbs field needs #[omit_bounds] because the derive's own [u64; LIMBS]: Archive where-clause shadows the impl the array's archived type normalises through (E0308 inside the derive otherwise). clippy was installed for the 1.99.0 toolchain this session (rustup component add) so the dbsp feature gets a strict lint pass; it is clean.
