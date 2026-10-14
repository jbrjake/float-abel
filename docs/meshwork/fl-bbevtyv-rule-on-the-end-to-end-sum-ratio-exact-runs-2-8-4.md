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
- 2026-10-14T13:21Z [claude (16e02ca3-15ad-4c80-bfb4-53e89fee757d)] Option (b) measured, benches/wire.rs at a22fae7, BENCH.md § The wire form: timely's three bincode
  passes on one exchanged batch (sized, written into the slab, read back), ns per update, the three
  summed. NEON: `isize` 8.7; `Exact` as today's 33-element `u8` tuple 47.8; a 7-element tuple of the
  fields `(u8, u64, u64, i32, i32, i32, i32)` 26.8; one `serialize_bytes` call 14.8. AVX2: 16.2;
  115.6; 55.8; 33.5. The cost is per serializer call on the write side, not per byte: bincode 1 writes
  each element with one `write_all` on the slab, a 1-byte copy and an 8-byte one at about the same
  price, so the tuple's write is 41 of its 48 ns on NEON and 89 of 116 on AVX2, while the sizing pass
  is near free and the read is cheap even byte by byte. Net of `isize` the tuple costs 39 ns an
  update on NEON and 99 on AVX2, which is the 45 and the 111 ns the end-to-end rows attribute to the
  wire form at one worker; the bytes form would leave 6 and 17, the fields form 18 and 40.
  
  Each candidate changes DESIGN.md § Wire format's clause that a format writing a tuple's elements
  back to back emits `to_bytes` byte for byte: the bytes form adds the format's own length prefix
  (8 bytes on bincode 1, so 41 an `Exact`; 1 on postcard, so 34), and the fields form keeps bincode's
  33 bytes exactly but not postcard's, whose varints change with the value, so tests/wire.rs's
  postcard assertion goes either way. serde_json carries the bytes form as the same array of numbers
  it carries today. Option (b) is therefore two options: (b1) `serialize_bytes`, the larger saving,
  with a prefix; (b2) the field tuple, no prefix on bincode, bincode's bytes unchanged. Neither is
  built: the build task waits on this ruling.
- 2026-10-14T13:30Z [claude (16e02ca3-15ad-4c80-bfb4-53e89fee757d)] Owner, 2026-10-14, in session, after BENCH.md § The wire form: "running bincode 3 times is crazy if
  it could be avoided, and if there is a way we can avoid it by making a small modification to the
  design that doesn't impact capabilities and does improve performance, i am all for it". The three
  passes are timely's and the crate cannot cut them; read as the go-ahead on option (b) in its (b1)
  form, one `serialize_bytes` call, which makes each pass about one serializer call and leaves
  `to_bytes`, `from_bytes`, FORMAT.md's encoding, the validating decoder, JSON's array and DBSP's
  rkyv path as they are; the clause that a tuple-writing format emits `to_bytes` with no prefix is
  the modification. Built under fl-11k5gz6. The ratio question, (a) against (c), stays open here for
  the re-measured rows.
- 2026-10-14T16:42Z [claude (16e02ca3-15ad-4c80-bfb4-53e89fee757d)] The ratio at 6be0b49 (BENCH.md § The end-to-end SUM, both machines, one quiet sitting each with
  the canaries matched), after the byte-string serde form (4153c01) and the inlining (4ec1471):
  serialized exchange 2.01× NEON and 2.20× AVX2 at 1 worker, 1.85× and 2.07× at 8; typed exchange
  2.14× and 2.14× at 1 worker, 1.77× and 1.78× at 8. Against the a22fae7 figures in the body
  (2.8–4.2× serialized, 1.9–2.8× typed), each serialized row cut its excess over isize by 40–70%.
  At 8 workers the rows sit at 1.77–2.07×, at 1 worker at 2.01–2.20×. The wire form's share, § The
  wire form at 2a82388: Exact's three passes cost 14.7 ns an update on NEON and 33.3 on AVX2 against
  isize's 8.1 and 16.3; the rest of the gap is the 33-byte word moving through timely's buffers and
  the arrangement against 8 bytes. The question in the body stands for the owner: (a) accept the
  ratio as it now is, or (c) revisit the layout.
