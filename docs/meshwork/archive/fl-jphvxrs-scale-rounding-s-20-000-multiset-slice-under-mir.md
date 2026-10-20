---
id: fl-jphvxrs
title: "Scale rounding's 20,000-multiset slice under miri so the miri job ends inside its hour"
category: crate/hardening
seq: 60
relates: [fl-f8brdgt]
docs:
  - docs/DESIGN.md#hardening-sp-hardening
verify: "contains tests/rounding.rs /agree\\(0x2a9ee, rounds\\(20_000\\)\\)/"
status: done
created: 2026-10-20T15:04Z
---
The `ci` workflow's miri job has hit its 60-minute ceiling on every run so far (the five of
2026-10-16, run 38957258898 among them: "The job has exceeded the maximum execution time of
1h0m0s"). Its log's last passing line, at 20:08 of a job started 19:45 and killed 20:46, is
`rounding::rounding_div_f64_matches_cpython_on_every_tier`; the next test in the harness's order
is `rounding_tiers_agree_on_a_slice_of_random_multisets`, `agree(0x2a9ee, 20_000)`, whose count
is a bare literal where every other sweep passes its count through `tests/support.rs`'s
`rounds()`, a hundredth under miri. ci.yml's own comment says the sweeps run a hundredth there.

Pass the count through `rounds()`: 200 multisets under miri, 20,000 natively. `agree`'s
assertions are ratios (at least half inexact, at least a thirty-second exact, at least a quarter
negative), so they hold at any count large enough, and the fixed seed makes 200 deterministic;
confirm the three hold at 200 by running the test under `cargo +nightly miri test --test main
rounding_tiers_agree_on_a_slice` here, then watch the miri job finish under its hour on the
pushed commit and record the run id in the close comment. The million-multiset test is `#[ignore]`
and never runs under miri.

## log
- 2026-10-20T15:04Z created
- 2026-10-20T15:31Z handoff by claude (4a8ded2c-6eb4-4638-a7f1-ee6da67e86ba)
- 2026-10-20T15:46Z handoff by claude (4a8ded2c-6eb4-4638-a7f1-ee6da67e86ba)
- 2026-10-20T17:04Z open→doing — claimed by claude (30b8d01c-d5ec-456d-a066-16edcae1c161)
- 2026-10-20T18:07Z doing→done — verify exit 0 @ 68415e0+1

## comments
- 2026-10-20T18:07Z [claude (30b8d01c-d5ec-456d-a066-16edcae1c161)] The miri job ended inside its hour on the pushed commit: run 39463529492 on 4a83a00 (e813edf is the change), job 118768785023, the `cargo miri test --locked --test main` step 49m26s (17:12:18 to 18:01:44) against the hour it had hit on every run before. `rounding::rounding_tiers_agree_on_a_slice_of_random_multisets` passed under miri at 17:47:42, 200 multisets, with agree's four ratio assertions holding at the fixed seed (and in 64 s on the M5 Max here). The harness reached the end of the suite for the first time: 175 tests, 135 passed, 39 ignored, 1 failed, and the one red is `binned_product_deposit_products_is_the_pairwise_fold_or_nothing` indexing 2999 into a slice `rounds(3000)` made thirty long, filed and fixed as fl-mewakns (c385a17, run 39471909369 in flight). The step's ten minutes of headroom is thin for a shared runner; the gap table from the log's timestamps is on fl-sq2sv0n, with the cap-edge oracle cross in tests/exact_cap.rs the longest single test at over sixteen minutes.
