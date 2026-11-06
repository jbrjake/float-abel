---
id: fl-h2xjbts
title: "Measure the row 'merges partial sums': three workers' partials merged in two orders must read the exact total 15/8"
category: crate/proof
parent: fl-9p8yw9v
needs: [fl-2r5tb8c]
seq: 270
docs:
  - docs/DESIGN.md#what-an-incremental-engine-asks-of-a-weight-sp-laws
verify: 'all(exists examples/short-table/OUTPUT.md, contains examples/short-table/OUTPUT.md /^\| merges partial sums \| yes \| yes \| no \| no \| yes \| yes \| yes \| yes \|$/)'
status: open
created: 2026-11-06T14:19Z
---
Workers sum their shares, and the engine combines the partial states.

Adds `merge(a, b)` to `Column`.

Probe: workers hold `+0.5 +0.25`, `+0.125 +1e16` and `+1.0 −1e16`. Merged as `(w1 ⊕ w2) ⊕ w3`
and as `w3 ⊕ (w2 ⊕ w1)`, both must read 1.875, the exact total and itself an f64. The split is
chosen so that losing the 0.125 under 1e16 anywhere shows: plain `f64` reads 0.0 one-shot and
merged, and a merge that goes through a rounded f64 reads 0.75 in one order and 1.0 in the other.
Whether merged states have equal bits is the previous row's question, not this one's.

Expected, and the run decides: absent for xsum (its `Xsum` trait is `new`, `add_list`, `add`,
`sum`, `clear`, with no `Add<Self>` or `AddAssign<Self>`) and fsum (only `+= f64` and `-= f64`).
Present and right for float-abel (`+`), bitrep (`merge`), accurate (`Add<Self>`), num-rational,
bigdecimal and fixed (`+`).

Mutant: `Exact` that merges `Exact::from(other.to_f64())`. Control: `f64` reads 0.0.

## log
- 2026-11-06T14:19Z created
