---
id: fl-6ht6aa6
title: "Measure the row 'holds every f64 exactly': seven edge doubles read back bit for bit, alone and less their next double down"
category: crate/proof
parent: fl-9p8yw9v
needs: [fl-2r5tb8c]
seq: 250
docs:
  - docs/DESIGN.md#what-an-incremental-engine-asks-of-a-weight-sp-laws
  - docs/DESIGN.md#nan-and--sp-specials
verify: 'all(exists examples/short-table/OUTPUT.md, contains examples/short-table/OUTPUT.md /^\| holds every f64 exactly \| yes \| yes \| yes \| yes \| yes \| yes \| yes \| no \|$/)'
status: open
created: 2026-11-06T14:19Z
---
The weight carries the column's value, so a lossy carrier makes the `SUM` wrong.

Probe values: `0.1`, `1e-20`, `1e30`, `1e300`, `f64::MAX`, `f64::MIN_POSITIVE` (2⁻¹⁰²²) and
`5e-324` (2⁻¹⁰⁷⁴). Two checks each, both compared with `to_bits`:

- `+x` reads `x`.
- `+x −below(x)` reads `x − below(x)`, one ulp and itself an f64, where
  `below(x) = f64::from_bits(x.to_bits() − 1)`. This catches a carrier that rounds on entry and
  rounds back the same way on read, which the first check alone cannot see.

Zero stays out: the group's zero is +0.0 and the sign of zero is not carried (DESIGN.md § NaN and
±∞), so −0.0 reading +0.0 is the same number, not a loss. NaN and ±∞ are the specials row's.

Expected, and the run decides: fixed no, since `checked_from_num` turns `5e-324` and `1e-20` into
0 and refuses `1e30` and `f64::MAX`. Every other column yes; accurate reads all seven, `f64::MAX`
included, and every gap.

Mutant: `Exact` that rounds each input through `f32` on entry. Control: `f64` passes this row.

## log
- 2026-11-06T14:19Z created
