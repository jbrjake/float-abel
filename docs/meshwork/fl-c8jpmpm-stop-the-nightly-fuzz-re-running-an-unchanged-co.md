---
id: fl-c8jpmpm
title: "Stop the nightly fuzz re-running an unchanged commit, carry the corpus between runs, and stop compiling cargo-fuzz every job"
category: crate/hardening
labels: [reproducible-floats]
seq: 20
parent: fl-b9qtzty
docs:
  - docs/DESIGN.md#hardening-sp-hardening
verify: "all(contains .github/workflows/fuzz.yml lookup-only, contains .github/workflows/fuzz.yml fuzz/corpus)"
status: open
created: 2026-11-02T15:14Z
handoff: |
  Next session: this is the first leaf of fl-b9qtzty after the advisories
  (fl-r0s86hg), on the owner's
  2026-11-02 word: "MAYBE WE SHOULDN'T BE RUNNING HOURS OF FUZZING WHEN
  THE CODE HASN'T CHANGED"; and
  "only work one or two of the task tree leafs and then pause for a new
  session", so take this one,
  then fl-cqmcs7c or fl-5mywkqp, and stop.
  
  One file, .github/workflows/fuzz.yml, three changes, all with the
  already-pinned
  actions/cache@0057852bfaa89a56745cba8c7296529d2fc39830 (v4.3.0 has
  `lookup-only`):
  
  1. Marker. After checkout, a step `id: marker` with `path: .fuzz-done`,
  `key: fuzz-done-${{
  matrix.target }}-${{ github.sha }}`, `lookup-only: true` (lookup-only
  skips the download, not the
  post-step save). The ten-minute run step's `if` becomes `(schedule ||
  workflow_dispatch) &&
  steps.marker.outputs.cache-hit != 'true'`, and a step after it, same
  condition, does `touch
  .fuzz-done`, which the post step then saves under the key. The push and
  pull_request builds are
  untouched: they are the two minutes that prove the targets compile. A
  scheduled run on a sha
  already fuzzed builds (cached) and skips the run: about two billed
  minutes a target instead of
  twelve. Seven idle days evict the marker and the tree is fuzzed again
  with a new seed, which is
  the one re-run of unchanged code worth paying for.
  2. Corpus. A second cache step, only on schedule/dispatch, `path:
  fuzz/corpus/${{ matrix.target }}`,
  `key: fuzz-corpus-${{ matrix.target }}-${{ github.run_id }}`,
  `restore-keys:
  fuzz-corpus-${{ matrix.target }}-`: a fresh key every run so the post
  step saves the grown corpus
  (cache keys are immutable; the restore-key prefix is what carries it
  forward). fuzz/corpus is
  gitignored locally, so nothing is committed. Check `fuzz/.gitignore` and
  the run's `cargo fuzz
  run` default corpus path (`fuzz/corpus/<target>/`, per the memory of the
  2026-10-13 session).
  3. cargo-fuzz. Cache `~/.cargo/bin/cargo-fuzz` keyed
  `cargo-fuzz-0.13.2-${{ runner.os }}` and put
  `if: steps.<id>.outputs.cache-hit != 'true'` on the `cargo install`
  line; the rustup line stays.
  
  Verify is `all(contains fuzz.yml lookup-only, contains fuzz.yml
  fuzz/corpus)`. Proof is two
  `workflow_dispatch` runs at one sha: the first runs ten minutes and
  saves markers (step
  conclusions in `gh run view --json jobs`), the second shows the run step
  `skipped` on all three
  targets. Each dispatch is a push-free run, so no gate; `gh workflow run
  fuzz.yml` starts one. The
  scripts/check-actions-pinned.sh gate sees the file at commit; no new
  action is introduced.
  
  What is proven already this session: the push gate and ci's deny jobs
  run `-D
  advisory-not-detected`; main at c885300 (or later) carries the lock with
  ctutils 0.4.7 and
  actix-http 3.20.0 and the single -0403 ignore. If the ci run on that
  push is not green when you
  start, that comes first (`gh run list --branch main --limit 3`).
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
