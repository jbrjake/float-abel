---
id: fl-zxpq1wz
title: "Investigate why Exact panics inside differential-dataflow on ordinary mixed-magnitude data, and lay out panic-free options with a plain rule for what fits"
status: done
category: crate/types
labels: [reproducible-floats]
verify: "all(exists docs/OVERFLOW.md, contains docs/OVERFLOW.md /^## What fits/, contains docs/OVERFLOW.md /^## Options/)"
docs:
  - docs/DESIGN.md#exactness-bounds-sp-bounds
  - docs/DESIGN.md#differential-dataflow-sp-differential
seq: 5
created: 2026-10-14T23:28Z
attachments: [attachments/fl-zxpq1wz/probe-Cargo.toml, attachments/fl-zxpq1wz/probe-src-main.rs, attachments/fl-zxpq1wz/probe-src-bin-dd.rs, attachments/fl-zxpq1wz/probe-src-bin-tiers.rs, attachments/fl-zxpq1wz/probe-src-bin-door.rs]
---

Owner, 2026-10-14, in session: "you need to investigate this more, because we should not be
panicing like this. the whole point of this is to safely do float math inside dd. also your answer
is completely inscrutable. i have no idea how to know if a value fits."

What prompted it, measured this session on a scratch crate outside the tree (float-abel at
75c25fe, differential-dataflow 0.25, timely 0.31.0, `explode` + `count_total`, one key):

- Two ordinary doubles reach `Exact`'s 127-bit width: `1e12 + 1e-12` and `1 + 1e-23` are
  `PastCap::Width` from `checked_add`; `1e11 + 1e-11` and `1 + 1e-22` fit.
- Four histories that end at the same multiset {2^-1000}: epoch 0 `+2^1000`, epoch 1
  `-2^1000 +2^-1000` panics; `+2^1000 | +2^-1000 | -2^1000` panics; all three in one epoch
  panics; `+2^1000 | -2^1000 | +2^-1000` completes and reads 9.33e-302 on 1 and 3 workers.
  Whether a dataflow completes depends on which partial sums the engine forms, which the caller
  does not control.
- On 3 workers the panic does not end the process: the surviving workers spin at ~200% CPU until
  killed (16 min once, then 20 s per run under an alarm).
- No per-value check can say a double is safe to insert into an `Exact` diff: every double fits
  alone (`From<f64>` is total), and whether a sum fits depends on what else is in the running
  total at that moment. `checked_*` exist but differential calls `plus_equals`, which is `+=`.

Answer, with evidence, in `docs/OVERFLOW.md`:

1. Where in differential the panic fires (operator and call path, from a backtrace), and which
   partial sums a dataflow forms: per-epoch accumulation, batch consolidation, spine merges.
2. Why a panic on one worker hangs the others in timely 0.31.0, from timely's source, and whether
   a consumer can make it exit instead (`panic = "abort"`, or anything timely offers).
3. What a caller can check per value, before insert, that guarantees no panic downstream, for
   each tier: `Exact`, `Fixed` (`try_deposit` refuses per value at the door, then a 2^64-deposit
   budget), `Wide` (every sum of doubles fits for 2^63 adds of `f64::MAX`).
4. The options for panic-free float sums inside dd, each with what it keeps and what it costs
   against the standing decisions (32-byte `Copy` word, nothing allocates, `Mul<Exact>`, specials
   counted): at least `Wide` as a diff, `Fixed` as a diff, a declared window on `Exact`, a wider
   `Exact`, and why a 32-byte word cannot hold every sum of doubles (counting argument). Measured
   where a number decides it, labelled as measured or derived.
5. A plain-language rule a user can apply to know whether their data fits each option.

Investigation only: no change to the crate until the owner rules on the options.

## log
- 2026-10-14T23:28Z created
- 2026-10-14T23:28Z open→doing — claimed by claude (34b44aef-4de7-4c82-b211-e1cc92e50e7f)
- 2026-10-14T23:34Z doing→done — verify exit 0 @ 75c25fe+7
