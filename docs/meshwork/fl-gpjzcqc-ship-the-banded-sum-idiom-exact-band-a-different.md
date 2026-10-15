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
