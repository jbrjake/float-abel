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
parent: fl-b9qtzty
---

The ci miri job's suite step is throughput-bound on the runner's cores: identical test code took 38m55s (run 39471909369) and 49m26s (run 39463529492), and cutting the cap-edge cross by two thirds (fl-sq2sv0n, run 39483989145) left the step at 40m52s on a slower box. The step's wall is the suite's total work over the harness's thread count, and that count is unknown: the job never prints it (the repo is private, so the hosted runner may be the two-core size). First make the number known: print `nproc` in the miri job's install step, so every later log carries it. Then cut the next two heavy tests under miri with their own counts, as tests/simd.rs's `MULTISETS` does rather than `rounds()`: `exact_heap_ring_a_dot_product_of_mixed_pairs_matches_the_oracle_both_ways` runs 1000 pairs under miri at 110 s here (rounds(100_000)); `wide_folds_are_order_independent` 2000 draws at 80 s, where the "2000 draws with no special; reseed" assertion pins the count to the seed, so a smaller miri count needs its own seed check. Single-threaded here the suite is 1312 s (profile on fl-sq2sv0n); the step ends under 30 minutes on the runner only once the total is nearer 800 s. The other lever is the owner's: `timeout-minutes: 90` on the miri job costs billed Actions minutes on a private repo and nothing else, and a ceiling the suite can only just clear is a flaky job; ask before changing it.

## log
- 2026-10-20T20:16Z created
- 2026-10-20T20:17Z handoff by claude (30b8d01c-d5ec-456d-a066-16edcae1c161)
- 2026-10-20T20:18Z handoff by claude (30b8d01c-d5ec-456d-a066-16edcae1c161)

## comments
- 2026-11-02T15:51Z [claude (c90f1eae-8c6b-41ec-aa59-9d0990f41bc2)] 2026-11-02 data point: ci run 41472015430 on main@682db04 (lockfile changed, so every cache was cold) ran the miri job in 21m14s, 15:29:15Z → 15:50:29Z, against the 38m55s and 49m26s this task records for identical test code on 2026-10-20. The suite under miri did not change between those runs (only Cargo.lock, deny-dbsp.toml, two workflows' deny arguments and docs did), so the spread is the runner, not the tests, which is this task's thesis: the thread count has to be read off the job before any per-test cut is judged.
- 2026-11-25T00:18Z [claude (4aebe618-581f-48ef-9ee8-943721730ef9)] 2026-11-24, fl-9rczhm8 at e2caba7: the bitrep test in tests/wide.rs now folds an `Exact` in its heap form too, and under miri here it takes 24 s of wall against 5 s before (three rounds, aarch64 macOS, nightly 11177f223). The suite step on the runner was 39 minutes of the job's 60 on 2026-11-24 (run 45407465768) before this.
- 2026-11-25T00:43Z [claude (4aebe618-581f-48ef-9ee8-943721730ef9)] 2026-11-25, the brief as it stands, in place of the handoff this task carried since 2026-10-20.
  
  Step one is in at 99b943b: the miri job's install step runs `nproc` after the version line. Not seen on a runner yet; the first suite run after 99b943b prints it. The repository is public since 2026-11-02, so the two-core guess of October may no longer hold.
  
  The job's times since: 2369 s on the nightly of 2026-11-24 (run 45407465768) and 2454 s on the dispatch of the same day (run 45512975046), where `cargo miri test --locked --lib` took 96 s, the noalloc crate 3 s and `--test main` 2324 s, 39 minutes of the job's 60. fl-9rczhm8 added some to the suite step after both: the bitrep test went from 5 s to 24 s of wall under miri on aarch64 macOS.
  
  Step two, unchanged from the earlier brief: two cuts, each a miri-specific count the way tests/simd.rs does it (`const MULTISETS: u64 = if cfg!(miri) { 20 } else { 10_000 }`), not `rounds()`. In tests/exact_heap_ring.rs, `exact_heap_ring_a_dot_product_of_mixed_pairs_matches_the_oracle_both_ways` has `let n = rounds(100_000)`, 1000 pairs under miri at 110 s on this machine in October; try 100. In tests/wide_group.rs, `wide_folds_are_order_independent` draws `(0..2_000)` and asserts a special was drawn, so a smaller miri count needs the seed checked by running the test under miri after the edit. Measure each with `cargo +nightly miri test --test main <name>` between two `date +%s` (the harness's "finished in" is miri's virtual clock), then read the step's wall from `gh run view <run> --json jobs` on the next suite run.
  
  The ceiling is the owner's: `timeout-minutes` on the miri job is not touched without asking, with the measured step times.
