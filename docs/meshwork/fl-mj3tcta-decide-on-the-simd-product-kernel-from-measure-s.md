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
