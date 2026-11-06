---
id: fl-sw53v5t
title: "Measure the row 'correct on every input tested': eleven finite sums with f64 answers, through each crate's one-at-a-time path and its batch entry"
category: crate/proof
parent: fl-9p8yw9v
needs: [fl-2r5tb8c]
seq: 320
docs:
  - docs/DESIGN.md#what-an-incremental-engine-asks-of-a-weight-sp-laws
verify: 'all(exists examples/short-table/OUTPUT.md, contains examples/short-table/OUTPUT.md /^\| correct on every input tested \| yes \| yes \| no \| no \| no \| yes \| yes \| no \|$/)'
status: done
created: 2026-11-06T14:19Z
---
Every other row assumes the arithmetic underneath is right.

Each input runs through the one-at-a-time recipe and through the column's batch entry where it
has one; both paths must read the answer, compared with `==`. Finite inputs only: the specials
are their own row.

| input | answer | why it is there |
|---|---|---|
| `0.1, 0.2, −0.1, −0.2` | 0 | the README's opening example |
| `1e100, 1.0, −1e100` | 1 | insert, insert, delete |
| `1e200, 1e100, 1.0, −1e100, −1e200` | 1 | three scales defeat one correction term |
| ten `0.1`, then `−1.0` | 2⁻⁵⁴ | `0.1` is 3602879701896397 / 2⁵⁵, so ten of them less one is 2 / 2⁵⁵; a carrier holding decimal 0.1 reads 0 |
| `1e308, 1e308, −1e308` | 1e308 | a partial past `f64::MAX` |
| `f64::MAX, f64::MAX, −f64::MAX` | `f64::MAX` | the same at the top of the range |
| `1.0, −1.0` | 0 | a state cancelled to exactly zero |
| `5e-324, 5e-324` | 1e-323 | subnormals |
| `1e-20, 1.0, −1.0` | 1e-20 | below a fixed grid |
| `1e30, 2.0, −1e30` | 2 | above a fixed range |
| 65,536 × `1.0` | 65536 | a long stream of one magnitude |

Expected, and the run decides: float-abel, bitrep, num-rational and bigdecimal yes. xsum no:
`[f64]::xsum()` on 65,536 × `1.0` reads `-65536.00000000001`, because the slice entry picks its
large accumulator from 1,000 elements, while `XsumSmall` one at a time is right. fsum no:
`1e308 + 1e308 − 1e308` reads NaN. accurate no: the reads of `MAX + MAX − MAX` and of `1.0 − 1.0`
panic at `accurate-0.4.1/src/sum/ifastsum.rs:89`. fixed no: `1e-20` enters as 0 and `1e30` is
refused.

Mutant: `Exact` that ignores every insert after the 65,535th. Control: `f64` reads 0 for
`1e100 + 1 − 1e100`.

## log
- 2026-11-06T14:19Z created
- 2026-11-06T15:00Z open→doing — claimed by claude (14a6c070-cc69-4944-b798-52354e82e5d7)
- 2026-11-06T15:04Z doing→done — verify exit 0 @ ab75d88+2
