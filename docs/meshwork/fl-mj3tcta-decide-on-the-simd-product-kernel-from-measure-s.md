---
id: fl-mj3tcta
title: "Decide on the SIMD product kernel from Measure's deposit_products ratio — keep the scalar path and say so in DESIGN.md § Speed, or build it behind simd"
status: open
category: rulings
labels: [reproducible-floats]
needs: [fl-e4jjqf1]
discovered-from: fl-cb4pxq1
docs:
  - docs/DESIGN.md#speed-sp-simd
seq: 156
created: 2026-10-13T15:14Z
verify: contains docs/meshwork/fl-mj3tcta-decide-on-the-simd-product-kernel-from-measure-s.md /2026-[0-9][0-9]-[0-9][0-9] owner ruled/
---

Owner's call, deferred from "Decide whether deposit_products gets a SIMD product kernel" (fl-cb4pxq1), where the owner chose "Hold for Measure's row": the decision is made on the measured ratio once Measure (fl-e4jjqf1) has a `deposit_products` bench row per tier, scalar check-then-deposit as it stands today. This task waits on Measure and carries the question forward.

What to bring the owner: the row's numbers (bulk `deposit_products` per tier against the same tier's bulk `deposit` kernel, same machine, same corpus, from BENCH.md) and the two options that remain:
(a) Keep the scalar product path and say so in DESIGN.md § Speed, one sentence: products are the dot-product shape, exact and bit-identical, and not vectorised.
(b) Build the kernel behind `simd`, scalar reference first as the sums were, with its own `simd_` tests and a bench row, on every tier: per lane the 106-bit mantissa product as four 32×32 multiplies, three limbs placed per value, the same soundness test and the same scalar fallback.

The owner writes the line this task's verify looks for, dated.

## log
- 2026-10-13T15:14Z created

## comments
- 2026-10-13T16:09Z [claude (10cb7eaf-2ccb-44b5-b4e4-5396d637aa48)] Measure's row is in: BENCH.md § The product-kernel row, commit 7792a3f, 2²⁰ same-scale factor pairs with 26-bit mantissas (on Fixed256's grid, under every ceiling). Per pair, bulk `deposit_products` as it stands (scalar check-then-deposit) / the one-at-a-time loop / the same tier's bulk `deposit` kernel, and the ratio of the first to the last. NEON: Wide 25.99 / 24.22 / 2.09 ns, 12.4×; Fixed256 10.70 / 9.24 / 1.60 ns, 6.7×; Binned<3> 14.88 / 12.89 / 1.97 ns, 7.5×. AVX2: Wide 56.08 / 50.17 / 2.44 ns, 23.0×; Fixed256 26.01 / 20.27 / 2.00 ns, 13.0×; Binned<3> 31.94 / 24.96 / 1.67 ns, 19.1×. The scalar product path sits 7–23× over the bulk deposit kernel, the band the sums' loops sat in before their kernels (5–14×); the check pass costs 1.5–2 ns a pair on NEON and 6–7 ns on AVX2. Against plain f64 (0.54 / 0.74 ns per add), the scalar products are 20–48× on NEON and 35–76× on AVX2. The ruling is the owner's; the numbers are its input.
