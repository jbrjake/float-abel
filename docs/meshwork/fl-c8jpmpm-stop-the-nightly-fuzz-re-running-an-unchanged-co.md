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
status: doing
created: 2026-11-02T15:14Z
handoff: |
  State at 2026-11-23, dev at c9fed1e plus this store commit; both inbound
  asks (sazed's Wide::deposit, leras's Exact → Fixed) are answered and
  the inbox is empty.
  
  This task, as ci.yml stands today: the `fuzz` matrix job installs
  cargo-fuzz with `cargo install cargo-fuzz@0.13.2 --locked` on every run
  and caches only the registry and `fuzz/target` under `fuzz-${{
  hashFiles('fuzz/Cargo.lock') }}`. Neither `fuzz/corpus` nor `cache-hit`
  appears in the file. What remains is the body's two changes in that one
  job: (1) cache `fuzz/corpus/${{ matrix.target }}` with a per-run key and
  a restore-key prefix so each run resumes the last corpus; (2) cache
  `~/.cargo/bin/cargo-fuzz` keyed by its version and skip the install on
  `steps.<id>.outputs.cache-hit == 'true'`. Every `uses:` names a commit
  hash with the tag in a trailing comment
  (`scripts/check-actions-pinned.sh` fails both gates otherwise);
  actions/cache is already pinned in the file, so copy that line. The
  earlier design of a per-sha marker cache is dead: the `changes` job
  gates the nightly.
  
  A workflow edit cannot be watched working from here: a push runs no CI.
  `gh workflow run ci.yml --ref dev` runs the suite on demand; say what
  was and was not observed.
  
  Open beside it, for the owner and not to be built on: fl-ng7bs72 (the
  SIMD kill condition). Before asking, retake `cargo bench --bench simd`
  on the Mac with nothing else running: today's NEON figures are medians
  on a host shared with another session's tests, and `Wide`'s 2.7× and
  `Fixed256`'s 3.1× sit close enough to the 3× line that a clean sitting
  may move them. `bench-baseline.json` was not re-recorded for the faster
  one-value deposit; `scripts/bench-baseline.sh write` wants a full
  sitting of every bench on a quiet host.
claimed-by: claude (4aebe618-581f-48ef-9ee8-943721730ef9)
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

## comments
- 2026-11-02T18:34Z [claude (227b40e5-20bd-44b0-9b79-d44c1a3c3587)] 2026-11-02, fl-ezhk4cz at 4bec2d5: the first of the three defects is closed by a different mechanism than the handoff's. fuzz.yml is gone; the fuzz targets are the `fuzz` matrix job in ci.yml, and the whole suite is gated by ci.yml's `changes` job, which on the nightly asks `gh run list` for the last completed scheduled run's commit and skips every job when `git diff --quiet <last> <now> -- . ':!docs/meshwork'` is quiet. No marker cache, no seven-day eviction, no per-sha key: an unchanged dev is not fuzzed again until it moves. The push-triggered build-only mode is gone with the push trigger; the one step is `cargo fuzz run`, ten minutes, on nightly, the PR into main and dispatch. What remains is defects 2 and 3 as the body states them, in ci.yml's fuzz job: add `fuzz/corpus/${{ matrix.target }}` to a cache with a per-run key and a restore-key prefix so each run resumes the last, and cache `~/.cargo/bin/cargo-fuzz` keyed by version with `if: steps.<id>.outputs.cache-hit != 'true'` on the install line. Verify repointed: `contains ci.yml fuzz/corpus` and `contains ci.yml cache-hit`, neither present today. The handoff's file path and its marker design are stale; its corpus and cargo-fuzz paragraphs stand.
- 2026-11-24T23:46Z [claude (4aebe618-581f-48ef-9ee8-943721730ef9)] 2026-11-24: re-scoped to the corpus, the second defect, after measuring both on the nightly of 2026-11-24 (run 45407465768 at c8093d3, ubuntu-latest, `fuzz sum_vs_oracle`, job 138492077833). The corpus is worth carrying: the run started on `0 files found` and reached 1777 of its final 1779 edges and 7129 of its final 7975 features in the first minute, then added 6 to 29 new inputs a minute for nine more, so nine tenths of a night's features are the last night's found again. The cargo-fuzz install is not worth caching: 16.34 s of a 653 s leg that ends 29 minutes before the miri job beside it, on a repository that is now public and bills nothing; it is filed apart with its numbers. Title and verify repointed: `cache-hit` is out, `cmin` and DESIGN.md's `minimised` are in, none of the three present at c8093d3. A third finding from the same logs, the build cache a nightly toolchain makes useless, is filed apart too.
