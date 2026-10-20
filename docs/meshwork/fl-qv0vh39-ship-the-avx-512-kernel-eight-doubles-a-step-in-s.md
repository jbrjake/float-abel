---
id: fl-qv0vh39
title: "Ship the AVX-512 kernel — eight doubles a step in src/simd/avx512.rs, dispatched above AVX2, landed only once a machine with AVX-512 runs the simd_ tests and the golden hash green"
status: open
category: crate/simd
labels: [reproducible-floats, rf-tail]
relates: [fl-b41y9kt]
verify: exists src/simd/avx512.rs
docs:
  - docs/DESIGN.md#speed-sp-simd
seq: 200
created: 2026-10-12T21:09Z
---

Per DESIGN.md § Speed: "AVX-512 where present". `src/simd/avx512.rs`, eight doubles a step, dispatched above AVX2 in `src/simd/mod.rs`'s `fold_tile`, `span` and `top_limb` behind `is_x86_feature_detected!("avx512f")`, the same algorithm as scalar.rs op for op. AVX-512 makes two of the AVX2 workarounds one instruction: `vpmaxuq`/`vpminuq` for the span pass and masked compares for the suspect test; the variable shifts keep AVX2's any-count-of-64-or-more-is-zero rule, so no byte chain.

Lands only once a machine with AVX-512 has run `cargo test -p float-abel simd_` and `cargo test -p float-abel-golden` green, because "done means you watched it work": this machine is aarch64, jons-amd-mini is a Ryzen 7 5700U and turing is an i7-5930K, and none has an avx512 flag. The x86 CI runner's CPU is not controllable.

MSRV: the AVX-512 intrinsics stabilised in Rust 1.89, above the crate's 1.74. The module needs either a version gate or a stated floor for the `simd` feature in Cargo.toml and CI; the default features keep 1.74 either way, because `simd` is not one.

## log
- 2026-10-12T21:09Z created
- 2026-10-16T19:34Z handoff by claude (03cbad3b-1e00-455d-9d44-3b0d9decee6e)
