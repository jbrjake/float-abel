---
id: fl-t154dsp
title: "Rule on differential's diff type — Wide, Fixed256, a windowed Exact, or Wide and Fixed256 together — so a caller can know before inserting that a dataflow will not panic"
status: open
category: rulings
labels: [reproducible-floats]
discovered-from: fl-zxpq1wz
relates: [fl-zxpq1wz]
verify: contains docs/meshwork/fl-t154dsp-rule-on-differential-s-diff-type-wide-fixed256-a.md /2026-[0-9][0-9]-[0-9][0-9] owner ruled/
docs:
  - docs/OVERFLOW.md#options
  - CLAUDE.md#standing-decisions
  - docs/OVERFLOW.md#the-banded-idiom
  - docs/OVERFLOW.md#cost
seq: 4
created: 2026-10-14T23:35Z
handoff: |
  The spike (fl-h41w0tr, closed 2026-10-15) settled the facts: the panic
  is real and
  arithmetic-correct, a plain `Exact` diff is not closed under `+`, and
  the cause is the dropped
  heap arm with nothing in its role. Read fl-h41w0tr's "Spike findings:"
  comment and
  docs/OVERFLOW.md first; the probes are attached to fl-h41w0tr (`repro`,
  `band`, `cost`) and to
  fl-zxpq1wz (`dd`, `tiers`, `door`), and build as a scratch crate with
  float-abel as a path
  dependency, `CARGO_TARGET_DIR` outside ~/Documents.
  
  This task is the owner's ruling. Ask in session, reading back the four
  options in this task's
  latest comment with the numbers, and the recommendation (option 4: the
  banded idiom as the
  documented SUM idiom, `Wide` with the engine traits for product folds).
  Record the ruling here,
  quoted, then close.
  
  Gated on this ruling: fl-gpjzcqc (ship the banded idiom: `Exact::band`,
  the operator, the
  3-worker adversarial test, the criterion rows), fl-pwq0h8n (engine
  traits on `Wide` and
  `Fixed`), fl-v2msbby (the outward docs and the `panic = "abort"` note).
  Not gated, ready now:
  fl-7r8qney (the `raise` one-bit overshoot, red test first) and
  fl-x8fwfe3 (a closure law in the
  laws kit with the 2^±300 negative control).
  
  Not established, if the owner asks: a history that panics on one worker
  count and completes on
  another (plausible by chunking, not demonstrated); the spine's
  after-compaction sum triggered in
  a run; the banded idiom's and `Wide`'s cost under criterion, at eight
  workers, on AVX2, with
  serialized exchange. The rough sitting ran at host load 8, so its ratios
  hold and its absolute
  numbers do not.
---

Owner, 2026-10-14, in session: "we should not be panicing like this. the whole point of this is to
safely do float math inside dd."

The question, from docs/OVERFLOW.md (fl-zxpq1wz): which type does differential carry as its diff,
so that a caller can know before inserting that a dataflow will not panic?

The baseline: today the diff is `Exact`, 32 bytes, which takes any single double but panics
inside a dataflow when a sum it forms needs more than 127 bits. 1e12 then 1e-12 in one key does
it. No per-value check can rule that out, and whether it happens depends on arrival order, and
on three workers a panic hangs the process unless it is built with `panic = "abort"`.

The options in docs/OVERFLOW.md § Options:
1. `Wide` (296 B, every double, nothing to check); its cost inside differential is unmeasured.
2. `Fixed256` (32 B, a declared window, `Fixed256::try_from(x)` checks each value; NaN and ±∞
   refused).
3. `Exact` with a declared window (32 B, a 63-bit window, narrower than option 2).
4. Options 1 and 2 together, `Exact` kept for in-thread folds with the `checked_*` methods. This
   is the recommendation, with `Wide`'s cost measured inside differential first.

Each option is a change to standing decisions that this ruling would make: `Exact` stops being
the engine weight in options 1, 2 and 4, and `Mul<Exact>` inside a dataflow ("we absolutely need
exact multiplication") goes with it unless a `Fixed256 × Fixed256 → Wide` product is built.

Close by recording the owner's ruling in this task's comments, quoted.

## log
- 2026-10-14T23:35Z created
- 2026-10-15T15:16Z handoff by claude (6cbb3399-1134-4aff-94e4-f8c1137cf60f)

## comments
- 2026-10-15T15:14Z [claude (6cbb3399-1134-4aff-94e4-f8c1137cf60f)] fl-h41w0tr's spike (2026-10-15) adds an option and the numbers, docs/OVERFLOW.md § The banded
  idiom and § Cost. The panic is real and arithmetic-correct: a plain `Exact` diff is not closed
  under `+`, so a key whose true total fits can still panic on a partial sum differential forms,
  and a key whose true total does not fit panics in any order.
  
  The options now:
  
  1. **The banded idiom, shipped**: the double's band (leading-bit exponent / 32) goes in the key,
     `count_total` per (key, band), a `reduce` folds a key's ≤ 66 band totals into a `Wide`. `Exact`
     stays the 32-byte diff; every double fits its band; nothing to check; sums only. Measured on
     this tree: the ten histories and a 7,207-update 2^±300 multiset with retractions and specials
     land a `Wide` oracle's bits on 1 and 3 workers, where the plain idiom panics and hangs. Rough
     cost 31.1 ns an update against plain `Exact`'s 30.4 and `isize`'s 14.7 on same-scale data.
  2. **`Wide` as the diff**: every double, products at the door, 296 bytes, 242.6 ns an update in
     the same sitting (8× plain `Exact`, 16× `isize`).
  3. **`Fixed256` as the diff**: 32 bytes, a declared window, a door check; specials refused.
  4. **1 and 2 together**: the banded idiom for sums of any doubles, `Wide` for product folds and
     for a caller who wants no idiom.
  
  Recommendation: 4, with the banded idiom as the documented SUM idiom; it keeps every standing
  decision. Tasks gated on this ruling: fl-h41w0tr's spawned banded, wide-traits and docs tasks.
  The ruling is the owner's, in session, quoted here when it comes.
