---
id: fl-sfp5tnw
title: "Make Wide's and Fixed's one-value deposit touch only the limbs the term lands on: add or subtract the shifted mantissa in place, carry or borrow running until it stops, in place of three full-width passes"
status: done
category: crate/simd
answers: sazed#sa-jxnkncd
verify: "all(contains src/limbs.rs add_shifted, lacks src/wide.rs limbs::set_shifted, run cargo test target=main placed_add)"
docs:
  - docs/DESIGN.md#speed-sp-simd
seq: -30
created: 2026-11-23T15:36Z
---

Answers sazed#sa-jxnkncd. `Wide::deposit_finite` (and `Fixed::add_placed`, the same three lines)
builds a zeroed `[u64; LIMBS]` term, `limbs::set_shifted`s the mantissa into it, `limbs::negate`s
the whole width for a negative value and `limbs::add_assign`s the whole width: three passes over
34 limbs for a mantissa that spans at most three. sazed's profile (2026-11-22, release, 1 ms
samples) puts 122,848 of about 176,000 samples in `Wide::deposit` and 13,218 in `limbs::negate`.

Build: one helper in `src/limbs.rs` that adds or subtracts a `u128` magnitude shifted by `shift`
bits into the limbs it lands on, the carry or the borrow running only until it stops, and returns
the signed overflow the full-width add reported (the headroom assertion stays in every build).
`Wide::deposit_finite` and `Fixed::add_placed` call it. Same bits; no API change; the SIMD bulk
kernels are untouched.

Pinned by a test of the placed add against the three-pass form it replaces (carry and borrow
rippling the whole width, the sign limb, the overflow flag at both ends) and by the existing
suites. Measured before and after in one sitting on `benches/simd.rs`'s one-at-a-time rows, into
BENCH.md.

## log
- 2026-11-23T15:36Z created
- 2026-11-23T15:36Z open→doing — claimed by claude (088ad122-d06d-463d-8c3f-01d8f3ef0f7f)
- 2026-11-23T16:21Z doing→done — verify exit 0 @ 216cf11+3

## comments
- 2026-11-23T16:21Z [claude (088ad122-d06d-463d-8c3f-01d8f3ef0f7f)] Done at 64fb3cd (code, tests) and the docs commit after it (BENCH.md's three simd tables, DESIGN.md § Speed's product band).
  
  What changed: `limbs::add_shifted` adds the shifted magnitude, or subtracts it as its complement with a carry in, across the at most three limbs it spans, then ripples the carry or the borrow until it stops; `Wide::deposit_finite` and `Fixed::add_placed` call it. Same bits, no API change. The bulk `Wide` kernel reaches the one-value deposit through `simd::wide_deposit_outside` (cold, out of line): inlined, the NEON kernel lost 13% (2.06–2.14 ns against 1.81). `limbs::set_shifted`, now only `Binned`'s, is `#[inline]`: on AVX2 `Binned`'s loop had drifted 6–15% slower without it and is 1.2–1.3× faster than the control with it.
  
  Measured (benches/simd.rs, control 0451613 in the same sitting): AVX2 (Ryzen 7 5700U, idle) `Wide` deposit 36.59 → 8.76 ns, product 49.99 → 15.15 ns; `Fixed256` 16.12 → 9.62 and 20.10 → 13.03 ns; bulk kernels within 1%. NEON (M5 Max, host shared with another session's tests, medians of four takes) `Wide` 18.6–19.3 → 5.08 ns.
  
  Pinned by tests/placed_add.rs (8 tests: both tiers against the three-pass form written in the test, `Fixed<-2148, 68>` included; the named headroom panic at both ends through the deposit path) and `limbs::tests::add_shifted_*`. Both were watched failing on two deliberate breaks (a ripple cut to four limbs, a dropped negative-overflow flag). The integration tests are characterisation: they passed on the three-pass code before the change. Full `cargo test` green here (206 + 38 + 14) and `--test main` green on the AMD box (206).
  
  Not done: the push gate, miri, the x86 lints, `bench-baseline.json` (a full sitting of every bench, and the Mac was shared). sazed's own verify (marasi's e2 gate) is theirs to run.
  
  Found on the way, filed: fl-ng7bs72 (the SIMD kill condition: `Wide`'s NEON kernel is now 2.7× its scalar loop, under the 3× line).
