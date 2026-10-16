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
status: done
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
- 2026-10-16T14:43Z handoff by claude (028f9a17-8bd2-4590-bb38-7e9820e83a28)
- 2026-10-16T15:04Z handoff by claude (028f9a17-8bd2-4590-bb38-7e9820e83a28)
- 2026-10-16T15:10Z open→doing — claimed by claude (94f54cd7-858c-4c70-ad69-f1f2ca548e35)
- 2026-10-16T16:34Z doing→done — verify exit 0 @ e00d92e+2

## comments
- 2026-10-16T15:26Z [claude (94f54cd7-858c-4c70-ad69-f1f2ca548e35)] 16db257 lands the code half: Exact::band in src/exact/mod.rs (is_compact, is_zero and is_finite moved to read.rs to keep it under 500 lines), BandedSum::banded_sum in src/exact/differential.rs with timely as an optional dependency of the feature for the Timestamp and TotalOrder bounds, tests/differential_banded.rs (five tests; the break check merged four bands in the control dataflow and 449 of 481 band totals promoted, so the compact-bands assertion is live), the lifted products helpers (feed_epochs, history_totals, pub(crate) multiset), benches/dd_sum.rs with banded rows beside exact on the same-scale fixture and both on the mixed 2^±300 fixture against a Wide oracle, README § In an engine's second idiom as a doctest, DESIGN.md § differential-dataflow, OVERFLOW.md § The banded idiom in the shipped form, and the CHANGELOG bullet. Pending: the quiet sitting of the release binary dd_sum-6802e010ed8c390f (20 rows) and its rows in BENCH.md § The end-to-end SUM.
- 2026-10-16T16:34Z [claude (94f54cd7-858c-4c70-ad69-f1f2ca548e35)] Banded idiom findings, at 16db257 (code, tests, docs) and the bench commit after it (rows).
  
  What shipped. `Exact::band(x)` in src/exact/mod.rs: ⌊(exponent of the leading bit + 1074) / 32⌋,
  0..=65 finite, 255 for the specials, zero in band 0, built from `ieee::decompose` (the odd
  mantissa's top bit plus the exponent), with the 84 + log₂ n span bound in its rustdoc.
  `BandedSum::banded_sum` in src/exact/differential.rs on `VecCollection<'scope, T, (K, u64),
  isize>` with `K: ExchangeData + Hash` (Hash, not Hashable: `(K, u8)` is Hashable only through
  the blanket impl over Hash) and `T: Timestamp + TotalOrder + Lattice`, which is why `timely`
  became an optional dependency of the feature. The reduce's closure types are inferred from the
  method's return type, because the no_std lib cannot name `Vec` without `alloc`. is_compact,
  is_zero and is_finite moved to src/exact/read.rs to keep mod.rs at 495 lines.
  
  What the tests prove (tests/differential_banded.rs, five tests). Every finite double's lowest
  set bit is within 84 bits of its band's top and a power of two at every exponent visits all 66
  bands; a fold of 2¹⁰ full-mantissa doubles of either sign is compact at every step on every
  band, and one value each from bands 30 and 34 promote at the first add; the fifteen histories
  (three pairs) land the plain idiom's updates on 1 and 3 workers; the seeded multiset lands the
  plain idiom's updates on both worker counts with every band total compact across the twenty
  bands occupied (24..=42 and 255) while more than half the key totals are heap values; and
  +2^14 | −2^13 −2^13 emits nothing at epoch 1 with both band totals live. The break check:
  merging four bands in the control dataflow promoted 449 of 481 band totals and failed the
  cancellation test's band assertions, so the compact-bands assertion is live.
  
  The rows (BENCH.md § The end-to-end SUM, one clean sitting, isize canaries within 0.4–4.4% of
  9714758's). Same-scale: banded costs 1.06–1.14× plain (1.8 ns an update typed, 3.9 serialized,
  at one worker). Mixed 2^±300: banded runs in 0.54–0.64× the plain idiom's time (1.56–1.86×
  faster); plain costs 2.3–2.9× its same-scale row there, 60 ns an update on either exchange, the
  promotion and limb arithmetic; banded 1.24–1.47× its own, 16–18 ns, the nineteen bands per
  account. The first sitting was contested by the owner's other project's `applied` binary (load
  7–15; eight-worker isize canaries 9–42% high) and was discarded; the sitting script now waits on
  that process too (build-environment memory).
  
  Cross-sitting: plain's same-scale rows read 4–9% above 9714758's with isize within 4%, so
  exact / isize reads 2.05–2.52× here against 1.93–2.38× there and DESIGN.md § kills' 1.9–2.4×.
  The plain path's code is unchanged; the binary carries the new rows and three fixtures.
  A/B in one quiet window (load 2.41), the two binaries back to back on the one-worker typed rows:
  9714758's binary 283.7 ms exact and 121.6 isize (2.33×), 16db257's 318.8 and 128.5 (2.48×). The
  isize dataflow and fixture are identical in both, so the shift is the bench binary's shape (6%
  on isize, 12% on exact), not the lib's plain path. Consequence for fl-p4dc5w5: its A/B against
  the Copy word's control binary (the 8-row harness) needs a variant of the same harness shape, or
  the harness shift is read as the word's.
