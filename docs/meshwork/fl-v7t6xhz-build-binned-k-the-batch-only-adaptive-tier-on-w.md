---
id: fl-v7t6xhz
title: "Build Binned<K> — the batch-only adaptive tier on Wide's grid, its Add, the conversions, its error bound, and compile-fail tests that keep it out of the engine weight traits"
category: crate/types
labels: [reproducible-floats, rf-tail]
docs: [docs/DESIGN.md#binned-sp-binned]
needs: [fl-at6nh41]
seq: 100
verify: run cargo test package=float-abel binned_
status: open
created: 2026-10-11T15:02Z
covers:
  - ref: docs/DESIGN.md#sp-binned
    sha: 28d2c310aff6a0a29005a96faae1d49e0557c54392112794fee21d28bddbecf2
---
`src/binned.rs` per DESIGN.md § Binned.

Tests `binned_*`:
- a fold is bit-identical under 1,000 shuffles and across 1–64 merge partitions;
- `into_wide().to_f64()` is within `n·2^(−60(K−1))·max|x|` of `Exact` on random and adversarial
  multisets;
- the history `1e30`, `1e-10`, `−1e30`, through a *fresh* fold, shows the documented truncation;
- `trybuild` cases under the features: `Binned<3>` fails to compile both as a differential-dataflow
  `Collection` diff and as a DBSP `OrdIndexedWSet` weight.

## log
- 2026-10-11T15:02Z created
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-binned @28d2c310aff6
