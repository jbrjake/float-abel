---
id: fl-6shtzpj
title: "Publish the laws test kit as a feature and run every exact tier through it, with a naive f64 control that must fail"
category: crate/laws
labels: [reproducible-floats, rf-critical-path]
docs: [docs/DESIGN.md#the-laws-kit-sp-laws-kit, docs/DESIGN.md#what-an-incremental-engine-asks-of-a-weight-sp-laws]
needs: [fl-n7beqqc, fl-at6nh41, fl-atkbp6q]
seq: 80
verify: "all(exists src/laws.rs, run cargo test package=float-abel laws_)"
status: open
created: 2026-10-11T15:02Z
covers:
  - ref: docs/DESIGN.md#sp-laws-kit
    sha: 2246c38eec815c9342db2654d61ce495f9e6e64acbc783b5ba59d0d91427cb28
  - ref: docs/DESIGN.md#sp-laws
    sha: d04a257c60cce5cd8d14fd90d26d72979fabd4bf84b6d91f25a22d89077bc56f
---
Per DESIGN.md § The laws kit: `float_abel::laws::{check_abelian_group, check_ring, check_canonical,
check_consolidation_invariance}`, generic over a `Strategy`. Its docs show a stranger running it
against their own type.

`laws_*` tests run the kit against `Fixed256`, `Wide` and `Exact`. They also run it against a naive
`f64` wrapper as the negative control, which must **fail** associativity and inverse.

## log
- 2026-10-11T15:02Z created
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-laws-kit @2246c38eec81
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-laws @d04a257c60cc
