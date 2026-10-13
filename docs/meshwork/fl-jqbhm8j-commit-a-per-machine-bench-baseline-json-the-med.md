---
id: fl-jqbhm8j
title: "Commit a per-machine bench-baseline.json — the medians and peak RSS criterion measured, written by a script from its estimates and compared at 1.5×, per CLAUDE.md § Performance"
status: open
category: crate/measure
labels: [reproducible-floats]
discovered-from: fl-e4jjqf1
relates: [fl-e4jjqf1]
verify: "all(exists bench-baseline.json, exists scripts/bench-baseline.sh)"
docs:
  - CLAUDE.md#performance
seq: 175
created: 2026-10-13T16:09Z
---

CLAUDE.md § Performance asks for criterion benches against a committed per-machine
`bench-baseline.json` (median of 7+ runs after warm-up, plus peak RSS) so a regression past 1.5×
is caught and taken to the owner. BENCH.md carries the numbers as prose tables with the machine
and the commit, and nothing in the tree holds them in a form a script can diff. Found by Measure
(fl-e4jjqf1) and left undone there on the owner's instruction to wrap up.

What to build: a `bench-baseline.json` keyed by machine (NEON: Apple M5 Max; AVX2: AMD Ryzen 7
5700U) holding each criterion id's median per element and the end-to-end peak RSS, written from
`target/criterion/**/new/estimates.json` by a script under scripts/ (criterion writes a group
named `a/b` to a directory `a_b`), and a compare mode that prints every row past 1.5× of the
baseline. Advisory, never a gate: the standing decision keeps speed out of the hooks. The
session's scratchpad script that read the estimates is the shape to start from; it is not in the
tree.

## log
- 2026-10-13T16:09Z created
