---
id: fl-jphvxrs
title: "Scale rounding's 20,000-multiset slice under miri so the miri job ends inside its hour"
category: crate/hardening
seq: 60
relates: [fl-f8brdgt]
docs:
  - docs/DESIGN.md#hardening-sp-hardening
verify: "contains tests/rounding.rs /agree\\(0x2a9ee, rounds\\(20_000\\)\\)/"
status: open
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
