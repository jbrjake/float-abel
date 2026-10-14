---
id: fl-mj3tcta
title: "Decide on the SIMD product kernel from Measure's deposit_products ratio — keep the scalar path and say so in DESIGN.md § Speed, or build it behind simd"
status: blocked
category: rulings
labels: [reproducible-floats]
needs: [fl-e4jjqf1]
discovered-from: fl-cb4pxq1
docs:
  - docs/DESIGN.md#speed-sp-simd
seq: 156
created: 2026-10-13T15:14Z
verify: contains docs/meshwork/fl-mj3tcta-decide-on-the-simd-product-kernel-from-measure-s.md /2026-[0-9][0-9]-[0-9][0-9] owner ruled/
blocked-reason: "awaiting a consumer with the bulk dot-product shape; the owner's hold, 2026-10-14"
---

Owner's call, deferred from "Decide whether deposit_products gets a SIMD product kernel" (fl-cb4pxq1), where the owner chose "Hold for Measure's row": the decision is made on the measured ratio once Measure (fl-e4jjqf1) has a `deposit_products` bench row per tier, scalar check-then-deposit as it stands today. This task waits on Measure and carries the question forward.

What to bring the owner: the row's numbers (bulk `deposit_products` per tier against the same tier's bulk `deposit` kernel, same machine, same corpus, from BENCH.md) and the two options that remain:
(a) Keep the scalar product path and say so in DESIGN.md § Speed, one sentence: products are the dot-product shape, exact and bit-identical, and not vectorised.
(b) Build the kernel behind `simd`, scalar reference first as the sums were, with its own `simd_` tests and a bench row, on every tier: per lane the 106-bit mantissa product as four 32×32 multiplies, three limbs placed per value, the same soundness test and the same scalar fallback.

The owner writes the line this task's verify looks for, dated.

## log
- 2026-10-13T15:14Z created
- 2026-10-14T21:21Z open→done — verify exit 0 @ e628ed5+6
- 2026-10-14T21:37Z done→open
- 2026-10-14T21:45Z open→blocked — awaiting a consumer with the bulk dot-product shape; the owner's hold, 2026-10-14

## comments
- 2026-10-13T16:09Z [claude (10cb7eaf-2ccb-44b5-b4e4-5396d637aa48)] Measure's row is in: BENCH.md § The product-kernel row, commit 7792a3f, 2²⁰ same-scale factor pairs with 26-bit mantissas (on Fixed256's grid, under every ceiling). Per pair, bulk `deposit_products` as it stands (scalar check-then-deposit) / the one-at-a-time loop / the same tier's bulk `deposit` kernel, and the ratio of the first to the last. NEON: Wide 25.99 / 24.22 / 2.09 ns, 12.4×; Fixed256 10.70 / 9.24 / 1.60 ns, 6.7×; Binned<3> 14.88 / 12.89 / 1.97 ns, 7.5×. AVX2: Wide 56.08 / 50.17 / 2.44 ns, 23.0×; Fixed256 26.01 / 20.27 / 2.00 ns, 13.0×; Binned<3> 31.94 / 24.96 / 1.67 ns, 19.1×. The scalar product path sits 7–23× over the bulk deposit kernel, the band the sums' loops sat in before their kernels (5–14×); the check pass costs 1.5–2 ns a pair on NEON and 6–7 ns on AVX2. Against plain f64 (0.54 / 0.74 ns per add), the scalar products are 20–48× on NEON and 35–76× on AVX2. The ruling is the owner's; the numbers are its input.
- 2026-10-14T16:42Z [claude (16e02ca3-15ad-4c80-bfb4-53e89fee757d)] The row re-taken at 2a82388 after the inlining (BENCH.md § The product-kernel row, commit
  6be0b49), deposit_products / loop / bulk deposit: NEON Wide 25.84 / 24.46 / 1.80 ns (14.4×),
  Fixed256 10.70 / 9.22 / 1.60 (6.7×), Binned<3> 15.13 / 13.21 / 2.00 (7.6×); AVX2 Wide 56.08 /
  50.13 / 2.49 (22.5×), Fixed256 26.08 / 20.37 / 2.01 (13.0×), Binned<3> 31.69 / 24.93 / 1.69
  (18.8×). The band is 7–23× as before; the check pass is 1.4–1.9 ns a pair on NEON and 6–7 ns on
  AVX2. The product figures moved within 3% on NEON and 0.5% on AVX2; the inlining reached only
  Wide's bulk deposit column (2.09 to 1.80 on NEON, the fold inlined into wide_extend), which
  raises Wide's ratio from 12.4× to 14.4×. The input stands as written.
- 2026-10-14T21:21Z [claude (4e0e29d7-b85d-4a99-93e3-a9a5c00bb168)] 2026-10-14 the session took as ruled, withdrawn in the next comment: owner, in session: "let's move on from benchmarking. i am happy with the results from last session, showing that the incremental version wins out well below 1M records." Read as option (a): the scalar product path stays, and DESIGN.md § Speed says so in one sentence: products are the dot-product shape, exact and bit-identical, and not vectorised, at 7–23× the same tier's bulk deposit kernel (BENCH.md § The product-kernel row, 6be0b49). No kernel is built behind `simd`. The reading from the owner's line to option (a) is the session's, read back in the closing report; a correction reopens this.
- 2026-10-14T21:37Z [claude (4e0e29d7-b85d-4a99-93e3-a9a5c00bb168)] 2026-10-14 owner, in session, on the reading recorded above: "i honestly don't know on product kernel, did you actually benchmark to see if there's a scale where it makes sense to make it faster?" The reading to option (a) is withdrawn and the task reopened; DESIGN.md § Speed says the ruling is open. Answer handed to the owner in the session's report: no kernel exists, so no kernel was benchmarked; what BENCH.md § The product-kernel row measures is the scalar product path against the same tier's vectorised sum kernel, 7–23× per pair, which bounds what a kernel could win and matches the band the sums' loops sat in before theirs (5–14×, won back as 5–14×); the cost is linear per pair, so no data size changes the ratio, and no consumer workload with the bulk dot-product shape has been measured. Not ruled.
- 2026-10-14T21:45Z [claude (4e0e29d7-b85d-4a99-93e3-a9a5c00bb168)] 2026-10-14 owner, in session: "leave the product kernel open awaiting a consumer". Not ruled either way: the scalar path stays and no kernel is built until a consumer brings the bulk dot-product shape (a column engine's batch dot product or Σx², not an engine's one-row-at-a-time join). Blocked on that consumer; DESIGN.md § Speed says so.
