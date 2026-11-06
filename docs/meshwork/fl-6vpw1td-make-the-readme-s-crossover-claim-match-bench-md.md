---
id: fl-6vpw1td
title: "Make the README's crossover claim match BENCH.md: 'always faster past 120k' sits inside the AVX2 bracket of 115,000 to ~170,000 that BENCH.md's in-cache rate gives"
status: open
category: crate/measure
relates: [fl-hzrz626]
verify: lacks README.md /more than 120k/
docs:
  - README.md#what-it-costs
created: 2026-11-04T16:30Z
---

README.md § what it costs says "float-abel's always faster than recompute once you're adding up
streams of more than 120k items". BENCH.md § The crossover does not support "always" at 120k:

- Its table's AVX2 (Ryzen 7 5700U) one-change crossover against the vectorised re-sum is 115,000,
  but that figure is a linear interpolation between 10⁵ and 10⁶, across the cache bend. BENCH.md's
  own next sentence: "the one-change crossover against it lies between the interpolated figure in
  the table and the 160,000 the in-cache rate gives." fl-034trfq's comment put the same bracket's
  top at 150,000 at fa2e7b5.
- From the table's rows at f24b051: the vectorised re-sum is 9.50 µs at 10⁵ (0.095 ns a value) and
  `Exact` in differential is 15.84–16.52 µs a change, so at the in-cache rate the re-sum costs what
  the engine's change does near 167,000–174,000 rows. At 120,000 rows that rate gives a re-sum of
  about 11.4 µs, under the engine's change.
- No size between 10⁵ and 10⁶ is measured, so the true AVX2 crossover is unknown inside
  115,000–~170,000; 120k sits inside that bracket.

Found while checking a blog post's technical claims against the crate (the post repeats the
README's 120k). Fix: state a figure every measured row supports (for instance "past about 200k on
both machines measured", with NEON's 53,000–60,000 as the "often before then"), or measure the
vectorised re-sum on the Ryzen at sizes between 10⁵ and 10⁶ and state the measured crossover.
BENCH.md's own bracket sentence stays; the README sentence moves to match it.

## log
- 2026-11-04T16:30Z created
