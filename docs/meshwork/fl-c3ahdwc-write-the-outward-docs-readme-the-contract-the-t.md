---
id: fl-c3ahdwc
title: "Write the outward docs — README (the contract, the trilemma, the tiers table, the not-a-group argument), rustdoc on every public item, SECURITY.md and CHANGELOG.md"
category: crate/docs
labels: [reproducible-floats, rf-tail]
docs: [docs/DESIGN.md#documentation-sp-docs, docs/DESIGN.md#why-the-adaptive-binned-accumulator-is-not-a-group-sp-not-a-group]
needs: [fl-6shtzpj]
seq: 160
verify: "all(exists README.md, contains README.md /not a group/, exists SECURITY.md, exists CHANGELOG.md, exists src/lib.rs, contains src/lib.rs missing_docs)"
status: open
created: 2026-10-11T15:02Z
covers:
  - ref: docs/DESIGN.md#sp-docs
    sha: 7e33ab21417fb13cc9dd4b1853f2e1acb70c1b497b75a9174f779ac844aabb40
  - ref: docs/DESIGN.md#sp-not-a-group
    sha: 373785c4859c6da80c731d8709b693004ccff63b692a79548c0e60e956976708
handoff: |
  The tree at 6be0b49 has the push gate green end to end
  (`./scripts/regression.sh`, sandbox off
  for cargo-deny, every leg, both deny configs ok) and BENCH.md re-taken
  at 2a82388 in every
  section, each naming its commit. The README this task writes cites §
  What reproducibility costs
  for the price of the guarantee: `Exact` one row at a time is 8.5× a
  hardware add on NEON and 22×
  on AVX2 (4.94 and 15.89 ns); the SIMD kernels sum a column at 1.6–2.5
  ns a value, 5–7× (AVX2) to
  20–25× (NEON) the vectorised float sum; inside differential-dataflow
  `Exact` is 1.8–2.2× the
  native `isize` diff (serialized 8 workers 1.85× NEON, 2.07× AVX2). The
  wire form is now a serde
  byte string: `Serialize for Exact` and `Wide` hand `to_bytes()` to
  `serialize_bytes` and the
  visitors take `visit_bytes` and `visit_seq` (src/wire/serde_impls.rs,
  4153c01), so bincode 1
  frames the 33 bytes with a u64 length (41 bytes) and postcard with a
  varint (34); `Fixed` keeps
  the tuple form because `1 + 8 * LIMBS` cannot be a const in the impl.
  FORMAT.md § serde and
  DESIGN.md § Wire format say so; the README's wire paragraph should
  match them, not the old
  33-element tuple. `Exact`'s operators, checked methods and the compact
  tier are `#[inline]`
  (4ec1471, 51 marks): a consumer's `+=` on the word compiles to the
  integer ops in place, which the
  README can promise; `to_f64` and `Wide`'s add stay out-of-line calls.
  
  Rulings for the owner sit in `ready` above this task, each with its
  final input in comments:
  fl-bbevtyv (the SUM ratio, now 1.77–2.07× at 8 workers and
  2.01–2.20× at 1; (a) accept or (c)
  revisit the layout), fl-cbtfmqz (the compact add meets its 5 ns target
  on NEON after the inlining
  and misses by 3.2× on AVX2; `to_f64` and `Wide` add miss on both; leads
  in the body), fl-mj3tcta
  (the product kernel; the row stands at 7–23×). fl-jqbhm8j files
  bench-baseline.json; the
  estimates to write it from are under `$CARGO_TARGET_DIR/criterion/` on
  this machine and
  `~/float-abel/target/criterion/` on jons-amd-mini, all at 2a82388.
  
  Files this task touches: README.md (new), SECURITY.md and CHANGELOG.md
  (new; the disclosure
  address is the owner's ruling, on the closed task in this task's
  `relates:` line), src/lib.rs (`#![warn(missing_docs)]` and the crate
  doc),
  and rustdoc on every public item in src/exact/mod.rs, src/wide.rs,
  src/fixed.rs, src/binned.rs,
  src/wire.rs, src/laws.rs and the two adapters. The verify wants
  README.md to contain "not a
  group", which DESIGN.md § Why the adaptive binned accumulator is not a
  group supplies (covered,
  @373785c4859c). Watch the file lengths: tests/wire.rs is 541 lines and
  five others sit just over
  500; rustdoc added to src/wide.rs (528) or src/binned.rs (506) should
  stay under the 750 ceiling or
  move a block out. Both bench hosts are shared: the AVX2 box with the
  owner's other project's
  `tpch_four_engines` test binary, this one with the owner's browser;
  nothing in this task benches.
relates: [fl-pmqp4rh]
---
Per DESIGN.md § Documentation. The not-a-group argument is the README's centrepiece, because nobody
else on crates.io makes it. `#![deny(missing_docs)]` in `src/lib.rs`.

## log
- 2026-10-11T15:02Z created
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-docs @7e33ab21417f
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-not-a-group @373785c4859c
- 2026-10-13T15:11Z handoff by claude (ace5f87f-8122-4167-a02a-0a2238009e67)
- 2026-10-13T15:11Z handoff by claude (ace5f87f-8122-4167-a02a-0a2238009e67)
- 2026-10-13T15:14Z handoff by claude (ace5f87f-8122-4167-a02a-0a2238009e67)
- 2026-10-13T15:15Z handoff by claude (ace5f87f-8122-4167-a02a-0a2238009e67)
- 2026-10-13T16:09Z handoff by claude (10cb7eaf-2ccb-44b5-b4e4-5396d637aa48)
- 2026-10-13T16:10Z handoff by claude (10cb7eaf-2ccb-44b5-b4e4-5396d637aa48)
- 2026-10-14T16:43Z handoff by claude (16e02ca3-15ad-4c80-bfb4-53e89fee757d)
- 2026-10-14T16:43Z handoff by claude (16e02ca3-15ad-4c80-bfb4-53e89fee757d)
