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
status: doing
created: 2026-10-15T15:33Z
handoff: |
  The test half landed at 9714758 (findings in the comment of the same
  date): tests/differential_products.rs
  and tests/dbsp_products.rs, both green on their toolchains, the docs
  amended. What remains is the
  bench half: benches/dd_sum.rs re-taken for the heap `Exact` at HEAD,
  typed and serialized, 1 and 8
  workers, beside `isize`, into BENCH.md § The end-to-end SUM with the
  machine (this aarch64 Mac,
  NEON) and the commit, and the kill-table row read against 2× (the owner
  accepted 1.8–2.2× for the
  Copy word in the ruling on the end-to-end SUM ratio, `search "Rule on
  the end-to-end SUM ratio"`,
  and is re-asked if the heap form moves it).
  
  The release bench binary at 9714758 is built:
  ~/.cache/cargo-target/float-abel/release/deps/dd_sum-369f3f9511ffa6c9
  (the hash is per target,
  not per content, so a rebuild overwrites it; `cargo bench --bench dd_sum
  --no-run` rebuilds in
  seconds warm). A sitting script that waits for the one-minute load under
  3 and no cargo, criterion
  or dd_sum process, up to 45 minutes, then runs the binary with `--bench`
  and appends to a log, was
  started in the 2026-10-16 session's scratchpad (sitting.sh, log
  dd_sum.log, a `.done` marker when
  it ends); it is not in the tree. The host's load sat at 3.9 all morning,
  carried by the Claude
  desktop app and WindowServer, nothing of ours, so the script may run
  past its wait. Judge the
  sitting by the `isize` canaries against BENCH.md's rows at 2a82388
  (214.9 ms serialized 1w,
  43.0 at 8w, 123.7 typed 1w, 28.2 at 8w on this machine): within a few
  percent is a clean
  sitting, 20–70% off is contested and is re-taken. The fixture is
  unchanged since 2a82388
  (benches/support/mod.rs's `nth_units` is a refactor with the same
  values), so the rows compare.
  `grep -A1 "^dd_sum/" <log>` reads the `time:` lines; the estimates are
  under
  `<target>/criterion/dd_sum_<row>/new/estimates.json`.
  
  Write the rows as a second table in BENCH.md § The end-to-end SUM
  headed by the commit and the
  form ("the heap form at 9714758, NEON"), with the ratio to `isize` and,
  with the cross-sitting
  caveat, to the Copy word's rows at 2a82388; the A/B against the saved
  control binary within the
  1.12×/1.20× budget is fl-p4dc5w5's sitting, not this one, so do not
  fold them. Peak RSS: the
  macOS high-water mark is allocator-driven (BENCH.md says the Linux
  figures are the ones to read)
  and the same-scale fixture never promotes, so state that RSS was not
  re-taken rather than take it.
  Then commit `bench(dd_sum): …`, `close fl-xf8hv25` (its verify is the
  differential_products_
  filter, warm), set the handoff on the task `prime` leads with, and
  commit the store.
claimed-by: claude (028f9a17-8bd2-4590-bb38-7e9820e83a28)
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
- 2026-10-16T14:06Z open→doing — claimed by claude (028f9a17-8bd2-4590-bb38-7e9820e83a28)
- 2026-10-16T14:23Z handoff by claude (028f9a17-8bd2-4590-bb38-7e9820e83a28)
- 2026-10-16T14:24Z handoff by claude (028f9a17-8bd2-4590-bb38-7e9820e83a28)

## comments
- 2026-10-16T14:23Z [claude (028f9a17-8bd2-4590-bb38-7e9820e83a28)] Findings at 9714758, the test half. Every history and product fold the body names holds on the
  heap `Exact` inside both engines, with no change to the adapters: src/exact/differential.rs and
  src/exact/dbsp.rs are untouched.
  
  tests/differential_products.rs, six tests, 0.05 s, all on one worker and three with serialized
  exchange (`ProcessBinary`), each watched failing on a deliberate break of its own expectation:
  - `the_five_histories`: A–E of docs/OVERFLOW.md on (1e12, 1e-12), (2^1000, 2^-1000) and
    (1e300, 1e-300), each landing exactly the updates its running totals predict through
    `count_total` and the small value as the last live read. History B holds the promoted
    `big + small` across an epoch as `count_total`'s data, so a heap value crosses the exchange as
    data too, not only as a diff. The epochs are fed by worker 0 and stepped to the probe before
    the next, as the spike's probes did.
  - `consolidate_updates_in_the_c_order`: the `repro` case as a unit test, both orders to
    `[small]` in the compact form; runs under miri.
  - `the_seeded_multiset`: the `band` probe's shape (10 epochs, 5 keys, 400 inserts an epoch over
    2^±300, a third of the live rows retracted, specials on key 4) through the plain idiom: 7,200+
    updates, all 50 (key, epoch) totals on the `Wide` oracle, 45+ of them heap values, key 4's
    carrying counts. The same shape panicked on one worker and hung on three on the Copy word.
  - `a_join_of_two_exact_weighted_collections`: 5 keys × 30 × 30 rows, `(Σaᵢ)(Σbⱼ)` as the ring
    computes it from the two heap sums equals the dataflow's fold of the 900 pairwise products
    (distributivity made structural by canonical form), and the `num-bigint` oracle in units of
    2^-2148 agrees; a retraction epoch of a third of the left rows moves the total with it.
  - `a_depth_three_join_chain`: 5 keys × 8³ triples, `(Σa)(Σb)(Σc)` against the oracle in units
    of 2^-3222, read through `Wide` (three 2^±300 factors sit inside its window; a fourth of that
    band would not, which is why the iterate test narrows the factor to 2^±100).
  - `iterate_carries_a_float_weight_through_three_rounds`: the body
    `inner.filter(round < 3).join(factors).map(round + 1).concat(roots).consolidate()` converges
    because round-3 rows stop advancing; rounds 2 and 3 carry 159- and 212-bit weights through the
    feedback loop, the arrangement and the exchange; 48 rows on the oracle in units of
    2^-1074(k+1).
  
  tests/dbsp_products.rs, two tests on rustc 1.99.0, both watched failing on a break:
  `aggregate_linear` promotes two accounts' accumulators past the width (one scaled ×2 and ×3 by the
  `ZWeight`), demotes each to the compact small value when the large rows are retracted, and deletes
  the groups when the rest go; and a dot-product circuit, `aggregate_linear(|pair| Exact::product(a,
  b))` over `Tup2<u64, u64>` rows with multiplicities 1..=3, folds 3 keys × 40 row products over
  2^±300 onto the `num-bigint` oracle, follows the retraction of half, and deletes each group when
  the rest go. `examples/dbsp_uda` builds on 1.99.0. Both clippy legs (default features, and
  `--features dbsp` on stable) and the rustdoc leg are clean.
  
  Docs: DESIGN.md § differential-dataflow and § DBSP, README § In an engine, and docs/OVERFLOW.md
  (opening, § What fits with a heap-arm row, § Why Exact panics, § Why one worker's panic hangs the
  rest, § The banded idiom) state what the heap arm holds and that the limb cap is the one panic
  left inside a dataflow, with `panic = "abort"` still the consumer's setting; that note was absent
  from the README before, and fl-v2msbby carries the rest of the outward-docs scope (comment there).
  
  Not done in this half: no limb-cap-in-a-dataflow test, by the body's ruling (a doc sentence, in
  OVERFLOW.md § Why one worker's panic hangs the rest and DESIGN.md § differential-dataflow).
