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
status: done
created: 2026-10-11T17:24Z
covers:
  - ref: docs/DESIGN.md#sp-ops
    sha: 7675d055f5a4e08218fb87c0e966dbe6b45fa34b6c986dbf4cb55c086bcce4dc
  - ref: docs/DESIGN.md#sp-wide
    sha: 29ff969fbdb9994b87f98cdf5047fc40d02a85e7365f89961f6ec757d3ac0938
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
- 2026-10-11T18:12Z cover docs/DESIGN.md#sp-ops @f7da3b4f752d
- 2026-10-11T18:12Z cover docs/DESIGN.md#sp-wide @f9cd0e6f7109
- 2026-10-11T18:27Z open→doing — claimed by claude (0f58a8d2-1e20-4d5a-ab10-caf961d3912f)
- 2026-10-11T18:28Z cover docs/DESIGN.md#sp-ops @7675d055f5a4
- 2026-10-11T18:28Z cover docs/DESIGN.md#sp-wide @29ff969fbdb9
- 2026-10-11T18:28Z doing→done — verify exit 0 @ fc8888b+3
