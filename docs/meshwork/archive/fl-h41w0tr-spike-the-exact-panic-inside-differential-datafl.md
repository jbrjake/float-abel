---
id: fl-h41w0tr
title: "Spike the Exact panic inside differential-dataflow — a sum past 127 bits panics in plus_equals, whether it is reached depends on arrival order and history, and on 3 workers the panic leaves the process running"
status: done
category: crate/differential
labels: [bug, rf-critical-path, reproducible-floats]
discovered-from: fl-zxpq1wz
verify: contains docs/meshwork/fl-h41w0tr-spike-the-exact-panic-inside-differential-datafl.md /^- 20[0-9][0-9]-[0-9][0-9]-[0-9][0-9]T[0-9:]+Z \[[^]]+\] Spike findings:/
docs:
  - README.md#the-contract
  - docs/DESIGN.md#differential-dataflow-sp-differential
  - docs/DESIGN.md#exactness-bounds-sp-bounds
seq: 1
created: 2026-10-14T23:39Z
attachments: [attachments/fl-h41w0tr/probe-src-bin-repro.rs, attachments/fl-h41w0tr/probe-src-bin-band.rs, attachments/fl-h41w0tr/probe-src-bin-cost.rs]
---

Owner, 2026-10-14, in session: "we should not be panicing like this. the whole point of this is to
safely do float math inside dd"; "i have no idea how to know if a value fits"; and on this
ticket: "a high priority bug that needs a spike prioritized for next session".

**What `Exact` holds.** A finite part `m · 2^exp` with `|m| < 2^127` and `|exp| ≤ 2^30`, and
`i32` counts of NaN, +∞ and −∞ (src/exact/mod.rs). A result whose mantissa needs more than 127
bits at the aligned exponent is past the cap: the operators panic with "Exact: past the cap: the
mantissa needs more than the 127-bit width", and the `checked_*` methods return `PastCap::Width`.
CLAUDE.md § Standing decisions: "Cap panic plus checked API".

**Under `differential`.** `Semigroup::plus_equals` is `+=` (src/exact/differential.rs), so the
cap inside a dataflow is a panic.

**Pairs of values, `checked_add`:** `1e11 + 1e-11` fits, `1e12 + 1e-12` is past the cap;
`1 + 1e-22` fits, `1 + 1e-23` is past; `1 + 2^-126` fits, `1 + 2^-127` is past; `1 + 5e-324`
and `f64::MAX + 5e-324` are past.

**Where it panics.** Backtrace, innermost first: `PastCap::exceed` … `Exact::plus_equals` ←
`consolidate_updates_slice_slow` ← `consolidate_updates_slice` ← `consolidate_updates_from` ←
`consolidate_updates` ← `Vec::consolidate_into` ← `ContainerChunker::finish` ← `arrange_core`.
`consolidate_updates_slice_slow` (differential-dataflow 0.25.1, src/consolidation.rs) sorts with
`sort_unstable_by` on `(data, time)` only, then folds each run's diffs with `plus_equals` in
slice order.

**Dataflow runs.** One key, `explode` + `count_total`; `|` separates epochs; every history ends
at the multiset {small}:

| history | 1e12, 1e-12 | 2^1000, 2^-1000 |
|---|---|---|
| A: `+big` \| `−big +small` | panic | panic |
| B: `+big` \| `+small` \| `−big` | panic | panic |
| C: `+big +small −big` | panic | panic |
| D: `+big` \| `−big` \| `+small` | completes | completes |
| E: `+big −big +small` | completes | completes |

Each cell is the same on 1 and 3 workers. C and E are the same three updates in one epoch, in two
arrival orders. The completed runs read the same double on 1 and 3 workers.

**On more than one worker.** On 1 worker the panic exits the process with status 101. On 3
workers, `timely:work-1` panicked and the process did not exit: one run spun at ~200% CPU for 16
minutes until killed, and histories A, B and C (2^1000 pair) each ran past a 20 s alarm. `WorkerGuards::drop`
(timely_communication 0.31.0, src/initialize.rs) joins the worker threads in index order.

**Checking a value.** `From<f64>` accepts every double. The `checked_*` methods take two `Exact`
values. The crate has no method that takes one value and reports whether inserting it into a
collection will reach the cap; in the runs above, that depends on the other values in the key
and on the order differential sums them.

**The documented contract.** README.md § The contract: "the same multiset lands the same bits
under any order, worker count, merge tree, consolidation timing or architecture". In the runs
above, the same multiset completes or panics depending on order and history.

**Repro.** The probe sources are attached to fl-zxpq1wz (docs/meshwork/attachments/fl-zxpq1wz/):
a scratch crate with float-abel at 75c25fe as a path dependency, differential-dataflow 0.25.1,
timely 0.31.0, run on aarch64-apple-darwin. `SCENARIO=B cargo run --bin dd -- -w 1` (2^1000 pair),
or `TIER=exact PAIR=everyday SCENARIO=B cargo run --bin tiers -- -w 1`. The table's 1e12 column
was run from a build with `panic = "abort"`, where a panic exits with status 134 on 1 and 3
workers; the 2^1000 column from the default build and again with `panic = "abort"`.

Close when the spike's findings are recorded in this task's comments, opening "Spike findings:".

## log
- 2026-10-14T23:39Z created
- 2026-10-14T23:39Z handoff by claude (34b44aef-4de7-4c82-b211-e1cc92e50e7f)
- 2026-10-15T14:53Z open→doing — claimed by claude (6cbb3399-1134-4aff-94e4-f8c1137cf60f)
- 2026-10-15T15:15Z doing→done — verify exit 0 @ 57e1354+11

## comments
- 2026-10-15T15:13Z [claude (6cbb3399-1134-4aff-94e4-f8c1137cf60f)] Spike findings: the panic is real, it is the arithmetic doing what the 127-bit width says, and it
  is the consequence of dropping the heap arm without replacing the role the original design gave
  it. One small separate bug sits at the boundary. A fix that keeps the 32-byte word is prototyped
  and measured below. Everything here was run this session on this tree (57e1354, float-abel as a
  path dependency of the scratch crate attached to this task and fl-zxpq1wz; differential-dataflow
  0.25.1, timely 0.31.0, aarch64-apple-darwin, rustc 1.99.0).
  
  **The arithmetic is right.** 1e12 is 5¹²·2¹², a 28-bit mantissa at 2¹²; 1e-12 is 53 bits reaching
  down to 2⁻⁹². Aligned, their sum occupies 2⁻⁹² … 2⁴⁰: 132 bits, past 127. Derived independently in
  Python integers from the doubles' bits: 1e12+1e-12 needs 132, 1+1e-23 needs 130, 1e11+1e-11 needs
  126 and 1+1e-22 needs 126, matching `checked_add`'s fits/refuses on each. `Compact::checked_add`
  aligns to the lower exponent, raises the other operand, adds in `i128`, and refuses exactly when
  the result's magnitude reaches 2¹²⁷, with one exception (the boundary bug below).
  
  **Reproduced.** `tiers` (TIER=exact PAIR=everyday) on one worker: A, B and C panic with "Exact:
  past the cap: the mantissa needs more than the 127-bit width" and the main thread's "Worker
  panic"; D and E complete reading 1e-12. On three workers C does not exit: killed by a 20 s alarm
  (exit 142); E completes. Without a dataflow (`repro`, attached): differential 0.25.1's public
  `consolidate_updates` over `[(1,0,+1e12), (1,0,+1e-12), (1,0,-1e12)]` panics, and over
  `[(1,0,+1e12), (1,0,-1e12), (1,0,+1e-12)]` consolidates to `[1e-12]`; the 2^±1000 pair the same.
  That function (consolidation.rs:116-127) sorts with `sort_unstable_by` on `(data, time)` and folds
  each run's diffs in the slice's order, so which partial sums exist is a property of the input
  order and of the sort, promised by nothing.
  
  **Two classes of failure, and they differ.**
  1. *The key's true running total at some epoch does not fit.* History B holds 1e12 and 1e-12 live
     at epoch 1. `count.rs:108,120` folds a key's diffs in time order, so this total is formed
     whatever the arrival order, and no 127-bit `Exact` can hold it. Only a wider carrier can.
  2. *The true total fits at every epoch but a partial sum does not.* A and C against D and E: the
     same multiset, panic or completion by order. Partial sums are formed in `consolidate_updates`
     (consolidation.rs:127), the batcher's chain merge (merge_batcher.rs:315), the spine's chunk
     merges (trace/chunk/vec.rs:264 and :363, the second collapsing times after compaction, so a
     retraction at a later time meets an insertion at an earlier one), and the running totals in
     count.rs and threshold.rs. The columnar paths (columnar/*) are not on the default path.
  Both come from one fact: the set of `Exact` values is not closed under `+`, and closure is the
  first group axiom. The laws kit cannot see it: tests/laws.rs draws `Exact` from `double(-28, 28)`,
  "doubles whose leading bits lie within 56 binary decades of each other", inside the width by
  construction, and the kit has no closure law.
  
  **Where it came from.** DESIGN.md at 622ad8a: `Exact` is "a 24-byte compact tier that carries the
  same-scale case without touching the heap, and an exact heap tier for everything else", the heap
  arm "reached by promotion only"; § Measured: "The compact tier is a same-scale optimisation, and
  the exact tier is mandatory for correctness, not a rare fallback"; § Hardening: "no panics on
  input … group ops never fail. The one panic is the named `EXACT_MAX_LIMBS` invariant", reachable
  "only through ≥ 16 nested float × float joins on extreme values". fl-atkbp6q (owner, 2026-10-11:
  "in general i approve dropping the heap arm", zero-copy paramount) removed the arm and made the
  width a panic on the operators. DESIGN.md § Exactness bounds still reads "Real columns look like
  the former; the exact tier exists for the latter", and the tier it names no longer exists. The
  trilemma row was rewritten to "a sum or product within a 127-bit span", which is accurate; README
  § The contract ("the same multiset lands the same bits under any order…") does not say a sum can
  fail, and § In an engine documents the idiom that panics on 1e12 then 1e-12.
  
  **One stupid bug, separate and small.** `raise` in src/exact/compact.rs refuses when the raised
  operand alone would need 128 bits, before the add. So `Exact::from(2^127).checked_add(&Exact::from(-1.0))`
  is `Err(Width)` although 2¹²⁷−1 is 127 one-bits, and `Exact::try_from(&(Wide::from(2^127) +
  Wide::from(-1.0)))` is `Ok` (both run in `repro`). Only a sum whose raised operand overshoots by
  exactly one bit and cancels back under 2¹²⁷ is affected; it explains nothing above. Fix: when
  `bits + by == 128`, raise by `by − 1` and add that operand twice, or add in a 129-bit
  intermediate; red test first.
  
  **Residual caps under any sum.** The counts are `i32`: 2³¹−1 live NaN or ∞ rows in one key is
  `PastCap::Count`. The exponent cap is unreachable by sums of doubles.
  
  **A fix that keeps the 32-byte word, prototyped and measured** (`band` and `cost`, attached).
  Put the double's band in the key, `band(x) = ⌊(exponent of the leading bit + 1074) / 32⌋`, 0..=65
  for finite values and 255 for specials; `explode` into `Exact` and `count_total` per
  `(key, band)`; then `reduce` each key's band totals, at most 66, into a `Wide` and emit the double.
  Every `Exact` diff differential then forms is a sum of values inside one band, whose set bits lie
  in a window of 32 + 52 bits, so any partial sum of n of them, in any order, with retractions and
  multiplicities, spans at most 84 + log₂ n bits: inside 127 for n < 2⁴³ rows per (key, band). Each
  band is a `Fixed` window chosen by the value, which is what makes a per-value check unnecessary:
  every double fits its band, and nothing in the dataflow depends on what else is in the key. The
  reduce re-folds a key's bands, not its rows. Measured on this tree:
  - the ten histories (A–E × the 1e12 and 2¹⁰⁰⁰ pairs) complete on 1 and 3 workers, each reading
    the small value exactly;
  - a seeded multiset, 7,207 updates over 10 epochs on 5 keys, magnitudes uniform over 2^±300 with
    full mantissas and either sign, a third of the live rows retracted each epoch, NaN and ±∞ on
    one key, 20 bands occupied: all 50 (key, epoch) reads equal a `Wide` oracle bit for bit, on 1
    and 3 workers; the same multiset through the plain idiom panics on 1 worker and hangs on 3.
  Rough cost, release, one worker, Thread allocator, 10⁶ updates over 16 keys in one epoch, best of
  three, host load 8 so the ratios are what to read: same-scale (units of 2⁻²⁰ below 2²⁰): `isize`
  14.7 ns an update, `Exact` plain 30.4, `Exact` banded 31.1, `Wide` as the diff 242.6; mixed
  2^±300: `Exact` plain panics, `Exact` banded 50.6, `Wide` 208.5. Banding costs 2% over the plain
  idiom on same-scale data and runs 5–8× under `Wide` as the diff.
  
  **What the banded idiom does not do.** Products. A row's product has a 106-bit mantissa, so a
  band would have to be 127 − 106 − log₂ n bits wide: useless. A fold of products inside a dataflow
  is `Wide`'s (BENCH.md § Where the width caps: 42% of same-scale products refuse in an `Exact`
  fold), with `deposit_product` refused at the door past 2¹⁰³⁷, and `Wide × Wide` does not exist.
  `Exact × Exact` inside a dataflow remains a single row's product, and a join's `R × R2` of two
  sums is where the width goes first.
  
  **Not established.** A history that panics on one worker count and completes on another
  (plausible: input containers chunk differently, so consolidation's partial sums differ; not
  demonstrated). The spine-after-compaction sum triggered in a run (read from the source). `Wide`'s
  and the banded idiom's cost under criterion, at eight workers, on AVX2, and with serialized
  exchange.
  
  **Plan.** The ruling is fl-t154dsp's, re-stated there with these numbers. Filed behind it: ship
  the banded SUM idiom as a crate helper and a differential operator with the 3-worker adversarial
  test and the criterion rows; give `Wide` (and `Fixed`) the engine traits for product folds and
  for callers who want no idiom; rewrite README § The contract, § In an engine and DESIGN.md
  § Exactness bounds, § Exact and § differential to state the rule and the idiom, with the
  `panic = "abort"` note for multi-worker consumers. Filed with no gate: the `raise` boundary fix,
  and a closure law in the kit with an adversarial strategy that `Exact` must fail and `Wide` pass.
