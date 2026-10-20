---
id: fl-wpgpd4q
title: "Make the miri job's thread count known and cut the heap-ring dot product and wide folds under miri, or ask the owner to raise the job's hour"
status: open
category: crate/hardening
relates: [fl-sq2sv0n, fl-btst4df]
verify: contains .github/workflows/ci.yml /nproc/
docs:
  - docs/DESIGN.md#hardening-sp-hardening
seq: 62
created: 2026-10-20T20:16Z
handoff: |
  Next session: main is at the first fully green `ci` runs (39471909369
  and 39483989145, every job),
  and this is the one open runner issue. The dbsp job is fixed for good
  (the disk step and the exact
  cache key in ci.yml) and the miri job ends inside its hour at 41 to 43
  minutes, which is the problem: the
  suite step's wall is the suite's total work over the runner's threads,
  the box varies a quarter
  between runs, and the ceiling is sixty.
  
  Step one is a line in `.github/workflows/ci.yml`'s miri job, `nproc`
  next to the versions in the
  install step, so the thread count is in every log (the repo is private;
  a two-core hosted runner
  would explain 1312 s of single-threaded work here landing at 40 minutes
  there). Step two, the two
  cuts, each a miri-specific count the way `tests/simd.rs` does it (`const
  MULTISETS: u64 = if
  cfg!(miri) { 20 } else { 10_000 }`), not `rounds()`: in
  `tests/exact_heap_ring.rs`
  `exact_heap_ring_a_dot_product_of_mixed_pairs_matches_the_oracle_both_ways`
  has `let n =
  rounds(100_000)`, 1000 pairs under miri at 110 s here, try 100; in
  `tests/wide_group.rs`
  `wide_folds_are_order_independent` draws `(0..2_000)` and asserts a
  special was drawn ("2000
  draws with no special; reseed"), so a smaller miri count needs the seed
  checked, by running the
  test under miri after the edit. Measure each with `cargo +nightly miri
  test --test main <name>`
  wrapped in `date +%T` before and after (the harness's "finished in" is
  miri's virtual clock).
  Then push and read the step's wall from `gh run view <run> --json jobs`;
  the per-test gap table
  is a short python over the job log's timestamps (`search "gap table"`
  finds the comment that
  records one, with the method), the log fetched with `gh api
  repos/jbrjake/float-abel/actions/jobs/<id>/logs` once the whole run
  completes.
  
  The ceiling itself is the owner's: `timeout-minutes: 90` on the miri job
  costs billed minutes and
  nothing else; ask, with the measured step times, before touching it.
  Also open and small:
  fl-btst4df (the heap ring test's miri-only warnings) and fl-jjy5xt9 (the
  heap word's
  integer-to-pointer cast under miri, an owner question). Pending for the
  owner, not a task here:
  the three dependabot PRs (#1 checkout 7.0.1, #2 upload-artifact 7.0.2,
  #3 cache 6.1.0) still
  carry red `ci` runs from before the smoke fix and need a rebase onto
  main to re-run.
---

The ci miri job's suite step is throughput-bound on the runner's cores: identical test code took 38m55s (run 39471909369) and 49m26s (run 39463529492), and cutting the cap-edge cross by two thirds (fl-sq2sv0n, run 39483989145) left the step at 40m52s on a slower box. The step's wall is the suite's total work over the harness's thread count, and that count is unknown: the job never prints it (the repo is private, so the hosted runner may be the two-core size). First make the number known: print `nproc` in the miri job's install step, so every later log carries it. Then cut the next two heavy tests under miri with their own counts, as tests/simd.rs's `MULTISETS` does rather than `rounds()`: `exact_heap_ring_a_dot_product_of_mixed_pairs_matches_the_oracle_both_ways` runs 1000 pairs under miri at 110 s here (rounds(100_000)); `wide_folds_are_order_independent` 2000 draws at 80 s, where the "2000 draws with no special; reseed" assertion pins the count to the seed, so a smaller miri count needs its own seed check. Single-threaded here the suite is 1312 s (profile on fl-sq2sv0n); the step ends under 30 minutes on the runner only once the total is nearer 800 s. The other lever is the owner's: `timeout-minutes: 90` on the miri job costs billed Actions minutes on a private repo and nothing else, and a ceiling the suite can only just clear is a flaky job; ask before changing it.

## log
- 2026-10-20T20:16Z created
- 2026-10-20T20:17Z handoff by claude (30b8d01c-d5ec-456d-a066-16edcae1c161)
- 2026-10-20T20:18Z handoff by claude (30b8d01c-d5ec-456d-a066-16edcae1c161)
