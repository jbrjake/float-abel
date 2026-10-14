---
id: fl-s9hcne6
title: "Speed up Exact as a differential diff — the 33-element serde tuple first, then the word, re-measured by the end-to-end SUM on both machines"
status: doing
category: crate/differential
labels: [reproducible-floats]
relates: [fl-bbevtyv, fl-cbtfmqz]
verify: "all(lacks BENCH.md /commit fdf0575/, run cargo test package=float-abel wire_)"
docs:
  - BENCH.md#the-end-to-end-sum
  - docs/DESIGN.md#wire-format-sp-wire
seq: 140
created: 2026-10-13T16:26Z
handoff: |
  The tree at b446dbd is clean and unpushed: the three bench targets,
  BENCH.md with every number
  DESIGN.md's kill conditions read, and the store. The commit gate passed
  on each of this session's
  commits; the push gate was not re-run after them, and clippy was green
  on the bench code before
  7792a3f, so run `./scripts/regression.sh` (sandbox off for cargo-deny)
  before closing anything.
  
  Start here, not with the docs task: the owner asked for this task in
  session on 2026-10-13 and it
  leads the queue. The body is the brief. The files are
  src/wire/serde_impls.rs (`Serialize for
  Exact`, `ExactVisitor::visit_seq`), tests/wire.rs (the postcard framing
  assertion that pins the
  tuple form), benches/dd_sum.rs (the measurement, `cargo bench --bench
  dd_sum`, eight rows in about
  six minutes on this machine; the rows are read with `grep -A1
  "^dd_sum/"` because the names wrap),
  and BENCH.md § The end-to-end SUM, which the verify wants re-taken at a
  newer commit than fdf0575.
  What is proven: the gap's split between wire form and word (BENCH.md's
  numbers, both machines);
  the oracle check in the bench, so a changed serializer that lands wrong
  bits fails before timing.
  What is not: whether bincode's cost is the 33 per-element calls or
  something else, which lead 1
  in the body measures first with a micro-bench. A length-prefixed form
  changes DESIGN.md § Wire
  format, a spec clause; raise it as a conflict with the micro-bench
  numbers and build it only on a
  ruling. Any number that moves goes into BENCH.md with the machine and
  the commit, both machines,
  never blended; the AVX2 box is jons-amd-mini by the rsync-and-ssh recipe
  in the session memory.
  The owner reads BENCH.md for the cost against plain floats first, so
  keep "What reproducibility
  costs" current when the end-to-end row moves.
claimed-by: claude (16e02ca3-15ad-4c80-bfb4-53e89fee757d)
---

Owner's instruction, 2026-10-13, in session: "add a task for the next session to speed up the
differential forms, like the serialization stuff you think is a likely culprit". The target is
`Exact` as a differential-dataflow diff, measured by `cargo bench --bench dd_sum` (BENCH.md § The
end-to-end SUM, commit fdf0575): on serialized exchange `Exact` runs 4.19× `isize` at one worker
and 3.51× at eight on NEON, 3.87× and 2.78× on AVX2; on typed exchange 2.78× and 2.12×, 2.61× and
1.92×. Per update on NEON at one worker, `Exact` costs about 65 ns over `isize` serialized and
21 ns typed, so the wire form is about 45 ns of the gap and the word about 21 ns. The ruling on
what ratio is acceptable (fl-bbevtyv) stays the owner's; this task moves the number.

Where the wire cost is: src/wire/serde_impls.rs. `Serialize for Exact` writes `to_bytes()` as a
33-element tuple, one `serialize_element::<u8>` call per byte, and `Deserialize` pulls 33
`next_element::<u8>()` calls through `ExactVisitor::visit_seq` before `from_bytes` validates.
timely's `ProcessBinary` exchange is bincode 1, so that is 33 single-byte writes and 33 reads
through serde's generic path per exchanged update, each way, at `count_total`'s
`arrange_by_self`, which is where 10⁷ `Exact` diffs cross the wire. The framing is a spec clause:
DESIGN.md § Wire format and the module doc say a value serializes as a fixed-length tuple so a
format emits `to_bytes` byte for byte with no length prefix, and tests/wire.rs asserts postcard's
bytes equal `to_bytes`. A `serialize_bytes` form would add a length prefix on bincode and postcard
and change that clause, which is a conflict to raise with the owner, not an edit.

Leads, in the order to try, each measured on both machines (the rsync-and-ssh recipe is in the
session memory) with the serialized rows expected to approach the typed ones:
1. A faster tuple path that keeps the framing: a `Serialize` that hands the 33 bytes to
   `serialize_tuple` through `SerializeTuple::serialize_element` on `&[u8; 33]` chunks is not
   available in serde, so measure whether bincode's cost is the per-element calls or the
   `Vec<u8>` growth, with a micro-bench of `bincode::serialize(&exact)` against a 33-byte array,
   before changing the form.
2. If the per-element calls are the cost, propose the length-prefixed form (`serialize_bytes`,
   41 bytes on bincode 1, 34 on postcard) to the owner with the micro-bench numbers, as the
   change to DESIGN.md § Wire format it implies; build it only on a ruling.
3. The word's 21 ns: `#[inline]` on `checked_add`, `AddAssign`, `From<f64>` and `is_zero`
   (fl-cbtfmqz's lead), and whether `explode`'s `Exact::from(f64::from_bits(bits))` can carry the
   word instead of rebuilding it.
4. Re-take BENCH.md § The end-to-end SUM at the new commit; the verify checks the section moved
   off fdf0575 and that the wire tests still pass. The number itself is never gated.

## log
- 2026-10-13T16:26Z created
- 2026-10-13T16:26Z handoff by claude (10cb7eaf-2ccb-44b5-b4e4-5396d637aa48)
- 2026-10-14T13:02Z open→doing — claimed by claude (16e02ca3-15ad-4c80-bfb4-53e89fee757d)
