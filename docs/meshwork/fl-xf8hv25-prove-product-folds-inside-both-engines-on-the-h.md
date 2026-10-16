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
handoff: |
  Next session proves product folds inside both engines on the heap
  `Exact` here. What stands at
  2ca2ddd: the heap arm (09c92b5) and its wire form (2ca2ddd; the findings
  are on the closed wire
  task, `search "built at 2ca2ddd"`).
  A heap weight crosses differential's serialized exchange as its `encode`
  bytes through serde and
  bincode, which tests/exact_heap_ring.rs's
  `exact_heap_ring_a_promoted_total_crosses_serialized_exchange_on_three_workers`
  already shows on
  the plain `explode` + `count_total` idiom with the `Wide` oracle per
  (key, epoch); dbsp's archive
  carries a heap weight with its block beside the word (tests/dbsp.rs,
  `dbsp_an_archived_heap_*`).
  So the exchange and the archive are not this task's question; the
  products and the histories
  are.
  
  Where to build: tests/differential.rs is 512 lines, so the tests go in a
  new topic,
  tests/differential_products.rs, as a `mod` of tests/main.rs
  (scripts/check-test-topics.sh fails
  on a topic nobody names), every test named `differential_products_*`
  because the verify filters
  on it. Reuse `crate::differential::{netted, sum_dataflow, Row}` (a `Row`
  is `(key, bits)`,
  `sum_dataflow` the plain idiom, `netted(workers, closure)` runs it and
  consolidates), and the
  oracle helpers `units`, `wide_units` and `two_to` in tests/exact_heap.rs
  (`pub(crate)`);
  `product_units` in tests/exact_heap_ring.rs is private and reads a
  product fold's finite part in
  2^-2148 units through `Wide`, so lift it to `pub(crate)` or take it
  along. The five histories
  are docs/OVERFLOW.md's and tests/exact_heap.rs walks them through `+=`;
  the spike's `repro`
  (the C-order `consolidate_updates` case) and `band` probes are
  attachments on the closed spike
  task (`search "Spike the Exact panic"`). A `join` of two
  `Exact`-weighted collections needs
  `Multiply<Exact, Output = Exact>`, which src/exact/differential.rs has;
  `iterate` needs
  `Abelian`, also there. The dbsp half runs on `cargo +1.99.0 test
  --features dbsp --test main
  dbsp_`: extend tests/dbsp.rs's `sum_circuit` cases with rows that
  promote (1e300 beside 1e-300)
  and retract back, and the push gate already builds examples/dbsp_uda on
  that toolchain.
  
  The bench half: benches/dd_sum.rs re-taken for the heap `Exact` at HEAD,
  typed and serialized,
  1 and 8 workers, beside `isize`, into BENCH.md § The end-to-end SUM
  with the machine and the
  commit. fl-p4dc5w5 is the separate A/B sitting against the Copy word's
  control binary (saved in
  ~/.cache/cargo-target/float-abel-control/release/deps/, hash
  ab4b21425246350c) within the
  1.12×/1.20× budget; do not fold the two together, and check `uptime`
  and `ps aux | grep cargo`
  before either sitting (the build-environment memory has the recipe and
  the canaries). The wire
  change touched only heap values' bytes, so the same-scale fixture's
  serialized rows are the Copy
  word's bytes still.
  
  One edge to know: the open cap-edge task (`search "closed under
  retraction"`), filed from the
  wire task, says a value of exactly
  `EXACT_MAX_LIMBS` full limbs cannot be retracted (`v − v` refuses on
  the operands' spans plus a
  carry bit). The 2^±300 band of these tests is thousands of bits below
  the cap, so it does not
  reach it; a limb-cap case stays a doc sentence, as the body says.
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
- 2026-10-16T14:04Z handoff by claude (3f2084cf-748a-445c-adcf-4fe16d8173b8)
- 2026-10-16T14:05Z handoff by claude (3f2084cf-748a-445c-adcf-4fe16d8173b8)
