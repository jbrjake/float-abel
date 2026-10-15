---
id: fl-gpjzcqc
title: "Ship the banded SUM idiom — Exact::band, a differential operator that bands the key, count_totals per band and reduces the band totals into a Wide, the 3-worker adversarial test against a Wide oracle, and the criterion rows beside the plain idiom"
category: crate/differential
discovered-from: fl-h41w0tr
verify: "all(contains src/exact/mod.rs /pub fn band/, run cargo test package=float-abel differential_banded_)"
seq: 30
labels: [reproducible-floats, rf-critical-path]
needs: [fl-t154dsp]
docs:
  - docs/OVERFLOW.md#the-banded-idiom
  - docs/DESIGN.md#differential-dataflow-sp-differential
  - README.md#in-an-engine
status: open
created: 2026-10-15T15:15Z
---
Gated on fl-t154dsp. The idiom as prototyped and measured in fl-h41w0tr's spike (`band` and
`cost`, attached there; docs/OVERFLOW.md § The banded idiom): `band(x)` = ⌊(exponent of the
leading bit + 1074) / 32⌋, 0..=65 finite and 255 for specials; `explode` into `Exact` and
`count_total` per `(key, band)`; `reduce` a key's band totals into a `Wide` and emit the double.
Every diff differential forms is a sum inside one band, spanning at most 84 + log₂ n bits, inside
the width for n < 2⁴³ live rows per (key, band); nothing to check at the door.

To build: `Exact::band(x: f64) -> u8` (or a small `Band` newtype) with the span bound in its
rustdoc and a unit test that every finite double's lowest set bit is within 84 bits of its
band's top; behind `differential`, an extension trait on `VecCollection<(K, u64), isize>` (the
amount as bits) with one method that runs the idiom and yields `(K, Wide)` or the double's bits,
decide which and say why; `tests/differential.rs` tests named `differential_banded_*`: the ten
histories, and the seeded 2^±300 multiset with retractions and specials on 3 workers against a
`Wide` oracle per (key, epoch), with the plain idiom on the same multiset as the negative control
that must panic (`should_panic` on one worker, or `panic = "abort"` cannot be set per test, so
one worker); `benches/dd_sum.rs` rows for the banded idiom on the same-scale fixture and on a
mixed band, recorded in BENCH.md § The end-to-end SUM beside the plain `Exact` and `isize` rows;
`examples/dd_sum.rs` moved to the idiom.

Products stay out: a row's product is 106 bits and no band fits a fold of them; that is `Wide`'s
(the sibling task on the engine traits).

## log
- 2026-10-15T15:15Z created
