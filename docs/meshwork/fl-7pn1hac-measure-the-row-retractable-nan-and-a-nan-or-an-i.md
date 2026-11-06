---
id: fl-7pn1hac
title: "Measure the row 'retractable NaN and ∞': a NaN or an infinity enters, reads as IEEE says, and leaves with the finite sum restored"
category: crate/proof
parent: fl-9p8yw9v
needs: [fl-2r5tb8c]
seq: 310
docs:
  - docs/DESIGN.md#nan-and--sp-specials
verify: 'all(exists examples/short-table/OUTPUT.md, contains examples/short-table/OUTPUT.md /^\| retractable NaN and ∞ \| yes \| no \| no \| no \| no \| no \| no \| no \|$/)'
status: open
created: 2026-11-06T14:19Z
---
f64 includes NaN and ±∞, and a `SUM` over a changing column has to recover when the NaN row is
deleted.

Probes (retract means the column's own retraction of that same value):

- `+1.5 +NaN` reads NaN; retract NaN; reads 1.5.
- `+1.5 +∞` reads +∞; retract +∞; reads 1.5.
- `+1.5 +∞ +(−∞)` reads NaN; retract both; reads 1.5.
- Retract NaN from empty, then `+NaN`, then `+1.5`: reads 1.5. A negative NaN count with no NaN
  yet is a state a group holds and IEEE cannot.

Expected, and the run decides: float-abel yes, by signed counts (DESIGN.md § NaN and ±∞). bitrep
no: `try_unmerge` of a NaN returns false (refused); its special flags are sticky. xsum no:
retracting by `add(−NaN)` reads NaN (ran). fsum no: `-= NaN` reads NaN (ran). accurate no: a read
after `+= ∞` panics at `accurate-0.4.1/src/sum/ifastsum.rs:89` with debug assertions on.
num-rational, bigdecimal and fixed no: `from_float(NaN)`, `from_f64(NaN)` and
`checked_from_num(NaN)` are `None` (refused).

Mutant: `Exact` with a sticky NaN flag that retraction does not clear. Control: `f64` keeps the NaN.

## log
- 2026-11-06T14:19Z created
