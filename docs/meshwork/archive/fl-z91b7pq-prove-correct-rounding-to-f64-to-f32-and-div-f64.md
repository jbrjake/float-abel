---
id: fl-z91b7pq
title: "Prove correct rounding — to_f64, to_f32 and div_f64 against CPython-generated vectors, and cross-tier agreement"
category: crate/proof
labels: [reproducible-floats, rf-critical-path]
docs: [docs/DESIGN.md#reads-and-rounding-sp-rounding]
needs: [fl-n7beqqc, fl-at6nh41, fl-atkbp6q]
seq: 60
verify: "all(exists tests/data/rounding_vectors.json, run cargo test package=float-abel rounding_)"
status: done
created: 2026-10-11T15:02Z
covers:
  - ref: docs/DESIGN.md#sp-rounding
    sha: a246b9799af15fa060ed301a4d1e7f206446621fec2f72f674c6149a3ac79b99
relates: [fl-a2wrx7w, fl-6shtzpj, fl-v7t6xhz, fl-cfx001b]
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
- 2026-10-11T19:01Z handoff by claude (9595457b-430f-4785-af6d-c23dc1a77cde)
- 2026-10-11T19:01Z handoff by claude (9595457b-430f-4785-af6d-c23dc1a77cde)
- 2026-10-12T12:36Z open→doing — claimed by claude (be8fad00-93e4-42e3-8b8f-40c127bab5c7)
- 2026-10-12T12:51Z cover docs/DESIGN.md#sp-rounding @a246b9799af1
- 2026-10-12T12:51Z doing→done — verify exit 0 @ fec0b3d+1
