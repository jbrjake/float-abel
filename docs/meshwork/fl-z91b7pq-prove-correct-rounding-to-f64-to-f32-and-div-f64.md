---
id: fl-z91b7pq
title: "Prove correct rounding — to_f64, to_f32 and div_f64 against CPython-generated vectors, and cross-tier agreement"
category: crate/proof
labels: [reproducible-floats, rf-critical-path]
docs: [docs/DESIGN.md#reads-and-rounding-sp-rounding]
needs: [fl-n7beqqc, fl-at6nh41, fl-atkbp6q]
seq: 60
verify: "all(exists tests/data/rounding_vectors.json, run cargo test package=float-abel rounding_)"
status: open
created: 2026-10-11T15:02Z
covers:
  - ref: docs/DESIGN.md#sp-rounding
    sha: d2071b1880ed28f2421e76a0f2c0b6fc50d0d2d1e18b9b1e9ab8859e623d103e
---
Per DESIGN.md § Reads and rounding. `scripts/gen_rounding_vectors.py` (committed; it uses
`fractions.Fraction`, `float()`, and `struct` for `f32`) writes `tests/data/rounding_vectors.json`.
The vectors cover ties at every tier boundary, subnormal results, overflow to ±∞, and the `to_f32`
cases where rounding via `f64` first would differ.

Tests `rounding_*` read the JSON. They also check that `Fixed256`, `Wide` and `Exact` agree on 10⁶
random finite multisets inside `Fixed256`'s window.

## log
- 2026-10-11T15:02Z created
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-rounding @d2071b1880ed
