---
id: fl-hzrz626
title: Re-take BENCH.md's crossover at f24b051 on both machines so the README's cost section rests on one commit
status: done
category: crate/measure
verify: "all(lacks README.md fa2e7b5, lacks BENCH.md fa2e7b5)"
docs:
  - BENCH.md#the-crossover
seq: 88
created: 2026-10-24T21:00Z
---

The README's "what it costs" section quotes two commits. Its table and its first line ("at commit
f24b051") come from BENCH.md § What reproducibility costs. Its crossover paragraph (8,300 / 54,000 /
20,800 / 113,000 rows, 116–1,180 batched, 4 µs against 77–499 µs at 10⁶) comes from BENCH.md § The
crossover, which is at fa2e7b5. The owner asked for the cost section aligned to one, the latest.

`src`, `benches`, `examples`, `Cargo.toml` and `Cargo.lock` are byte-identical between f24b051 and
HEAD (the one commit since is docs), so f24b051 is the latest code, and only the crossover needs
re-taking.

Work: `cargo bench --bench crossover` (43 rows, about four minutes) on NEON here and on AVX2 on
jons-amd-mini, `ps` checked once for competing work first. Rewrite § The crossover's three tables
and every derived figure in its prose. The crossover is the engine row's mean across its sizes
interpolated linearly on the re-sum row between the bracketing sizes, which reproduces the
fa2e7b5 table exactly. Relabel the section to f24b051, and update the README paragraph's figures
and drop its separate commit. docs/CROSSOVER.md is the owner's decision memo and cites its own
measurement at fa2e7b5, so it stays. Not committed: the owner asked for working-tree changes.

## log
- 2026-10-24T21:00Z created
- 2026-10-24T21:00Z open→doing — claimed by claude (a0a32f19-86b8-4ac5-b961-9cbdab04ae32)
- 2026-10-24T21:09Z doing→done — verify exit 0 @ 1b50150+7

## comments
- 2026-10-24T21:09Z [claude (a0a32f19-86b8-4ac5-b961-9cbdab04ae32)] Sitting, both machines at once: `cargo bench --bench crossover`, 43 rows each, exit 0. Run at HEAD
  1b50150, whose `src`, `benches`, `examples`, `Cargo.toml` and `Cargo.lock` are byte-identical to
  f24b051's (an empty `git diff f24b051 HEAD` over them), so BENCH.md labels it f24b051. The rsynced
  tree on jons-amd-mini hash-matched `git archive f24b051 src benches` (45 files, under `LC_ALL=C`
  sort; the default sort collates differently on Linux and gave a false mismatch).
  
  Conditions: NEON load 1.65 before with no other cargo; AVX2 load 0.20–1.33 through the run, and no
  `tpch_four_engin` in any of the 17 sidecar samples.
  
  Canaries against the fa2e7b5 sitting: the re-sum rows within 1–5% on NEON and 3% on AVX2, NEON's
  `isize` rows within 1–4%. AVX2's engine rows read about 7% higher on both diffs, `isize` included,
  so that is the host or the engine, not the lib. `bench-baseline.sh compare --no-rss`: every one of
  114 rows within 1.5× on both machines.
  
  What moved in the lib's own rows: the no-engine `Exact` deposit is 5.52 ns on NEON (4.71 at
  fa2e7b5) and 15.12 ns on AVX2 (13.16), and NEON's batched `Exact` rows rose 7–12% with `isize`
  flat. That is fl-60fzw87's question, noted there. The one-change crossover moves under 3% from it.
  
  Method check: the calculator (mean of the engine row over its sizes, linear interpolation on the
  re-sum row, proportional from (0, 0) below 100 rows) reproduces all 32 cells of the fa2e7b5 table
  from that table's own rows within 0.84%. docs/CROSSOVER.md stays at fa2e7b5: it is the owner's
  decision memo and cites its own measurement. Left uncommitted, as the owner asked.
