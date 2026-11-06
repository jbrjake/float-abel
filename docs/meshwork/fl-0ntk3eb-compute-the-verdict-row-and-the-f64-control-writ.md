---
id: fl-0ntk3eb
title: "Compute the verdict row and the f64 control, write the lesson into main.rs's module doc, and read the whole OUTPUT.md against the target table cell by cell"
category: crate/proof
parent: fl-9p8yw9v
needs: [fl-6ht6aa6, fl-qzxka6v, fl-h2xjbts, fl-yfyzeg7, fl-rraj2p3, fl-13314st, fl-7pn1hac, fl-sw53v5t]
seq: 330
docs:
  - docs/DESIGN.md#what-an-incremental-engine-asks-of-a-weight-sp-laws
verify: 'all(exists examples/short-table/OUTPUT.md, contains examples/short-table/OUTPUT.md /^\| \*\*usable as the weight\*\* \| \*\*yes\*\* \| \*\*no\*\* \| \*\*no\*\* \| \*\*no\*\* \| \*\*no\*\* \| \*\*no\*\* \| \*\*no\*\* \| \*\*no\*\* \|$/, contains examples/short-table/OUTPUT.md /^## Control: plain f64/)'
status: open
created: 2026-11-06T14:19Z
---
- The bold verdict row is the AND of the eight cells per column, computed, never written.
- `## Control: plain f64` below the evidence: `f64` through every probe, with its verdicts.
- Tests: float-abel reads yes on all eight rows; the control reads yes on the first row and no on
  the other seven; every row's mutant reads no on its row.
- `src/main.rs`'s module doc is the lesson, in the voice of `examples/dd_sum.rs`: what a weight is
  to an engine, one sentence per property and why the engine needs it, how to read the four
  evidence kinds, why every answer is an f64, and the command that regenerates `OUTPUT.md`.
- Then read `OUTPUT.md` whole against the parent task's table, cell by cell, and every evidence
  line against what its probe claims. A cell that differs goes to the owner with its evidence
  lines before this task closes.

## log
- 2026-11-06T14:19Z created
