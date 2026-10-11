---
id: fl-b41y9kt
title: "Ship the simd feature — runtime-dispatched NEON and AVX2 (AVX-512 where present) bulk kernels for Fixed, Binned and Wide, a scalar reference kernel, and the golden hash over every path"
category: crate/simd
labels: [reproducible-floats, rf-tail]
docs: [docs/DESIGN.md#speed-sp-simd]
needs: [fl-n7beqqc, fl-at6nh41, fl-9kp61j0, fl-v7t6xhz]
seq: 120
verify: run cargo test package=float-abel simd_
status: open
created: 2026-10-11T15:02Z
covers:
  - ref: docs/DESIGN.md#sp-simd
    sha: 8ff7be5375be41fcd0546ebe39bb157963b16daf08be459475027b1bf597fd3d
---
Per DESIGN.md § Speed. `src/simd/{mod,scalar,neon,avx2,avx512}.rs`, with `unsafe` only here and a
`// SAFETY:` line per block. `extend_from_slice` and `deposit_products` on the three tiers;
lane-parallel partials merged at the tile's end. Both `Binned` kernels are built and benchmarked, and
the loser is deleted.

Tests `simd_*`:
- every kernel equals the scalar kernel bit for bit on the conformance and rounding vectors and on
  10⁴ seeded multisets, specials and subnormals included;
- lane-count independence;
- the golden hash extended to every kernel.

## log
- 2026-10-11T15:02Z created
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-simd @8ff7be5375be
