---
id: fl-yfyzeg7
title: "Measure the row 'standalone negatives': negate a state without its inputs, let the negative arrive first, and read zero once the inputs follow"
category: crate/proof
parent: fl-9p8yw9v
needs: [fl-2r5tb8c]
seq: 280
docs:
  - docs/DESIGN.md#what-an-incremental-engine-asks-of-a-weight-sp-laws
verify: 'all(exists examples/short-table/OUTPUT.md, contains examples/short-table/OUTPUT.md /^\| standalone negatives \| yes \| no \| no \| no \| no \| yes \| yes \| yes \|$/)'
status: open
created: 2026-11-06T14:19Z
---
`negate` runs inside `iterate`, `antijoin` and `reduce_abelian`, and consolidation can meet a −x
diff before its +x, so every state needs a negative that exists on its own.

Adds `negate(state)` to `Column`.

Probe: S is `+0.1 +0.2`. `negate(S)` must exist without knowing S's inputs. Then the history
where the negative comes first: start from `negate(S)`, insert 0.1, insert 0.2, and read exactly
0.0. Equality with the empty state is the `==` row's question; this row asks only that the
negative exists and cancels exactly.

Expected, and the run decides: float-abel, num-rational, bigdecimal and fixed yes, by `Neg`.
bitrep no: no `Neg` (absent), and its nearest operation, `try_unmerge` into an empty accumulator,
returns false (refused). xsum, fsum and accurate no: no `Neg` (absent); retracting a value an
accumulator already holds is not a negative of a state.

Mutant: `Exact` whose negation is `Exact::from(-self.to_f64())`. Control: `f64` reads
`-2.7755575615628914e-17`.

## log
- 2026-11-06T14:19Z created
