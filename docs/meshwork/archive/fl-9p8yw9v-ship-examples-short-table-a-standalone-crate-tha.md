---
id: fl-9p8yw9v
title: "Ship examples/short-table: a standalone crate that runs float-abel and seven neighbours through the eight properties an engine's weight needs, prints the comparison table it measures, and commits that output"
category: crate/proof
seq: 220
docs:
  - README.md#how-its-checked
  - docs/DESIGN.md#what-an-incremental-engine-asks-of-a-weight-sp-laws
verify: 'all(exists examples/short-table/OUTPUT.md, contains scripts/regression.sh examples/short-table/OUTPUT.md, contains examples/short-table/OUTPUT.md /^\| \*\*usable as the weight\*\* \| \*\*yes\*\*/)'
status: done
created: 2026-11-06T14:19Z
---
The README is to carry this table, and a program anyone can run has to produce it:

| | float-abel | bitrep | xsum | fsum | accurate | num-rational | bigdecimal | fixed |
|---|---|---|---|---|---|---|---|---|
| holds every f64 exactly | yes | yes | yes | yes | yes | yes | yes | no |
| same bits for the same number, however reached | yes | no | no | no | no | yes | no | yes |
| merges partial sums | yes | yes | no | no | yes | yes | yes | yes |
| standalone negatives | yes | no | no | no | no | yes | yes | yes |
| `==` / `Hash` / `is_zero` mean the number | yes | no | no | no | no | yes | yes | yes |
| exact products (explode, join) | yes | no | no | no | no | yes | yes | no |
| retractable NaN and ∞ | yes | no | no | no | no | no | no | no |
| correct on every input tested | yes | yes | no | no | no | yes | yes | no |
| **usable as the weight** | **yes** | **no** | **no** | **no** | **no** | **no** | **no** | **no** |

Each row is a property an engine needs of a weight (DESIGN.md § What an incremental engine asks of
a weight); a column is usable only if every cell in it is yes.

Deliverable: `examples/short-table/`, a package excluded from the workspace like `fuzz/`, whose
`cargo run` prints the table and the evidence for every cell, and `OUTPUT.md` beside it, the
committed output, which the push gate re-derives and diffs.

Rules for every child:

1. Every cell is run, and the verdict row is computed.
2. Every no prints its evidence: ran, refused, panicked, or absent (a trait's absence is a `const`
   `impls!` assertion, so a version bump that adds it breaks the build).
3. Each column uses its crate's strongest API, so a no is the crate's limit.
4. Every input's exact answer is an f64, so no oracle is needed.
5. Probes are fixed first. A cell that disagrees with this table goes to the owner with its
   evidence, and no probe changes to match.
6. Each row ships a mutant, `Exact` with that one property broken, which the row must mark no.
   Plain `f64` runs as a control, outside the table.
7. Output is byte-identical in debug and release and on aarch64 and x86_64: no timings, hashes,
   paths or float-abel version.

Out of scope: the README, cost (BENCH.md carries it), and engines.

## log
- 2026-11-06T14:19Z created
- 2026-11-06T15:16Z open→done — verify exit 0 @ 46e848b+5
