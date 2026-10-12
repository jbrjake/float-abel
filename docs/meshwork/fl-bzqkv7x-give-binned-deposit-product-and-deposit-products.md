---
id: fl-bzqkv7x
title: "Give Binned deposit_product and deposit_products — the 106-bit product sliced into the bins with the product's sign, refused by name past the ceiling"
status: open
category: crate/types
labels: [reproducible-floats, rf-tail]
relates: [fl-v7t6xhz, fl-b41y9kt]
verify: run cargo test package=float-abel binned_product_
docs:
  - docs/DESIGN.md#operations-and-conversions-sp-ops
seq: 115
created: 2026-10-12T15:10Z
---

DESIGN.md § Operations and conversions: "Every tier has the bulk entry points
`extend_from_slice(&[f64])` and `deposit_products(&[f64], &[f64])`; they are the SIMD surface."
`Binned` has `extend_from_slice` and no product entry. Add `deposit_product(a, b)` and
`deposit_products(&[f64], &[f64])` in src/binned.rs: the 106-bit mantissa product as a `u128`
(`ieee::factor` on both sides, as `Wide::deposit_product` does), sliced at Wide's limb boundaries
into up to three bins with the product's sign, specials by the same sign rule as Wide's, and a
zero factor depositing nothing.

Two semantics to read back before building, since neither is in the spec:

- A product with a set bit below 2^-1074 (`Wide` refuses it as `OutOfWindow::OffGrid`). For a tier
  whose window never reaches below limb 0, those bits are below the window like any other dropped
  bits, so truncating them silently is consistent with the type; refusing them is consistent with
  `Wide`. The bound `n · 2^(−64(K−1)) · max|x|` holds either way.
- A product at or past 2^1037 (`AboveCeiling` on `Wide`): the top bin would sit at limb 33 or
  beyond, and `into_wide` cannot place its carry, so this one must be refused by name, which makes
  the method fallible as `Wide`'s is.

Tests `binned_product_*`: the deposit equals `Wide::deposit_product` truncated below the window on
random factor pairs and on the adversaries (a product straddling the window's bottom, a subnormal
times a subnormal, factors whose product crosses three limbs, specials and zeros), bit-identical
under shuffles, and the refusal leaves the state untouched.

## log
- 2026-10-12T15:10Z created
