---
id: fl-bbevtyv
title: "Rule on the end-to-end SUM ratio — Exact runs 2.8–4.2× isize on serialized exchange and 1.9–2.8× on typed, past the design's 2× kill line: accept, fix the wire form, or revisit the layout"
status: open
category: rulings
labels: [reproducible-floats]
discovered-from: fl-e4jjqf1
relates: [fl-e4jjqf1]
verify: contains docs/meshwork/fl-bbevtyv-rule-on-the-end-to-end-sum-ratio-exact-runs-2-8-4.md /2026-[0-9][0-9]-[0-9][0-9] owner ruled/
docs:
  - docs/DESIGN.md#targets-and-kill-conditions-sp-kills
  - BENCH.md#the-end-to-end-sum
seq: 152
created: 2026-10-13T16:09Z
---

Owner's call, raised by Measure (fl-e4jjqf1). DESIGN.md § Targets and kill conditions, row "the
type is cheap enough to be a diff": the differential-dataflow end-to-end `SUM` over 10⁷ updates,
`Exact` against `isize`, at 1 and 8 workers; over 2× means revisiting the layout, 1.3× or under
is done. BENCH.md § The end-to-end SUM, commit fdf0575, same-scale fixture, oracle-checked:

| time per run | NEON Exact | NEON isize | ratio | AVX2 Exact | AVX2 isize | ratio |
|---|--:|--:|--:|--:|--:|--:|
| serialized, 1 worker | 864.5 ms | 206.5 ms | 4.19× | 2,816.8 ms | 727.5 ms | 3.87× |
| serialized, 8 workers | 150.3 ms | 42.8 ms | 3.51× | 503.8 ms | 181.0 ms | 2.78× |
| typed, 1 worker | 328.1 ms | 118.0 ms | 2.78× | 1,585.4 ms | 607.4 ms | 2.61× |
| typed, 8 workers | 57.8 ms | 27.2 ms | 2.12× | 323.9 ms | 168.9 ms | 1.92× |

The kill condition fires on every serialized row (the probe's shape, `ProcessBinary`) and on three
typed rows of four. The split: on typed exchange the word itself costs 21 ns an update over
`isize` on NEON at one worker (the compact add, the 48-byte update tuple through consolidation's
sort against 24, and `From<f64>`); serialized exchange adds 45 ns an update net of `isize`'s own
serialization, which is the 33-element `u8` tuple written and read one element at a time through
serde (src/wire/serde_impls.rs) and validated by `from_bytes` on decode. Two thirds of the
serialized gap is the wire form, one third the word.

Options the numbers suggest, none chosen:
(a) Accept the ratio and restate the kill condition: `Exact` is the exact weight, and 2–4× an
    integer diff is its price.
(b) Serialize `Exact` as one 33-byte `serde_bytes`-style blob (or `serialize_newtype_struct` over
    `[u8; 33]`) instead of a 33-element tuple, and re-measure the serialized rows; the typed rows
    bound what that can reach (2.1–2.8×).
(c) Revisit the layout, as the row says: a narrower word for the diff (the counts out of the hot
    word, or a 16-byte compact form) with `Wide`/`Exact` behind it.
The owner writes the line this task's verify looks for, dated.

## log
- 2026-10-13T16:09Z created

## comments
- 2026-10-13T16:26Z [claude (10cb7eaf-2ccb-44b5-b4e4-5396d637aa48)] Owner, 2026-10-13, in session: "add a task for the next session to speed up the differential forms, like the serialization stuff you think is a likely culprit". Filed as , seq 140, leading the queue; it moves the number and this ruling on what ratio is acceptable stays open.
- 2026-10-13T16:26Z [claude (10cb7eaf-2ccb-44b5-b4e4-5396d637aa48)] Correction: the speed-up task the owner asked for is fl-s9hcne6 (the previous comment's id was blank: the add had failed on its verify's quoting).
