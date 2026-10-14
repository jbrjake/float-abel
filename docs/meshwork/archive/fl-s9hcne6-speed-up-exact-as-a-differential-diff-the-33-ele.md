---
id: fl-s9hcne6
title: "Speed up Exact as a differential diff — the 33-element serde tuple first, then the word, re-measured by the end-to-end SUM on both machines"
status: done
category: crate/differential
labels: [reproducible-floats]
relates: [fl-bbevtyv, fl-cbtfmqz]
verify: "all(lacks BENCH.md /commit fdf0575/, run cargo test package=float-abel wire_)"
docs:
  - BENCH.md#the-end-to-end-sum
  - docs/DESIGN.md#wire-format-sp-wire
seq: 140
created: 2026-10-13T16:26Z
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
- 2026-10-14T16:42Z doing→done — verify exit 0 @ 6be0b49+4

## comments
- 2026-10-14T13:59Z [claude (16e02ca3-15ad-4c80-bfb4-53e89fee757d)] Progress, 2026-10-14. Lead 1 measured: benches/wire.rs (a22fae7), BENCH.md § The wire form; the cost is per serializer call on the write side. Lead 2 ruled by the owner in session and built under fl-11k5gz6 at 4153c01: Exact and Wide serialise as one byte string. Lead 3 built at 4ec1471: #[inline] on the operator chain down to the compact tier and ieee::decompose. Lead 4 pending: the a22fae7 control and the 2a82388 variant run in one quiet window on each machine (neon-quiet.sh and avx2-quiet.sh in the session scratchpad); the first attempts were contested by the owner's browser on NEON and another project's tpch test binary on AVX2, so their rows are not recorded.
- 2026-10-14T14:42Z [claude (16e02ca3-15ad-4c80-bfb4-53e89fee757d)] NEON A/B landed, one quiet sitting 10:23-10:41 (start load 2.8, canaries f64 += 0.581 vs 0.581 ns, Wide add 15.6 vs 15.8, to_f64 9.78 vs 9.72, isize SUM rows within 4%): control a22fae7 → variant 2a82388. ops same-scale: deposit 5.80 → 4.94 ns, add both built 8.54 → 3.74, += 7.20 → 3.81, deposit_product 11.76 → 11.45; the 5 ns compact-add target is met on NEON. dd_sum, ms per run of 10^7: serialized 1w Exact 859.1 → 432.5 (isize 208.2 → 214.9; 4.13× → 2.01×), serialized 8w 141.4 → 79.7 (43.0 → 43.0; 3.29× → 1.85×), typed 1w 348.9 → 265.1 (123.1 → 123.7; 2.83× → 2.14×), typed 8w 61.1 → 50.0 (30.5 → 28.2; 2.00× → 1.77×). Per update at 1w: serialized gap 65 → 22 ns, typed gap 23 → 14, the wire's share 42 → 8. wire at 2a82388 (isize canary 5.7/2.4 as on the idle host): isize 8.1, the byte string 14.7, fields 32.2, the tuple newtype 85.2 ns a crossing. AVX2 still waiting on a window free of the other project's tpch binary (avx2-guarded.sh on the box).
- 2026-10-14T16:42Z [claude (16e02ca3-15ad-4c80-bfb4-53e89fee757d)] Simd sitting at 2a82388 on both machines, BENCH.md commit 6be0b49. AVX2: one guarded attempt,
  clean (load 0.00 at the start, no overlap with the other project's test binary, every row within
  1.3% of the previous AVX2 run by criterion's own change lines). NEON: the f64 vectorised floor at
  0.080 against 0.084, Fixed256 and Binned bulk within 0.5%. The one kernel that moved is Wide's
  NEON bulk entry, from 2.09 to 1.82 ns with src/simd untouched. An A/B/A in one quiet window (load
  1.9) against a simd bench built from a22fae7 in its own target dir gave 2.11 / 1.82 / 2.11 ns with
  the one-at-a-time loop (18.66 / 18.73 / 18.64) and the floor (0.082 / 0.080 / 0.081) flat.
  objdump explains it: at a22fae7, wide_extend calls neon::fold_tile::<4, false> out of line (191
  instructions, 22 stp and 17 ldp, the refusal closure behind an indirect blr); at 2a82388,
  fold_tile and the closure are inlined into wide_extend (782 instructions, no call but
  limbs::negate and panic_fmt), because #[inline] on ieee::decompose (4ec1471) made Wide::deposit,
  the closure's body, cheap enough to inline. The AVX2 build inlines the fold either way: 2.52 ns,
  0.11% under the previous run. Lead status: (1) the wire bench, a22fae7 and 2a82388; (2) the
  byte-string serde form built at 4153c01 under the owner's "running bincode 3 times is crazy"
  message, the Fixed tuple form kept; (3) #[inline] at 4ec1471, 51 marks; (4) the SUM re-taken on
  both machines, BENCH.md § The end-to-end SUM: serialized 8 workers 1.85× NEON and 2.07× AVX2, 1
  worker 2.01× and 2.20×, typed 1.77–2.14×. The push gate (scripts/regression.sh, sandbox off for
  cargo-deny) is green at 6be0b49: regression: OK, exit 0, every leg, on a warm target.
