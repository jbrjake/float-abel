---
id: fl-ng7bs72
title: "Rule on the SIMD kill condition now that the scalar reference is 4× faster: Wide's NEON kernel is 2.7× its one-at-a-time loop, under the 3× line DESIGN.md § Targets and kill conditions draws"
status: open
category: rulings
discovered-from: fl-sfp5tnw
verify: contains docs/meshwork/fl-ng7bs72-rule-on-the-simd-kill-condition-now-that-the-sca.md /2026-[0-9][0-9]-[0-9][0-9] owner ruled/
docs:
  - docs/DESIGN.md#targets-and-kill-conditions-sp-kills
created: 2026-11-23T16:21Z
---

DESIGN.md § Targets and kill conditions, the row "SIMD earns its feature": "bulk deposit, each
kernel vs the scalar reference, same-scale fixture, NEON and AVX2", kill "< 3× over scalar → 0.1.0
ships scalar-only, and the kernels stay behind the feature as experimental".

The scalar reference moved at 64fb3cd (fl-sfp5tnw, answering sazed#sa-jxnkncd): the one-value
deposit adds the shifted mantissa in the limbs it lands on, in place of three full-width passes.
The kernels did not move. BENCH.md § SIMD earns its feature, at 64fb3cd:

| tier | NEON one at a time | NEON bulk | gain | AVX2 one at a time | AVX2 bulk | gain |
|---|--:|--:|--:|--:|--:|--:|
| Wide | 5.08 ns | 1.90 ns | 2.7× | 8.76 ns | 2.53 ns | 3.5× |
| Fixed256 | 5.15 ns | 1.65 ns | 3.1× | 9.62 ns | 2.02 ns | 4.8× |
| Binned<3> | 8.65 ns | 2.09 ns | 4.1× | 14.03 ns | 1.67 ns | 8.4× |

Before, the gains were 10.3×, 4.9×, 5.1× (NEON) and 14.4×, 8.0×, 11.0× (AVX2). `Wide` on NEON is
now under the 3× line and `Fixed256` on it, on a shared host (median of four takes).

The question for the owner: does the kill condition stand as written against the faster scalar
reference (then `Wide`'s NEON kernel reads as "experimental"), or is it restated, for example
against the absolute per-value targets the same section already carries (1.5 ns `Wide`, 1.0 ns
`Fixed` NEON), or dropped for a crate past 0.1.0? Nothing is built until the owner answers; the
verify is the design row naming the ruling.

## log
- 2026-11-23T16:21Z created
