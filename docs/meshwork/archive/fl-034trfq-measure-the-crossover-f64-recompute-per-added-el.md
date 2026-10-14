---
id: fl-034trfq
title: "Measure the crossover: f64 recompute per added element against Exact incremental in differential as the group grows, and write the decision memo"
status: done
category: crate/measure
labels: [reproducible-floats]
relates: [fl-bbevtyv]
verify: "all(contains BENCH.md /^## The crossover/, exists docs/CROSSOVER.md, exists benches/crossover.rs)"
docs:
  - BENCH.md#what-reproducibility-costs
  - docs/DESIGN.md#the-problem
created: 2026-10-14T18:24Z
---

Owner, 2026-10-14, in session: "To understand whether we move forward from benchmarking or
continue to look for performance improvements, I need to know a number, and any others you think
are relevant. The one I want is the cost of f64 recompute adding one element at a time as the
collection size grows. Versus the cost of our floats the way dd uses them to incrementally add
one element without needing full recompute, as collection size grows. [...] This project only
becomes broadly useful if it makes sense to turn to it because you can't do floats incrementally
in dd or dbsp. And it only makes sense to turn to it for that if it's still faster than recompute
of all the floats despite all the overhead of using float-abel. Figure this out and any other
context you think I should absolutely have to make this decision and report back to me here as
well as in a markdown file."

The measurement: `benches/crossover.rs`. One worker, one group preloaded with N rows (N from 10²
to 10⁶, untimed), then one element at a time, each in its own epoch stepped to the probe, timed
per element. Three dataflows on the same rows: `reduce` over `f64` in the data, the recompute
path differential offers today (O(group) per change); `explode` + `count_total` with `Exact`, the
crate's path; the same with `isize`, the floor and the fixed-point workaround. Beside them the
no-engine floor: a `Vec<f64>` re-summed per push against one `Exact +=`. Both architectures.

Deliverables: BENCH.md § The crossover with the rows, and docs/CROSSOVER.md, the decision memo
the owner asked for: the crossover group size, how each path scales, and what the number does
and does not settle. Unmeasured and said so: the DBSP side.

## log
- 2026-10-14T18:24Z created
- 2026-10-14T18:24Z open→doing — claimed by claude (c5fe58ea-7ba0-49b4-9459-0877a7ec2d6d)
- 2026-10-14T19:20Z doing→done — verify exit 0 @ dc4273f+1

## comments
- 2026-10-14T18:40Z [claude (c5fe58ea-7ba0-49b4-9459-0877a7ec2d6d)] Owner, 2026-10-14, in session, on a first cut that put the f64 recompute inside differential
  as a `reduce`: "No no not in dd." The recompute side is plain f64 with no engine, a Vec<f64> of
  the group re-summed per change; the incremental side is Exact inside differential with all the
  engine does per change. benches/crossover.rs is built that way: `crossover/exact|fixed-point/
  serialized|typed/N` one change an epoch at N = 10²..10⁶, `crossover/batched/...` a thousand
  changes an epoch at N = 10⁴..10⁶, `crossover/recompute/f64 re-sum, vectorised|chain/N`, and
  `crossover/no-engine/Exact deposit, one change`. Oracle-checked on both exchanges and both batch
  shapes before timing (count of outputs, then the netted total against integer units).
  
  A finding on the way: differential-dataflow 0.25.1's `reduce` panics in reduce.rs ("failed to
  find index", the capability search at the output step) on a key whose group was loaded in one
  epoch and then changed in a later epoch, down to one preloaded row and one change, on typed and
  serialized exchange alike; the suite's `differential_reduce_puts_exact_or_isize_out` passes
  because it has one epoch. Not pursued further (the owner moved the baseline out of dd), and not
  filed upstream (CLAUDE.md § Boundaries). Note for anyone reaching for `reduce` across epochs on
  0.25.1.
  
  Smoke on NEON before the sitting, typed exchange, N = 1000, one change an epoch: Exact 4.43 µs a
  change, isize 4.37 µs. The epoch is the cost; the word is invisible at this granularity.
- 2026-10-14T18:50Z [claude (c5fe58ea-7ba0-49b4-9459-0877a7ec2d6d)] NEON sitting, 2026-10-14 14:41–14:45, load 2.7–4.8 (sampled every 30 s; no cargo or sazed
  process; spotlightknowledged and syspolicyd were the load), bench source as committed at fa2e7b5,
  rows in BENCH.md § The crossover. One change an epoch through differential, flat from N = 10² to
  10⁶: Exact 3.98–4.10 µs typed, 4.52–4.58 µs serialized; isize 3.94–4.04 and 4.44–4.54. A thousand
  changes an epoch, per change: Exact 21.3–22.6 ns typed, 36.6–37.4 serialized; isize 10.0–10.2 and
  19.3–19.7. Re-sum of a Vec<f64> per change: vectorised 5.3 ns / 62.0 / 751 / 7.55 µs / 77.2 µs at
  10²..10⁶ (0.075 ns a value from 10⁴ up); chain 14.6 ns / 409 / 4.90 µs / 49.8 / 499 (0.50 ns a
  value from 10⁴ up; 0.15–0.41 below, criterion's iterations overlapping on the core). Exact deposit
  with no engine 4.71 ns. Crossovers by interpolation: one change an epoch 8,300 (chain) / 54,000
  (vectorised) typed and 9,300 / 60,000 serialized; a thousand an epoch 116 / 362 and 151 / 604;
  isize 8,200 / 53,000, 9,200 / 60,000, 69 / 177, 111 / 325. The epoch is the cost at one change an
  epoch (Exact and isize within 3%); the word moves the batched crossover by 1.7–2× and the
  one-change crossover by 2%. AVX2 sitting pending behind the owner's tpch binary on jons-amd-mini.
- 2026-10-14T19:19Z [claude (c5fe58ea-7ba0-49b4-9459-0877a7ec2d6d)] AVX2 sitting on jons-amd-mini, 2026-10-14 15:12–15:16 after the guard waited out the owner's
  tpch binary (zero tpch processes in every 30 s sample, load 1.0–1.5), bench source byte-identical
  to fa2e7b5 (sha1 checked both sides), rows in BENCH.md § The crossover. One change an epoch:
  Exact 14.77–14.95 µs typed and 15.37–15.52 serialized from 10² to 10⁵, isize 14.60–14.74 and
  15.46–15.62; the 10⁶ rows 11.50–12.94 on every diff and exchange, 15–25% under, the longer
  untimed preload the only difference, read as the host's clock and left unverified. A thousand
  changes an epoch, per change: Exact 69.1–71.4 ns typed, 102.4–104.4 serialized; isize 26.4–30.2
  and 43.4–47.2 (Exact 1.9–2.7× isize, above dd_sum's 2.1× at one worker). Re-sum per change:
  vectorised 9.8 ns / 87.3 / 885 / 9.46 µs / 364 µs (0.09 ns a value to 10⁵, 0.36 at 10⁶ out of
  cache); chain 47.8 ns / 672 / 6.92 µs / 69.4 / 740. Exact deposit with no engine 13.16 ns.
  Crossovers: one change an epoch 20,800 (chain) / 113,000 (vectorised, interpolated across the
  cache bend; 150,000 at the in-cache rate) typed, 21,200 / 113,000 serialized; a thousand an epoch
  132 / 800 and 180 / 1,180; isize 20,600 / 112,000, 21,700 / 114,000, 60 / 321, 96 / 518. Same
  reading as NEON: the epoch is the cost at one change an epoch, the word moves the batched
  crossover by 1.9–2.5× here (1.4–2× on NEON) and the one-change crossover by under 2%.
  docs/CROSSOVER.md is the memo, written for the owner's decision.
