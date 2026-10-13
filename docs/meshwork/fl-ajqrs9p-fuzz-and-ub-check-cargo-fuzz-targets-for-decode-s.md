---
id: fl-ajqrs9p
title: "Fuzz and UB-check — cargo-fuzz targets for decode, sum-vs-oracle and sub-round-trip, and cargo miri test in CI"
category: crate/hardening
labels: [reproducible-floats, rf-tail]
docs: [docs/DESIGN.md#hardening-sp-hardening]
needs: [fl-a2wrx7w]
seq: 140
verify: "all(exists fuzz/fuzz_targets/decode_robustness.rs, exists fuzz/fuzz_targets/sum_vs_oracle.rs, exists fuzz/fuzz_targets/sub_roundtrip.rs)"
status: open
created: 2026-10-11T15:02Z
covers:
  - ref: docs/DESIGN.md#sp-hardening
    sha: a3b209f092634765b7ec3afcf91073d89ff4365d1a6e4dd5514ad25a89abde8f
handoff: |
  The tree is at 53b1b5d plus the store commit after it; the push gate is
  green end to end
  (`./scripts/regression.sh`, sandbox off for cargo-deny's advisory
  fetch), the dbsp feature is
  shipped and its deny posture ruled and built, and nothing of this task
  exists yet: no fuzz/
  directory, no miri job, no num-bigint anywhere. The spec is DESIGN.md §
  Hardening's second
  bullet, three `cargo-fuzz` targets named there plus `cargo miri test` in
  CI, and this task's
  verify is the three target files existing, so the shape is fixed and the
  work is the contents.
  
  What the tree gives the targets. Decode: `Exact::from_bytes`,
  `Wide::from_bytes` and
  `Fixed::<LSB, LIMBS>::from_bytes` (src/exact/wire.rs, src/wide.rs,
  src/fixed.rs, through
  src/wire.rs's `body`/`u64_le` helpers) each return a `WireError` and
  never panic; the fuzz
  target feeds arbitrary bytes and asserts no panic and, on `Ok`, that
  `to_bytes` reproduces the
  input byte for byte, which is the canonical-form claim. The `arbitrary`
  feature already gives
  `Arbitrary` impls for fuzz inputs (src/, `#[cfg(feature =
  "arbitrary")]`), so sum-vs-oracle
  can draw `Vec<f64>` and fold through `Wide`, `Fixed256` and `Exact`
  (skip a `Fixed` input the
  window refuses, by its `OutOfWindow`), comparing `to_f64` against a
  `num-bigint` rational sum
  rounded to nearest-even, which `tests/support.rs` and tests/rounding.rs
  already do in integer
  arithmetic for the deterministic suite; the oracle is a dev-dependency
  of the fuzz crate only,
  never of the lib. Sub-round-trip: `a + b - b == a` bit for bit on every
  tier, with the specials
  in. Every crash found is a regression test in tests/, not only a corpus
  file.
  
  Mechanics that bite. cargo-fuzz makes fuzz/ its own package: add
  `exclude = ["fuzz"]` to the
  root `[workspace]` or cargo will try to build it as a member on stable;
  keep
  `CARGO_TARGET_DIR` as the repo's gitignored settings set it (the fuzz
  crate needs nightly and
  `-Z sanitizer`, so it gets its own target dir under ~/.cache). miri:
  `cargo +nightly miri test
  --no-default-features --features serde,laws,arbitrary` or similar,
  because the simd feature's
  NEON/AVX2 intrinsics are not supported by miri and `std::arch` detection
  is where it stops;
  gate the trybuild topic (tests/ui.rs) and the engine topics
  (differential, dbsp) with
  `#[cfg_attr(miri, ignore)]`, since trybuild runs cargo and the engines
  spin threads miri runs
  too slowly; and cut the laws kit's case count under `cfg(miri)`. The
  nightly job runs 10
  minutes per target (`cargo fuzz run <target> -- -max_total_time=600`) on
  a schedule, its own
  workflow beside golden.yml, never in a git hook. The dbsp feature stays
  out of both: its
  toolchain is pinned and nothing in it decodes untrusted bytes except
  through `Exact::canonical`,
  which the decode target reaches through `from_bytes` already.
  
  Owner-scoped, open beside this: fl-yzy8bwc (dbsp's dependency line) and
  fl-cb4pxq1 (the SIMD
  product kernel), both rulings, neither blocking this.
relates: [fl-yzy8bwc, fl-cb4pxq1]
---
Per DESIGN.md § Hardening. The oracle is a `num-bigint` rational sum, as a dev-dependency only. A
nightly job runs 10 minutes per target, and anything it finds is committed as a regression test.

## log
- 2026-10-11T15:02Z created
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-hardening @071b3fa1d56a
- 2026-10-11T18:13Z cover docs/DESIGN.md#sp-hardening @a3b209f09263
- 2026-10-13T00:50Z handoff by claude (70076334-fceb-445b-b3a5-700374edbd88)
