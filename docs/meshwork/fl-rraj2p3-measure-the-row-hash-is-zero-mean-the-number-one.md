---
id: fl-rraj2p3
title: "Measure the row '== / Hash / is_zero mean the number': one number by two histories compares and hashes equal, a cancelled state is zero, two numbers differ"
category: crate/proof
parent: fl-9p8yw9v
needs: [fl-2r5tb8c]
seq: 290
docs:
  - docs/DESIGN.md#what-an-incremental-engine-asks-of-a-weight-sp-laws
verify: 'all(exists examples/short-table/OUTPUT.md, contains examples/short-table/OUTPUT.md /mean the number \| yes \| no \| no \| no \| no \| yes \| yes \| yes \|$/)'
status: open
created: 2026-11-06T14:19Z
---
Builders dedup by `==`, `is_zero` deletes an update, and `count_total` makes the weight data, so
records merge by comparison and route to workers by `Hash`.

Adds `same(a, b)` (equal by `==`, and equal hashes) and `is_zero(s)` to `Column`; a crate with
`==` but no `is_zero` is asked `s == empty`.

Probes:

- `+1.0` and `+0.5 +0.5` are `==` and hash equal.
- `+0.1 −0.1` is zero, and `==` the empty state.
- `+0.1` and `+0.2` are not `==`, so an `Eq` that says yes to everything fails.

Hashes use `std::hash::DefaultHasher` and are compared inside the run, never printed: their values
are not stable across Rust releases.

Expected, and the run decides: float-abel, num-rational and fixed yes; bigdecimal yes as well
(`1.0 == 1.00` with equal hashes, though its digits differ, which the bits row records). bitrep
no: its `Eq` derives over the insert count, so `+1.0` and `+0.5 +0.5` are not equal (ran), and it
has no `Hash` (absent). xsum: no `PartialEq` or `Hash`. fsum: `Clone` and `Debug` only.
accurate: no `PartialEq` or `Hash`. All three absent.

Mutant: `Exact` whose `Hash` mixes in an insert count while `==` stays numeric, so the hash leg
alone fails. Control: `f64` has no `Hash` (absent).

## log
- 2026-11-06T14:19Z created
