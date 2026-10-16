---
id: fl-jqbhm8j
title: "Commit a per-machine bench-baseline.json — the medians and peak RSS criterion measured, written by a script from its estimates and compared at 1.5×, per CLAUDE.md § Performance"
status: done
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
- 2026-10-16T18:13Z handoff by claude (cd1130c2-56bf-432b-bcb1-8ed1c474e8fa)
- 2026-10-16T18:28Z open→doing — claimed by claude (2ba3bd95-5552-4160-8df2-82be1495b54b)
- 2026-10-16T19:10Z doing→done — verify exit 0 @ e9cc317+3

## comments
- 2026-10-16T19:10Z [claude (2ba3bd95-5552-4160-8df2-82be1495b54b)] Done at e9cc317. scripts/bench-baseline.sh is a shell entry (cd to the repo root, `python3 -I scripts/bench-baseline.py`) over a Python program with two verbs. `write` reads every `<criterion>/**/new/benchmark.json` and `estimates.json` (criterion writes a group `a/b` to a directory `a_b`; `full_id` and `throughput.Elements` come from benchmark.json, the median point estimate from estimates.json), measures the peak resident set of the five dd_sum rows BENCH.md quotes (`exact|isize` × `1w|8w` serialized and `wide/1w/serialized`) by running the newest `dd_sum-*` bench binary under `/usr/bin/time -l` (macOS, bytes) or `-v` (Linux, kB) with `CRITERION_HOME` set to a temp dir so the sitting's estimates stay as they were, and replaces this machine's entry in bench-baseline.json, keyed by the CPU brand string with the commit, date, OS and rustc. `compare` prints every row whose median per element is past `--threshold` (1.5×) of the recorded one, worst first, plus ids new or missing on either side; `--rss` re-measures the resident sets, `--all` prints every row; exit 0 always, advisory by the standing decision. Validated this session: `write` on the prior sitting's data gave 124 rows under `Apple M5 Max`; `compare` on identical data printed no row; a deliberate halving of two recorded medians came back as two rows at 2.00× (the deliberate-break rule). The sittings: NEON 26 min (ops 183 s, simd 192, wire 109, crossover 266, dd_sum 805 for 24 rows), AVX2 31 min (256, 196, 117, 250, 1051), each with the old criterion dir moved aside first so every `new/` is the sitting's; the AVX2 chain guarded each target against the owner's tpch binary and none overlapped; no load wait on either box, per the owner's correction. Canaries: NEON f64 floor 1.00×, Wide add 1.01×, isize rows 1.01–1.03× of the prior data; AVX2 floor 0.75 ns, Wide add 29.4 ns, isize rows within 0–6% of BENCH.md's (720 vs 766 ms serialized 1w, 603 vs 603 typed). Both entries at 85b93c2, 114 ids each, the same ids. Resident sets with the three-fixture harness: 11.4–12.4 GB on every NEON row and 7.08 GB on every AVX2 row, the check phase's (every configuration once before any row is timed), so those rows compare only against the entry, which BENCH.md's intro now says. Two robustness points found on the AVX2 box: the synced tree has no .git and rustc is off the login PATH, so the script records `unknown` rather than raising when either is absent, and takes `--commit`. The RSS write on AVX2 ran with `python3 -I scripts/bench-baseline.py` from the tree since the .sh needs git. Not done: no CI job runs `compare`; the standing decision keeps speed out of the gates and nobody asked for an advisory job.
