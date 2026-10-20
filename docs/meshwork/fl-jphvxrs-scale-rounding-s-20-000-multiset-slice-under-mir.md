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
handoff: |
  Next session: this is the one buildable, un-held task in `ready`, and it
  is one line plus a watch.
  At 79cf25b on origin, CI run 39449087008 is the first `ci` run since the
  smoke job's meshwork
  failure was fixed (ef90777) and dbsp's floor moved to 0.367 (db4e5d9);
  its gate job's conclusion
  is what fl-f8brdgt closes on, and its miri job is what this task is
  about. On every `ci` run so far
  the miri job has died at the 60-minute ceiling after
  `rounding::rounding_div_f64_matches_cpython_on_every_tier`
  passed, and the next test in the harness's order is
  `rounding_tiers_agree_on_a_slice_of_random_multisets`,
  `agree(0x2a9ee, 20_000)` in tests/rounding.rs, whose count is a bare
  literal where every other sweep
  goes through `tests/support.rs`'s `rounds()` (a hundredth under miri, at
  least one).
  
  The fix: `agree(0x2a9ee, rounds(20_000))` with `rounds` imported from
  `crate::support` (see how
  tests/simd.rs scales its `MULTISETS`), so miri runs 200 multisets.
  `agree` asserts ratios (at
  least half inexact, at least a thirty-second exact, at least a quarter
  negative, at least half
  inexact in f32) and the seed is fixed, so confirm the four hold at 200
  by running
  `cargo +nightly miri test --test main rounding_tiers_agree_on_a_slice`
  here (miri builds to
  `<CARGO_TARGET_DIR>/miri`, its own lock), then commit, push, and watch
  the miri job finish under
  its hour on that commit; record the run id in the close comment. If the
  job still times out,
  the next suspect is whatever test follows in the same job, found by the
  `date +%T` prefix trick
  in the build-environment memory. Also pending for the owner, not this
  task: the three dependabot
  PRs (actions/checkout 7.0.1, actions/cache 6.1.0,
  actions/upload-artifact 7.0.2) whose `ci`
  runs were red on the same smoke failure and need a rebase onto main to
  re-run.
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
