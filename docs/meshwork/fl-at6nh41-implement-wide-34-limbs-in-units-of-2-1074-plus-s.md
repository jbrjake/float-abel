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
