---
id: fl-2r5tb8c
title: "Build the short table's machinery: the Column trait's core, recipes, the four evidence kinds, one panic guard, the renderer, and nine columns' insert, retract, read, view and batch"
category: crate/proof
parent: fl-9p8yw9v
needs: [fl-v56pe0k]
seq: 240
docs:
  - docs/DESIGN.md#what-an-incremental-engine-asks-of-a-weight-sp-laws
verify: 'all(exists examples/short-table/OUTPUT.md, contains examples/short-table/OUTPUT.md /^\| \| float-abel \| bitrep \| xsum \| fsum \| accurate \| num-rational \| bigdecimal \| fixed \|$/)'
status: open
created: 2026-11-06T14:19Z
---
Each later row adds at most one method to the trait, in all nine columns, so each row is one diff.

- `Column`: `NAME`, `State`, `empty`, `insert`, `retract` (the crate's own inverse), `read`,
  `view` (bytes, or `Debug` where that is the crate's only window), `batch` (its slice entry). No
  defaults: an absent method says why.
- `Outcome<T> = Result<T, Shortfall>`: `Refused`, `Panicked { message, at }`, `Absent`. A trait's
  `Absent` sits beside `const _: () = assert!(!impls!(Ty: Trait));`.
- States are built from recipes (`+0.5 +0.5`), never cloned, since xsum has no `Clone`. The log
  prints the recipe.
- One guard around every call: `catch_unwind` under a quiet hook, the location trimmed to
  `<crate>-<version>/<path>:<line>`.
- The renderer: the header from the eight names, a line per finished row, an evidence section per
  row.

| type | insert | retract | view | batch |
|---|---|---|---|---|
| `Exact` | `+=` | `-=` | `encode()` | `Sum` |
| `SumF64` | `add` | `try_unmerge` | `to_bytes` | none |
| `XsumSmall` | `add` | `add(−x)` | `Debug` | `[f64]::xsum()` |
| `FSum` | `+=` | `-=` | `Debug` | `with_all` |
| `OnlineExactSum<f64>` | `+=` | `+= −x` | `Debug` | `absorb` |
| `BigRational` | `from_float` | `-=` | numer, denom | none |
| `BigDecimal` | `from_f64` | `-=` | `as_bigint_and_exponent` | none |
| `I64F64` | `checked_from_num` | `-=` | `to_bits` | none |
| `f64` (control) | `+=` | `-=` | `to_bits` | `sum` |

`encode()` because `to_bytes` panics on the heap form; `checked_from_num` because `from_num`
panics on overflow with debug assertions on.

One file per column under `src/columns/` and per row under `src/rows/`, probes as `const` data
with each answer's reason beside it; about 1,000 lines, no file past 500.

Tests: the guard turns accurate's zero-state panic into `Panicked` at
`accurate-0.4.1/src/sum/ifastsum.rs:89`; one recipe built twice views equal.

## log
- 2026-11-06T14:19Z created
