---
id: fl-qzxka6v
title: "Measure the row 'same bits for the same number, however reached': five pairs of histories that hold one number, compared through each crate's own view of its state"
category: crate/proof
parent: fl-9p8yw9v
needs: [fl-2r5tb8c]
seq: 260
docs:
  - docs/DESIGN.md#what-an-incremental-engine-asks-of-a-weight-sp-laws
verify: 'all(exists examples/short-table/OUTPUT.md, contains examples/short-table/OUTPUT.md /^\| same bits for the same number. however reached \| yes \| no \| no \| no \| no \| yes \| no \| yes \|$/)'
status: done
created: 2026-11-06T14:19Z
---
Consolidation adds equal-key diffs in an unspecified order and matches them structurally, so one
number has to be one bit pattern whatever history produced it.

Each route is two recipes that hold the same number; the column passes if every route's two views
are equal. A column with no view at all (no bytes, no `Debug`) is no: nobody can show its states
equal.

| route | left | right |
|---|---|---|
| split | `+1.0` | `+0.5 +0.5` |
| order | `+(1+ε) +(1+ε) +1` | `+(1+ε) +1 +(1+ε)` (ε = `f64::EPSILON`) |
| cancel | empty | `+0.1 −0.1` |
| return | `+1.0` | `+1.0 +0.5 −0.5` |
| excursion | `+1e-12` | `+1e-12 +1e12 −1e12` (float-abel passes through its heap form and back) |

Expected, and the run decides: float-abel yes, by `encode()` bytes. bitrep no: on the split
route `to_bytes` differ, because the state carries an insert count. xsum no: `Debug` differs on
split and cancel. fsum no: cancel leaves `FSum { partials: [0.0] }` against
`FSum { partials: [] }`. accurate no: `Debug` differs on split and order. num-rational yes: the
fraction is kept reduced. bigdecimal no: return leaves digits 10 at scale 1 against 1 at scale 0
(its `==` still says equal; that is the `==` row's question). fixed yes, by `to_bits`.

Mutant: `Exact` whose view appends an insert count. Control: `f64` fails order (`3.0000000000000004`
against `3.0`) and excursion (`0` against `1e-12`).

## log
- 2026-11-06T14:19Z created
- 2026-11-06T14:42Z open→doing — claimed by claude (14a6c070-cc69-4944-b798-52354e82e5d7)
- 2026-11-06T14:45Z doing→done — verify exit 0 @ f37253b+2
