---
id: fl-h41w0tr
title: "Spike the Exact panic inside differential-dataflow — a sum past 127 bits panics in plus_equals, whether it is reached depends on arrival order and history, and on 3 workers the panic leaves the process running"
status: doing
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
handoff: |
  Next session starts here, on the owner's word: "a high priority bug that
  needs a spike
  prioritized for next session". The body is the problem as measured,
  facts only; the owner asked
  that it carry no proposed fix, so start from the facts, not from
  docs/OVERFLOW.md's options.
  
  Proven this session: the five-history table and the pair limits in the
  body, the backtrace, the
  3-worker hang. Evidence: docs/meshwork/attachments/fl-zxpq1wz/ holds the
  scratch crate's
  Cargo.toml and the three binaries (dd, tiers, door); copy them into a
  scratchpad crate, keep
  CARGO_TARGET_DIR outside ~/Documents, and run a 3-worker case under an
  alarm
  (`perl -e 'alarm 20; exec @ARGV' …`), because a panicking 3-worker run
  does not exit.
  
  Not established: whether the worker count alone, with one history and
  one arrival order, can
  change panic or completion; which other operators (spine merges after
  compaction, `reduce`,
  `join`) form sums the table did not exercise.
claimed-by: claude (6cbb3399-1134-4aff-94e4-f8c1137cf60f)
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
