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
seq: 4
created: 2026-10-14T23:35Z
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
