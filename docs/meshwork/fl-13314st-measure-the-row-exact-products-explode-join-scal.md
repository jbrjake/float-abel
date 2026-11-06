---
id: fl-13314st
title: "Measure the row 'exact products (explode, join)': scale a state by counts, and square 1 + 2^-40 keeping the 2^-80 that a rounded or 64.64 product drops"
category: crate/proof
parent: fl-9p8yw9v
needs: [fl-2r5tb8c]
seq: 300
docs:
  - docs/DESIGN.md#what-an-incremental-engine-asks-of-a-weight-sp-laws
verify: 'all(exists examples/short-table/OUTPUT.md, contains examples/short-table/OUTPUT.md /^\| exact products \(explode. join\) \| yes \| no \| no \| no \| no \| yes \| yes \| no \|$/)'
status: open
created: 2026-11-06T14:19Z
---
`explode` multiplies each value by its ±1 diff, and `join` evaluates `(a + b) × c` as
`a × c + b × c`, so a weight multiplies exactly, by counts and by other weights.

Adds `scale(s, k: i64)` and `mul(a, b)` to `Column`.

Probes:

- By count: `scale(+0.1 +0.2, 3)` reads what `+0.1 +0.2 +0.1 +0.2 +0.1 +0.2` reads, and
  `scale(+0.1 +0.2, -1)` then `+0.1 +0.2` reads exactly 0.0.
- By weight: `mul(+(1 + 2⁻⁴⁰), +(1 + 2⁻⁴⁰))`, then retract `1 + 2⁻³⁹`, reads 2⁻⁸⁰
  (`8.271806125530277e-25`). The exact square is `1 + 2⁻³⁹ + 2⁻⁸⁰`; a product rounded to 53 bits,
  or truncated at 64 fractional bits, drops the 2⁻⁸⁰ and reads 0.

Expected, and the run decides: float-abel yes (`Mul<Exact>`, `Mul<i64>`); num-rational and
bigdecimal yes. fixed no: `I64F64` multiplies, and the by-weight probe reads 0 (ran). bitrep no:
`SumF64` has no `Mul` (absent; `DotF64` is a different type). xsum, fsum and accurate: no `Mul`
(absent).

Mutant: `Exact` whose product is `Exact::from(a.to_f64() * b.to_f64())`. Control: `f64` reads 0.

## log
- 2026-11-06T14:19Z created
