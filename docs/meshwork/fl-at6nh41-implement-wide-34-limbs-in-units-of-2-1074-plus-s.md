---
id: fl-at6nh41
title: "Implement Wide — 34 limbs in units of 2^-1074 plus special counters, total From<f64>, group ops, Mul<i64>, projection, bitrep limb conformance"
category: crate/types
labels: [reproducible-floats, rf-critical-path]
docs: [docs/DESIGN.md#wide-sp-wide, docs/DESIGN.md#nan-and--sp-specials]
needs: [fl-1dw33g3]
seq: 30
verify: run cargo test package=float-abel wide_
status: open
created: 2026-10-11T15:02Z
covers:
  - ref: docs/DESIGN.md#sp-wide
    sha: f63aa61088306a5bdb2273e46ee37d3cfc962902e340d9058b0232c64b9065c6
  - ref: docs/DESIGN.md#sp-specials
    sha: addd4f3bbbdf9d2ff253af0022364ac7d0702feb43c1a014a14161878ec9de0f
handoff: |
  Wide is the next tier, and nearly everything it needs is already in the
  tree from Fixed.
  
  Reuse, not rewrite:
  - `src/limbs.rs`: two's-complement `add_assign` (returns signed
  overflow), `negate`, signed `cmp`,
  `mul_assign_u64` on a magnitude, `abs_into`, `set_shifted(dst, u128,
  shift)`, `top_bit`,
  `window`, `any_below`. All take slices, so `[u64; 34]` works as `[u64;
  LIMBS]` did.
  - `src/ieee.rs`: `decompose(f64) -> Option<Dyadic>`; `None` is NaN or
  ±∞, which is exactly the
  branch where Wide counts a special instead of refusing.
  - `src/round.rs`: `round(negative, limbs, exp, sticky, F64) -> (bits,
  inexact)` and
  `div_window(mag, exp, n)`. Wide's `to_f64` is `abs_into` then `round(..,
  -1074, false, F64)`
  on the finite part, after the projection rule decides it is finite at
  all.
  - `src/fixed.rs` is the pattern for the trait surface: `Default` by hand
  (no array `Default`
  past 32), hand `Debug`, `TryFrom`/`From`, `Ord` via `limbs::cmp`,
  `Mul<i64>`/`Mul<isize>`,
  `Sum` for values and refs, `debug_assert!` on headroom.
  
  Shape (DESIGN.md § Wide, § NaN and ±∞): `struct Wide { limbs: [u64;
  34], specials: [i64; 3] }`,
  296 bytes; the limbs are two's complement in units of 2^-1074, so a
  finite deposit is
  `set_shifted` at `shift = exp + 1074` (never negative) and `add_assign`.
  `From<f64>` is total:
  finite deposits, NaN bumps `n`, +∞ bumps `p`, −∞ bumps `q`;
  negation and `Mul<i64>` apply to
  the limbs and the counters alike, so retracting a NaN restores the
  finite sum exactly. Projection
  at read, in this order: `n != 0` → NaN; `p != 0 && q != 0` → NaN; `p
  > 0` → +∞, `p < 0` → −∞;
  `q > 0` → −∞, `q < 0` → +∞; else `round` the finite part.
  `is_zero` is limbs and counters all
  zero; `is_finite` is counters all zero; `specials()` reads `(n, p, q)`.
  `Ord` is numeric on the
  finite part then `(n, p, q)` lexicographic; `Eq`/`Hash` derive, because
  the representation is
  canonical by construction as Fixed's is. Decide and document what
  `deposit_product` does with a
  non-finite factor: the design's § NaN algebra with a scalar count is
  the consistent choice
  (NaN factor → `n += 1`; ∞ × ∞ by the sign rule; ∞ × nonzero
  finite flips by sign; ∞ × 0 is an
  exact zero, by bilinearity, not NaN).
  
  bitrep is on crates.io at 0.5.1 (`cargo info bitrep` names the
  repository). Read its
  `FORMAT.md` "Adding a finite f64" BEFORE laying out the limbs: the task
  says Wide shares bitrep's
  limb layout, and sign representation, limb order and the exact unit must
  match for its
  `conformance/vectors.json` finite-only vectors to reproduce the 272 limb
  bytes. Fetch the vector
  file into `tests/data/` with its licence noted, and read it from
  `tests/wide.rs` with no new
  dev-dependency if the format allows (a hand-rolled line reader beats
  adding serde_json; if it is
  nested JSON, serde_json as a dev-dependency is fine and deny.toml's
  licence list grows with it).
  
  Tests `wide_*` in `tests/wide.rs` (a `mod wide;` line in
  `tests/main.rs`, or
  scripts/check-test-topics.sh fails): the group laws with specials in the
  multiset, size 296,
  every finite bitrep vector, `1e100 + (-1e100) + 1e-300` reads `1e-300`,
  NaN inserted then
  retracted leaves `is_zero()`. Reuse the hardware oracles from
  `tests/fixed.rs`: IEEE `a + b`,
  `a * b`, `a.mul_add(b, c)` and `a / n` for `n < 2^53` are correctly
  rounded two-term results, and
  on Wide there is no grid or ceiling to dodge, so random doubles over the
  whole finite range are
  fair game, subnormals included. Consider lifting `Rng` and the oracle
  helpers into a
  `tests/support.rs` (`mod support;`) rather than copying them. One trap
  already hit twice:
  `2f64.powi(e)` for `e <= -1024` reciprocates an infinite power and
  returns 0; use
  `f64::from_bits` or scale in two halves.
  
  Proven so far: Fixed's whole surface against the hardware oracles and an
  i128 oracle, the
  rounding routine on ties, subnormals, overflow and sticky, `div_window`
  against IEEE division.
  Gates: `./scripts/smoke.sh` and `./scripts/regression.sh` both green at
  the Fixed commit; run them
  with `CARGO_TARGET_DIR` outside ~/Documents (this session used
  `~/.cache/cargo-target/float-abel`).
  Standing finding to carry: the dbsp task fl-06sggdc has a comment with a
  toolchain probe; dbsp
  0.356.0 and 0.364.0 ICE on rustc 1.97/1.98 but build on 1.96.1 and
  1.99.0, so the design's "only
  1.96.1" is stale and the pin is the owner's call.
---
`src/wide.rs` per DESIGN.md § Wide and § NaN and ±∞. The deposit follows bitrep's `FORMAT.md`
section "Adding a finite f64", with the same limb positions. The counters `(n, p, q)` are signed, and
projection happens at read time.

Tests `wide_*`:
- the group laws;
- `size_of::<Wide>() == 296`;
- every finite-only vector in bitrep's `conformance/vectors.json` (fetched into `tests/data/`, its
  licence noted) reproduces the 272 limb bytes and the `f64` bits;
- `1e100 + (−1e100) + 1e-300` reads `1e-300`;
- a NaN inserted then retracted leaves `is_zero()`.

## log
- 2026-10-11T15:02Z created
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-wide @f63aa6108830
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-specials @addd4f3bbbdf
- 2026-10-11T15:59Z handoff by claude (85c2609c-6abc-4455-b7f3-6c1c79eda92c)
