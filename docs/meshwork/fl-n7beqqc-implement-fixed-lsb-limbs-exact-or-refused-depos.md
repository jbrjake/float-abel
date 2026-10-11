---
id: fl-n7beqqc
title: "Implement Fixed<LSB, LIMBS> — exact-or-refused deposit, quantize, group ops, canonical zero, Ord and Hash, to_f64 and div_f64"
category: crate/types
labels: [reproducible-floats, rf-critical-path]
docs: [docs/DESIGN.md#fixed-sp-fixed]
needs: [fl-1dw33g3]
seq: 20
verify: run cargo test package=float-abel fixed_
status: open
created: 2026-10-11T15:02Z
covers:
  - ref: docs/DESIGN.md#sp-fixed
    sha: 8fa2eadaab9aab115d1312b30e795f024b3b0fcdf571a7817f12502a4690e1ae
---
`src/fixed.rs` per DESIGN.md § Fixed: two's-complement `[u64; LIMBS]` in units of 2^LSB.
`try_deposit` accepts a finite double iff it is on the grid and inside the window, else
`Err(OutOfWindow)`. `quantize` rounds to nearest-even on the grid. `Fixed256 = Fixed<-96, 4>`.

Tests `fixed_*`:
- the group laws, hand-rolled until the laws kit lands;
- `Fixed256` accepts 2⁻⁴⁴ and refuses 2⁻⁴⁵ + ulp, and refuses 2⁹⁵;
- `quantize(−x) == −quantize(x)`;
- a `const` assertion `size_of::<Fixed256>() == 32`.

## log
- 2026-10-11T15:02Z created
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-fixed @8fa2eadaab9a
