---
id: fl-cb4pxq1
title: "Decide whether deposit_products gets a SIMD product kernel — vectorise the 106-bit product on Fixed, Wide and Binned, or keep the scalar check-then-deposit and say so in DESIGN.md § Speed"
status: done
category: crate/simd
labels: [reproducible-floats]
relates: [fl-b41y9kt, fl-e4jjqf1]
docs:
  - docs/DESIGN.md#speed-sp-simd
seq: 155
created: 2026-10-12T21:36Z
verify: contains docs/meshwork/fl-cb4pxq1-decide-whether-deposit-products-gets-a-simd-prod.md /2026-[0-9][0-9]-[0-9][0-9] owner ruled/
---

Owner's call, surfaced from "Ship the simd feature" (fl-b41y9kt). Today `deposit_products` on `Wide`, `Fixed` and `Binned` runs the scalar check-then-deposit: every pair is checked against the tier's window first, then deposited one at a time, so the bulk entry is bit-identical by construction and refuses the first bad pair with the state untouched. tests/simd.rs's `simd_deposit_products_*` and the golden transcript cover it. DESIGN.md § Speed's kernel table has no product row, so vectorising it is not in the design and nobody but the owner adds a kernel.

What a product kernel would be: per lane, the 106-bit mantissa product as four 32×32 multiplies (or two 64×64 low/high on AVX-512 IFMA, absent here), then three limbs placed per value instead of two, with the same soundness test and the same scalar fallback; the sums' kernels gained 5–15× over their loops, and a product kernel would gain in the same band on the dot-product shape, minus the multiply.

Options:
(a) Keep the scalar product path and say so in DESIGN.md § Speed, one sentence: products are the dot-product shape, exact and bit-identical, and not vectorised. Nothing to build.
(b) Build the kernel behind `simd`, scalar reference first as the sums were, with its own `simd_` tests and a bench row, on every tier.
(c) Hold until "Measure" (fl-e4jjqf1) has a `deposit_products` row, then decide on the ratio.

Recommendation: (c), because no consumer's hot path is known to be a dot product, and the Measure task's row costs one bench function. Approving (b) or (a) here is the ruling; the owner writes the line this task's verify looks for, dated.

## log
- 2026-10-12T21:36Z created
- 2026-10-13T15:14Z open→done — verify exit 0 @ 656c340+8

## comments
- 2026-10-13T15:14Z [claude (ace5f87f-8122-4167-a02a-0a2238009e67)] 2026-10-13 owner ruled, in session: "Hold for Measure's row", which is option (c): the decision is made on the measured ratio once Measure (fl-e4jjqf1) has a deposit_products bench row. Carried forward as fl-mj3tcta, gated on Measure.
