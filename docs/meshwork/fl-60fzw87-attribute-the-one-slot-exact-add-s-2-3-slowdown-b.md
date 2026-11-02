---
id: fl-60fzw87
title: "Attribute the one-slot Exact add's 2.3× slowdown between 2a82388 and 85b93c2 on NEON to the lib or the ops harness, and put it to the owner"
status: open
category: crate/measure
relates: [fl-0qp07ry]
verify: contains BENCH.md /### The one-slot add across the heap arm/
docs:
  - BENCH.md#the-prototype-table-re-taken
created: 2026-10-23T21:12Z
handoff: |
  The figures in the body come from one NEON sitting at f24b051,
  2026-10-23: `exact/same-scale/deposit (from_f64 + add)` 8.57 ns and `+=,
  both operands built` 8.60 ns (point estimates), with the f64 floor at
  0.57 ns. On AVX2 the same rows read 21.27 and 19.35 ns, against 15.89
  and 10.89 at 2a82388: 1.34× for the deposit and 1.78× for the `+=`, so
  the `+=` is past 1.5× there too. bench-baseline.json at 85b93c2 holds
  the slower figures on both machines, so 85b93c2..f24b051 is clean; the
  move is in 2a82388..85b93c2. Commits there touching benches/ops.rs:
  fa2e7b5, 09c92b5, 16db257. Commits touching src/exact: 1c76193, 09c92b5,
  2ca2ddd, 16db257, 6c04216. Start with a control: `git worktree add` at
  2a82388, build `cargo bench --bench ops` there and here, keep both
  binaries from <target>/release/deps (the newest `ops-*` without `.d`),
  then run them back to back with `CRITERION_HOME` apart and `--bench
  '^exact/same-scale/|^f64/same-scale/'`. If the gap survives with a
  single harness (the 2a82388 benches/ops.rs built against each lib, where
  it compiles), bisect the lib. Nothing is attributed yet.
---

Found during fl-0qp07ry. On NEON, `benches/ops.rs`'s same-scale one-slot rows for `Exact` read, at point estimates:

| row | 2a82388 (BENCH.md § The prototype table) | f24b051 (this sitting) | ratio |
|---|--:|--:|--:|
| `Exact::deposit` (`from_f64` + add) | 4.94 ns | 8.57 ns | 1.74× |
| `Exact += &Exact`, operands built | 3.81 ns | 8.60 ns | 2.26× |
| `f64 +=` floor (canary) | 0.58 ns | 0.57 ns | 0.98× |

bench-baseline.json at 85b93c2 already holds the slower figures (8.21 and 8.69 ns medians), so the move is between 2a82388 and 85b93c2 and no later. Candidates: 09c92b5 (the heap arm, whose message says "The compact fast paths keep same-scale data at the word's cost", and which also edited benches/ops.rs), 1c76193 (the compact add deciding on the sum's span), and the ops harness itself. The heap arm's budget, BENCH.md § The heap arm's budget, was read on the `dd_sum` rows only (1.08–1.09× at one worker), never on the one-slot add.

Attribute it in one sitting: build the ops bench at 2a82388 and at f24b051 into one harness shape (the variant lib in the control's benches/ops.rs, as the heap-arm A/B did for dd_sum), run the same-scale `exact` and `f64` rows back to back, and bisect the lib side across 09c92b5 and 1c76193 if the lib owns the gap. Past 1.5× this is the owner's call per CLAUDE.md § Performance: accept the cost, or fix the compact path. Record the A/B under its own heading in BENCH.md.

## log
- 2026-10-23T21:12Z created
- 2026-10-23T21:18Z handoff by claude (101dc294-a5c4-4a2c-9db5-a90033e6f8c5)
- 2026-10-23T21:18Z handoff by claude (101dc294-a5c4-4a2c-9db5-a90033e6f8c5)

## comments
- 2026-10-24T21:09Z [claude (a0a32f19-86b8-4ac5-b961-9cbdab04ae32)] Data point from fl-hzrz626's crossover sitting at f24b051: the no-engine Exact deposit (benches/crossover.rs, crossover/no-engine, 2^20 deposits into one slot) reads 5.52 ns a change on NEON against 4.71 at fa2e7b5, and 15.12 ns on AVX2 against 13.16; NEON's batched Exact rows rose 7-12% while the isize rows held flat, so the engine is not the cause there.
