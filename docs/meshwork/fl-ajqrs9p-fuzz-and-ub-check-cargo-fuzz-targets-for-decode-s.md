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
---
Per DESIGN.md § Hardening. The oracle is a `num-bigint` rational sum, as a dev-dependency only. A
nightly job runs 10 minutes per target, and anything it finds is committed as a regression test.

## log
- 2026-10-11T15:02Z created
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-hardening @071b3fa1d56a
- 2026-10-11T18:13Z cover docs/DESIGN.md#sp-hardening @a3b209f09263
