---
id: fl-pwq0h8n
title: "Give Wide and Fixed differential's and DBSP's weight traits — IsZero, Semigroup, Monoid, Abelian, Multiply<isize>/<i64> and the DBSP set — so a product fold and a declared window can ride a dataflow without a newtype, with the 3-worker test and the criterion row"
category: crate/differential
discovered-from: fl-h41w0tr
verify: "all(contains src/wide.rs /differential/, run cargo test package=float-abel differential_wide_)"
seq: 40
labels: [reproducible-floats]
needs: [fl-t154dsp]
docs:
  - docs/OVERFLOW.md#options
  - docs/DESIGN.md#wide-sp-wide
  - docs/DESIGN.md#fixed-sp-fixed
status: open
created: 2026-10-15T15:15Z
---
Gated on fl-t154dsp. `Wide` is the one carrier that holds every sum of doubles and every row
product below 2¹⁰³⁷ with nothing to check, at 296 bytes a diff: 8× the plain `Exact` row and 16×
`isize` in fl-h41w0tr's rough sitting (docs/OVERFLOW.md § Cost). `Fixed256` is the 32-byte
declared window with a per-value door check. Neither has an engine trait today; the scratch
crates gave each a newtype with five of differential's.

To build: the differential impls on `Wide` and on `Fixed<LSB, LIMBS>` in the shape of
src/exact/differential.rs (`Multiply<Wide> for isize`/`i64` too; no `Wide × Wide`, stated in the
rustdoc with the bit count), the DBSP set in the shape of src/exact/dbsp.rs behind `dbsp`;
`tests/differential.rs` tests named `differential_wide_*` and `differential_fixed_*`: the five
histories on both pairs complete on 3 workers, a product fold through `deposit_product` at the
door, and `Fixed256` refusing 2¹⁰⁰⁰ at the door; a `benches/dd_sum.rs` row for `Wide` on the
same-scale fixture recorded in BENCH.md § The end-to-end SUM. DESIGN.md § Wide and § Fixed and
the trilemma row say which engine traits each carries.

## log
- 2026-10-15T15:15Z created
