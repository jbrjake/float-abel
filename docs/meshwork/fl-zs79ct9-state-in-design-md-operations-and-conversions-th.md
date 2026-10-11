---
id: fl-zs79ct9
title: "State in DESIGN.md § Operations and conversions that Wide refuses products off its grid by name, as built"
category: crate/types
labels: [reproducible-floats]
relates: [fl-at6nh41]
docs:
  - docs/DESIGN.md#operations-and-conversions-sp-ops
  - docs/DESIGN.md#wide-sp-wide
verify: contains docs/DESIGN.md /refused by name on .Fixed. and .Wide./
seq: 44
status: open
created: 2026-10-11T17:24Z
---
Owner's ruling, 2026-10-11, on `Wide::deposit_product` for a product outside `Wide`'s window (a set
bit below `2^-1074`, or a magnitude at or past `2^1037`): "Refuse by name". The code already does
(`OutOfWindow::OffGrid` and `OutOfWindow::AboveCeiling`, state untouched; never `NotFinite`), and
`tests/wide.rs` pins it. The design's sentence "deposits the exact 106-bit product on every tier
(refused on `Fixed` when it is off-grid)" reads as if `Wide` accepts every product.

Rewrite that sentence so it says the product is refused by name on `Fixed` and `Wide` when it lies
outside the window, and that `Exact` is the tier that holds every product. § Wide's "never refuses"
gets the same qualification. Repin the drifted clauses on the live tasks that cover them.

## log
- 2026-10-11T17:24Z created
