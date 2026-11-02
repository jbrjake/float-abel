---
id: fl-0qp07ry
title: Re-take BENCH.md's reproducibility-cost table at the current src on both machines
status: done
category: crate/measure
verify: contains BENCH.md f24b051
docs:
  - BENCH.md#what-reproducibility-costs
created: 2026-10-23T20:50Z
---

Owner, in session: "for 2. please re-run the bench. do not commit changes to the bench.md just leave the file modified."

BENCH.md § What reproducibility costs is at 2a82388 (`Exact` one slot 4.94 / 15.89 ns, the 8-worker SUM 1.85× / 2.07×) while README § what it costs quotes bench-baseline.json at 85b93c2 (8.2 / 21.3 ns, 2.2–2.3× / 1.9–2.2×), so a reader following the README's link finds other numbers for the same rows. Re-take the table's three harnesses, `cargo bench --bench ops|simd|dd_sum`, on both machines (NEON on the M5 Max, AVX2 on jons-amd-mini) at the current src, and rewrite that section's table and prose at the new commit. Leave BENCH.md modified and uncommitted for the owner. Compare each sitting against bench-baseline.json with `scripts/bench-baseline.sh compare` and report every row past 1.5×; do not `write` the baseline.

## log
- 2026-10-23T20:50Z created
- 2026-10-23T20:50Z open→doing — claimed by claude (101dc294-a5c4-4a2c-9db5-a90033e6f8c5)
- 2026-10-23T21:18Z doing→done — verify exit 0 @ f24b051+4

## comments
- 2026-10-23T21:18Z [claude (101dc294-a5c4-4a2c-9db5-a90033e6f8c5)] Both sittings at f24b051, ops, simd and dd_sum in sequence: NEON on the M5 Max (rustc 1.99.0, macOS 26.5.2) 16:51–17:10, AVX2 on jons-amd-mini (rustc 1.96.1) 16:51–17:16. Before the start, ps showed no other cargo, criterion or bench binary on either host, and jons-amd-mini's load was 0.07 with no tpch_four_engines running, then or at the end.
  
  `scripts/bench-baseline.sh compare --all` against bench-baseline.json at 85b93c2: no row past 1.5× on either machine. NEON's highest is exact/adversarial/to_f64 at 1.49× (24.7 → 36.8 ns median), then dd_sum/wide/8w/serialized at 1.15×. AVX2's highest is dd_sum/exact/1w/typed at 1.05×. Canaries on NEON: f64 += 0.95×, Wide add 1.00×, isize dd_sum 0.99–1.01×. On AVX2: isize dd_sum 1.02–1.04×, Wide add 1.00×. So the README's 85b93c2 figures still hold; what was stale was BENCH.md's table at 2a82388.
  
  BENCH.md § What reproducibility costs is rewritten at f24b051 from criterion's point estimates (the file's stated method). Exact one slot: 8.57 ns, 15× on NEON, and 21.27 ns, 29× on AVX2. The 8-worker SUM: 2.33× serialized and 2.23× typed on NEON, 2.04× and 1.83× on AVX2. Left modified and uncommitted, per the owner. The one-slot gap against § The prototype table's 2a82388 rows is filed as fl-60fzw87. The criterion data stays in each host's target/criterion, and the AVX2 logs are on jons-amd-mini as float-abel/avx2-{ops,simd,dd_sum}.log.
