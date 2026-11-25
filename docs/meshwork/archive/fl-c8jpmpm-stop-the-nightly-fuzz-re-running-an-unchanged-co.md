---
id: fl-c8jpmpm
title: Carry the fuzz corpus between runs
category: crate/hardening
labels: [reproducible-floats]
seq: 20
parent: fl-b9qtzty
docs:
  - docs/DESIGN.md#hardening-sp-hardening
verify: "all(contains .github/workflows/ci.yml fuzz/corpus, contains .github/workflows/ci.yml cmin, contains docs/DESIGN.md minimised)"
status: done
created: 2026-11-02T15:14Z
---
Owner, 2026-11-02: "MAYBE WE SHOULDN'T BE RUNNING HOURS OF FUZZING WHEN THE CODE HASN'T CHANGED".
fuzz.yml's `schedule` ran three ten-minute targets every night of October, 576 Linux minutes, and
main sat at f24b051 from the 20th to the 31st: twelve of those nights fuzzed the same tree. Three
defects in the workflow, one change each:

1. No memory of what was fuzzed. A marker cache per target, `fuzz-done-${{ matrix.target }}-${{
   github.sha }}`, read with `actions/cache` `lookup-only: true` as the first step after checkout;
   every later step carries `if: steps.marker.outputs.cache-hit != 'true'`, and a scheduled run that
   finishes green saves the marker. A hit costs one billed minute a job instead of eleven. Caches
   evict after seven days unused, so an untouched main is re-fuzzed weekly with a fresh seed, which
   is the one case where fuzzing unchanged code earns anything (libFuzzer's seed differs per run).
2. No corpus between runs. `fuzz/corpus` is not in the cache path, so every night starts from the
   seed inputs and rediscovers the same paths in its ten minutes. Add `fuzz/corpus/${{ matrix.target
   }}` to a cache keyed by target with a restore-key prefix, so each run resumes the last.
3. `cargo install cargo-fuzz@0.13.2 --locked` compiles cargo-fuzz from source in every job, one to
   two minutes of the push-triggered build-only runs' two. Cache `~/.cargo/bin/cargo-fuzz` keyed by
   the version, or install the prebuilt binary pinned by its release digest the way golden.yml
   installs wasmtime and cross.

The push and pull_request triggers stay: they build the targets so none rots, and they are the two
minutes, not the thirty.

## log
- 2026-11-02T15:14Z created
- 2026-11-02T15:26Z handoff by claude (c90f1eae-8c6b-41ec-aa59-9d0990f41bc2)
- 2026-11-02T18:36Z handoff by claude (227b40e5-20bd-44b0-9b79-d44c1a3c3587)
- 2026-11-23T16:38Z handoff by claude (088ad122-d06d-463d-8c3f-01d8f3ef0f7f)
- 2026-11-24T23:34Z open→doing — claimed by claude (4aebe618-581f-48ef-9ee8-943721730ef9)
- 2026-11-25T00:42Z doing→done — verify exit 0 @ 99b943b+6

## comments
- 2026-11-02T18:34Z [claude (227b40e5-20bd-44b0-9b79-d44c1a3c3587)] 2026-11-02, fl-ezhk4cz at 4bec2d5: the first of the three defects is closed by a different mechanism than the handoff's. fuzz.yml is gone; the fuzz targets are the `fuzz` matrix job in ci.yml, and the whole suite is gated by ci.yml's `changes` job, which on the nightly asks `gh run list` for the last completed scheduled run's commit and skips every job when `git diff --quiet <last> <now> -- . ':!docs/meshwork'` is quiet. No marker cache, no seven-day eviction, no per-sha key: an unchanged dev is not fuzzed again until it moves. The push-triggered build-only mode is gone with the push trigger; the one step is `cargo fuzz run`, ten minutes, on nightly, the PR into main and dispatch. What remains is defects 2 and 3 as the body states them, in ci.yml's fuzz job: add `fuzz/corpus/${{ matrix.target }}` to a cache with a per-run key and a restore-key prefix so each run resumes the last, and cache `~/.cargo/bin/cargo-fuzz` keyed by version with `if: steps.<id>.outputs.cache-hit != 'true'` on the install line. Verify repointed: `contains ci.yml fuzz/corpus` and `contains ci.yml cache-hit`, neither present today. The handoff's file path and its marker design are stale; its corpus and cargo-fuzz paragraphs stand.
- 2026-11-24T23:46Z [claude (4aebe618-581f-48ef-9ee8-943721730ef9)] 2026-11-24: re-scoped to the corpus, the second defect, after measuring both on the nightly of 2026-11-24 (run 45407465768 at c8093d3, ubuntu-latest, `fuzz sum_vs_oracle`, job 138492077833). The corpus is worth carrying: the run started on `0 files found` and reached 1777 of its final 1779 edges and 7129 of its final 7975 features in the first minute, then added 3 to 29 new inputs a minute for nine more, so nine tenths of a night's features are the last night's found again. The cargo-fuzz install is not worth caching: 16.34 s of a 653 s leg that ends 29 minutes before the miri job beside it, on a repository that is now public and bills nothing; it is filed apart with its numbers. Title and verify repointed: `cache-hit` is out, `cmin` and DESIGN.md's `minimised` are in, none of the three present at c8093d3. A third finding from the same logs, the build cache a nightly toolchain makes useless, is filed apart too.
- 2026-11-25T00:42Z [claude (4aebe618-581f-48ef-9ee8-943721730ef9)] 2026-11-25 at 57746b0, watched on run 45512975046 (dispatch on dev at 37e3431, ubuntu-latest, every job green), both ways.
  
  Empty, attempt 1, all three legs: `Cache not found for input keys: corpus-<target>-45512975046-1, corpus-<target>-`, `0 files found`, ten minutes, `cargo fuzz cmin`, `Cache saved with key: corpus-<target>-45512975046-1` on refs/heads/dev.
  
  - sum_vs_oracle (job 138865187410): ended cov 1778 ft 7849 corp 1250; cmin 1246 files to 961; 90,777 B saved.
  - decode_robustness (138865187423): cov 682 ft 999 corp 103; cmin 102 to 85; 6,267 B.
  - sub_roundtrip (138865187460): cov 923 ft 2126 corp 355; cmin 354 to 283; 26,712 B.
  
  Carried, attempt 2, sum_vs_oracle re-run alone (`gh run rerun 45512975046 --job 138865187410`, job 138884251648): `Cache hit for restore-key: corpus-sum_vs_oracle-45512975046-1`, `961 files found`, `seed corpus: files: 961`, and the first line of the run is `#962 INITED cov: 1773 ft: 7844 corp: 935`, where the empty run reached 7849 at its end. It ended cov 1779 ft 7890 corp 955 after 3,707,491 runs. The directory then held 1618 files for that corpus of 955, the growth the cmin step is there for; cmin wrote 946, and `Cache saved with key: corpus-sum_vs_oracle-45512975046-2`, 90,058 B. The cmin step took 0 to 1 s in all four jobs and rebuilt nothing (`Finished … in 0.01s`).
  
  Not observed: a restore across two runs (the re-run shares the run id, and its key differs by the attempt), a pull request into main reading dev's corpus, and a crash's effect on the save. The first suite run after this one shows the first: its fuzz legs should log `Cache hit for restore-key: corpus-<target>-45512975046-`.
  
  Correction: the commit message of 57746b0 says the 2026-11-24 nightly added "6 to 29" new inputs a minute after its first; the log's table says 3 to 29 (minute six had 3). The re-scope comment above is corrected; the commit is pushed and stays as it is.
