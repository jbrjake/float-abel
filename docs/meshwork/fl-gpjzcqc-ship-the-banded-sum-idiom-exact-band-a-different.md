---
id: fl-gpjzcqc
title: "Ship the banded SUM idiom as the fast path on the heap Exact — Exact::band, a differential operator that bands the key, count_totals per band and reduces the band totals into one Exact, the 3-worker test against a Wide oracle with the plain idiom's promotions as the control, and the criterion rows"
category: crate/differential
discovered-from: fl-h41w0tr
verify: "all(contains src/exact/mod.rs /pub fn band/, run cargo test package=float-abel differential_banded_)"
seq: 30
labels: [reproducible-floats, rf-critical-path]
docs:
  - docs/OVERFLOW.md#the-banded-idiom
  - docs/DESIGN.md#differential-dataflow-sp-differential
  - README.md#in-an-engine
status: open
created: 2026-10-15T15:15Z
needs: [fl-th7g69s]
handoff: |
  Next session ships the banded SUM idiom as the fast path here. What
  stands at 91abe38: the heap
  arm (09c92b5), its wire form (2ca2ddd), and the engine proof (9714758,
  findings on the closed
  task `search "Prove product folds"`): the plain `explode` +
  `count_total` idiom completes every
  history and the seeded 2^±300 multiset on the heap `Exact`, products
  fold, and the bench rows at
  91abe38 price the non-`Copy` word at 3 ns an update over the Copy word
  on same-scale data
  (BENCH.md § The end-to-end SUM; the one-worker typed ratio past the
  accepted band is the open ask
  fl-vy2nxc8, which banding does not answer: inside a band the diff is the
  same non-`Copy` word).
  What banding buys is the promoting bands: OVERFLOW.md § Cost has the
  niche form at 70–86 ns typed
  against banded 42–49 on the spread and mixed fixtures, and this task's
  rows re-take that on the
  real arm.
  
  Where to build. `Exact::band` must be `pub fn band` in src/exact/mod.rs
  literally, because the
  verify greps that file; mod.rs is 501 lines, so move something out first
  (src/exact/read.rs is 116
  lines and takes `band`'s neighbours) or re-set the verify to the file it
  lands in. The extension
  trait goes in src/exact/differential.rs (79 lines). Tests go in a new
  topic,
  tests/differential_banded.rs (tests/differential.rs is 512 lines and
  scripts/check-file-length.sh
  warns past 500), a `mod` of tests/main.rs, every test
  `differential_banded_*` for the verify.
  Reuse tests/differential_products.rs rather than re-prototype:
  `multiset()` is the seeded shape
  (10 epochs, 5 keys, 400 inserts an epoch over 2^±300, a third of the
  live rows retracted,
  specials on key 4) and returns the `Wide`-derived totals per (key,
  epoch); `histories(big, small)`
  and `pairs()` are the ten histories; `epochs_dataflow` feeds epochs from
  worker 0 and steps each
  to its probe; `count_total_updates(&totals)` is what `count_total` must
  emit. Lift those to
  `pub(crate)` and give the banded dataflow the same `Epochs` so the two
  idioms run one input. The
  control the body asks for is already measured on the plain side:
  `the_seeded_multiset` asserts 45
  of the 50 (key, epoch) totals are heap values; on the banded side count
  `is_compact()` over
  `count_total`'s per-band outputs, which must all be compact, while the
  reduce's one `Exact` per
  key is the cross-band sum and may promote. The allocation counter in
  tests/support.rs is
  per-thread and the dataflow runs on worker threads, so it does not see
  those allocations;
  `is_compact()` on the outputs is the count. In differential 0.25
  `probe(self)` consumes and
  returns the collection, `join` takes the other side by value, and a
  collection is cloned before
  any consuming operator; tests/differential_products.rs shows each.
  
  The bench: benches/dd_sum.rs with a banded row beside the plain `exact`
  rows on the same-scale
  fixture, and both on a mixed fixture over 2^±300
  (benches/support/mod.rs's `Rng::on_grid`; the
  oracle there is `Wide` per key, not the integer units). Build with
  `cargo bench --bench dd_sum
  --no-run` and run the binary in one quiet sitting: the script in the
  2026-10-16 session's
  scratchpad waited 750 s for the one-minute load under 3 (the desktop app
  and WindowServer held it
  at 3.8 with nothing of ours running) and the `isize` canaries landed
  within 0.3–5% of the earlier
  rows, which is the check a sitting passes. The rows go in BENCH.md §
  The end-to-end SUM under the
  heap-form table with the machine and the commit. README § In an engine
  then shows both idioms and
  says when to reach for the banded one; fl-v2msbby carries the rest of
  the outward docs and has a
  comment on what 9714758 already landed.
---

Owner, 2026-10-15, in session: "i like your option 4 so if there's a way to get the best of both
worlds, i'm all for it." With the heap arm (fl-heap in the batch of 2026-10-15; see the task
whose title starts "Restore the heap arm") the plain idiom is correct on every input, and the
banded idiom becomes the fast path for sums: inside a band the compact form never promotes, so a
banded `SUM` runs at the 32-byte word's speed with no allocation, where the plain idiom on mixed
magnitudes promotes and allocates. Measured on the Copy word (docs/OVERFLOW.md § Cost): banded
26.5 ns an update same-scale against the heap proxy's 33.5, and 43–50 against 101–119 on the
spread and mixed bands.

The idiom as prototyped (`band` and `cost`, attached to fl-h41w0tr and fl-t154dsp;
docs/OVERFLOW.md § The banded idiom): `band(x)` = ⌊(exponent of the leading bit + 1074) / 32⌋,
0..=65 finite and 255 for specials; `explode` into `Exact` and `count_total` per `(key, band)`;
`reduce` a key's band totals, at most 66, into one `Exact` (the heap form holds the cross-band
sum now; `Wide` is no longer needed for it) and emit it. Every diff differential forms inside a
band spans at most 84 + log₂ n bits, inside the width for n < 2⁴³ live rows per (key, band), so
no promotion happens there.

To build: `Exact::band(x: f64) -> u8` with the span bound in its rustdoc and a unit test that
every finite double's lowest set bit is within 84 bits of its band's top; behind `differential`,
an extension trait on `VecCollection<(K, u64), isize>` (the amount as bits) with one method that
runs the idiom and yields `(K, Exact)`; `tests/differential.rs` tests named
`differential_banded_*`: the ten histories and the seeded 2^±300 multiset with retractions and
specials on 3 workers against the `Wide` oracle per (key, epoch), and the control that gives the
idiom its reason: the same multiset through the plain idiom promotes (count the heap-form
outputs through `is_compact()`, or an allocation counter in the test) where the banded run
promotes nowhere; `benches/dd_sum.rs` rows for the banded idiom on the same-scale and a mixed
fixture beside the plain rows, recorded in BENCH.md § The end-to-end SUM. README § In an engine
shows both idioms and says when to reach for the banded one.

## log
- 2026-10-15T15:15Z created
- 2026-10-16T14:43Z handoff by claude (028f9a17-8bd2-4590-bb38-7e9820e83a28)
