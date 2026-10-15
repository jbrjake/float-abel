---
id: fl-xf8hv25
title: "Prove product folds inside both engines on the heap Exact — the five histories complete on 3 workers in every order, a float × float join folded by count_total and a depth-3 join chain land an integer oracle's bits, the DBSP aggregate and UDA still build on 1.99, and the dd_sum rows are re-taken beside isize"
category: crate/differential
labels: [reproducible-floats, rf-critical-path]
discovered-from: fl-t154dsp
needs: [fl-th7g69s, fl-mrdqpay]
seq: 16
verify: run cargo test package=float-abel differential_products_
docs:
  - docs/DESIGN.md#differential-dataflow-sp-differential
  - docs/DESIGN.md#dbsp-sp-dbsp
  - docs/OVERFLOW.md#why-exact-panics-in-a-dataflow
status: open
created: 2026-10-15T15:33Z
---
Owner, 2026-10-15, in session: "allow product fols inside a dataflow". The adapters in
src/exact/differential.rs and src/exact/dbsp.rs are the group operators by name and change
little with the heap arm; what this task proves is that the panic of fl-h41w0tr is gone and that
products fold.

Tests in tests/differential.rs named `differential_products_*` (a new file if the 500-line
target is crossed): the five histories of docs/OVERFLOW.md on both pairs through the plain
`explode` + `count_total` idiom on 3 workers with serialized exchange, each completing and
reading the small value exactly, with the C-order `consolidate_updates` case of fl-h41w0tr's
`repro` as a unit case; the seeded 2^±300 multiset of fl-h41w0tr's `band` probe through the plain
idiom on 3 workers against the `Wide` oracle per (key, epoch); a `join` of two `Exact`-weighted
collections over mixed-magnitude weights folded by `count_total`, and a depth-3 join chain, each
against a `num-bigint` oracle (the dev-dependency fl-heap adds), bit for bit on 1 and 3 workers;
`iterate` with a float carrier through three rounds of `Exact × Exact`; a limb-cap case that
panics by name on one worker under `panic = "abort"` is not a test, it is a doc sentence. Under
`dbsp` on rustc 1.99.0: `aggregate_linear` scaling and retracting a promoted accumulator, and
`examples/dbsp_uda` building. Then `benches/dd_sum.rs` re-taken for the heap `Exact` on the
same-scale fixture at 1 and 8 workers, typed and serialized, beside `isize`, recorded in
BENCH.md § The end-to-end SUM with the machine and commit, and the kill-table row read against
2× (owner's prior acceptance of 1.8–2.2× for the Copy word, fl-bbevtyv, is re-asked if the heap
form moves it). `examples/dd_sum.rs` stays the plain idiom; the banded fast path is
fl-gpjzcqc's.

## log
- 2026-10-15T15:33Z created
