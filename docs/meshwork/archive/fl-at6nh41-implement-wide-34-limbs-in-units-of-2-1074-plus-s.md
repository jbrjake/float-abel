---
id: fl-at6nh41
title: "Implement Wide — 34 limbs in units of 2^-1074 plus special counters, total From<f64>, group ops, Mul<i64>, projection, bitrep limb conformance"
category: crate/types
labels: [reproducible-floats, rf-critical-path]
docs: [docs/DESIGN.md#wide-sp-wide, docs/DESIGN.md#nan-and--sp-specials]
needs: [fl-1dw33g3]
seq: 30
verify: run cargo test package=float-abel wide_
status: done
created: 2026-10-11T15:02Z
covers:
  - ref: docs/DESIGN.md#sp-wide
    sha: f63aa61088306a5bdb2273e46ee37d3cfc962902e340d9058b0232c64b9065c6
  - ref: docs/DESIGN.md#sp-specials
    sha: addd4f3bbbdf9d2ff253af0022364ac7d0702feb43c1a014a14161878ec9de0f
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
- 2026-10-11T15:59Z handoff by claude (85c2609c-6abc-4455-b7f3-6c1c79eda92c)
- 2026-10-11T16:13Z open→doing — claimed by claude (ffdbc5f1-8414-4a1d-8060-cf0fb191411f)
- 2026-10-11T16:31Z doing→done — verify exit 0 @ 1a182d4+1

## comments
- 2026-10-11T16:31Z [claude (ffdbc5f1-8414-4a1d-8060-cf0fb191411f)] Built at 1a182d4: `src/wide.rs`, with `ieee::special` (a non-finite double classified by its
  bits) and `round::div_f64` (the division read, now shared with `Fixed`). Tests in `tests/wide.rs`
  and `tests/wide_group.rs`; `Rng` and the hasher lifted into `tests/support.rs`.
  
  Decisions taken here, each documented on the item:
  
  1. `deposit_product` returns `Result<(), OutOfWindow>` on `Wide` too. A product of two doubles
     spans 2^-2148 … 2^2048 and cannot be exact in 2176 bits, so a product below the unit is
     `OffGrid` and one at or past 2^1037 (Fixed's rule: 64·34 − 65 bits over the unit) is
     `AboveCeiling`, the state untouched. `NotFinite` is never returned. DESIGN.md § Operations and
     conversions says "refused on `Fixed` when it is off-grid", which reads as if `Wide` accepts every
     product; that sentence needs the owner's ruling (refuse by name as built, round onto the grid, or
     panic). Hardware has overflowed to ±∞ or underflowed to 0 long before either bound.
  2. Special factors follow § NaN and ±∞ with unit counts: NaN → n += 1; ∞·∞ by the sign rule;
     ∞ × nonzero finite → ±1 on that infinity's counter by the finite factor's sign, so (+∞)·(−2)
     is p = −1, which reads −∞ but is a different state from q = 1, exactly as the algebra says
     (−P ≠ Q); a zero factor deposits nothing, so 0·∞ and 0·NaN are the exact zero.
  3. `try_to_f64` returns `Some(special)` for a special projection: nothing was rounded.
  4. `is_finite()` means the counters are zero; a finite part past f64::MAX still reads ∞.
  5. Headroom is 2^77 deposits of f64::MAX (bitrep documents 2^63); limb overflow and i64 counter
     overflow are `debug_assert!`s, wrapping in release, as on `Fixed`.
  6. `Debug` prints the value, the three counts, and the limbs up to the last one that differs from
     the sign fill.
  
  Oracles: every case of bitrep 0.5.1's `conformance/vectors.json` (272 limb bytes, flag byte, value
  bits; forward, reversed, two merged shards), bitrep itself as a pinned dev-dependency on 300 random
  multisets (its default feature set has zero dependencies; deny.toml's licence list did not grow),
  and IEEE `a + b`, `a * b`, `mul_add`, `a / n` over the whole finite range, subnormals included.
  bitrep's FORMAT.md says the state is 289 bytes; `SumF64::BYTES` and every vector say 281, noted in
  `tests/data/bitrep/README.md`.
  
  Verified this session: `cargo test` 49 tests green, `cargo clippy --all-targets -- -D warnings`
  and the two no_std lib configurations clean, `RUSTDOCFLAGS=-D warnings cargo doc` clean,
  `scripts/smoke.sh` and `scripts/regression.sh` green at 1a182d4 (MSRV 1.74.0 build and cargo-deny
  included). Mutation checks: dropping the deposit sign failed 8 of 16 tests; swapping the two
  infinities in the projection failed 6.
  
  Not built, by scope: the tier conversions (`Wide: From<Fixed>`, `Exact: From<Wide>`,
  `Exact: From<Fixed>`, `Wide::try_from(&Exact)`), filed as their own task; the bulk entry points
  (simd task); `to_f32` (rounding task); the wire encoding (wire task).
