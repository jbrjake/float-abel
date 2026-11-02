---
id: fl-c8jpmpm
title: Carry the fuzz corpus between runs and stop compiling cargo-fuzz every job
category: crate/hardening
labels: [reproducible-floats]
seq: 20
parent: fl-b9qtzty
docs:
  - docs/DESIGN.md#hardening-sp-hardening
verify: "all(contains .github/workflows/ci.yml fuzz/corpus, contains .github/workflows/ci.yml cache-hit)"
status: open
created: 2026-11-02T15:14Z
handoff: |
  Two changes in one job, `fuzz` in .github/workflows/ci.yml, both with
  the already-pinned
  actions/cache@0057852bfaa89a56745cba8c7296529d2fc39830 (v4.3.0). The
  suite's gating is done: the
  `changes` job skips every nightly on an unchanged dev, so no marker
  cache is wanted here.
  
  1. Corpus. A cache step `path: fuzz/corpus/${{ matrix.target }}`, `key:
  fuzz-corpus-${{
  matrix.target }}-${{ github.run_id }}`, `restore-keys: fuzz-corpus-${{
  matrix.target }}-`: a
  fresh key every run so the post step saves the grown corpus (cache keys
  are immutable; the
  restore-key prefix carries it forward). fuzz/.gitignore lists corpus, so
  nothing is committed;
  `cargo fuzz run` writes to fuzz/corpus/<target>/ by default.
  2. cargo-fuzz. A cache step `id: cargo-fuzz`, `path:
  ~/.cargo/bin/cargo-fuzz`, `key:
  cargo-fuzz-0.13.2-${{ runner.os }}`, and `if:
  steps.cargo-fuzz.outputs.cache-hit != 'true'` on
  the `cargo install cargo-fuzz@0.13.2 --locked` line; the rustup line
  stays. The install is one
  to two minutes a job today.
  
  Verify is `all(contains ci.yml fuzz/corpus, contains ci.yml cache-hit)`.
  Proof is two
  `gh workflow run ci.yml --ref dev` dispatches: the first saves
  `fuzz-corpus-<target>-<run_id>` and
  `cargo-fuzz-0.13.2-Linux` (the post-cache step's `Cache saved` lines in
  `gh run view --log`), the
  second logs `Cache restored from key: fuzz-corpus-<target>-` and the
  install step `skipped`. Each
  dispatch runs the whole suite, about two billed hours; the first is also
  fl-ezhk4cz's proof if it
  has not run yet. The check-actions-pinned gate sees the file at commit;
  no new action.
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

## comments
- 2026-11-02T18:34Z [claude (227b40e5-20bd-44b0-9b79-d44c1a3c3587)] 2026-11-02, fl-ezhk4cz at 4bec2d5: the first of the three defects is closed by a different mechanism than the handoff's. fuzz.yml is gone; the fuzz targets are the `fuzz` matrix job in ci.yml, and the whole suite is gated by ci.yml's `changes` job, which on the nightly asks `gh run list` for the last completed scheduled run's commit and skips every job when `git diff --quiet <last> <now> -- . ':!docs/meshwork'` is quiet. No marker cache, no seven-day eviction, no per-sha key: an unchanged dev is not fuzzed again until it moves. The push-triggered build-only mode is gone with the push trigger; the one step is `cargo fuzz run`, ten minutes, on nightly, the PR into main and dispatch. What remains is defects 2 and 3 as the body states them, in ci.yml's fuzz job: add `fuzz/corpus/${{ matrix.target }}` to a cache with a per-run key and a restore-key prefix so each run resumes the last, and cache `~/.cargo/bin/cargo-fuzz` keyed by version with `if: steps.<id>.outputs.cache-hit != 'true'` on the install line. Verify repointed: `contains ci.yml fuzz/corpus` and `contains ci.yml cache-hit`, neither present today. The handoff's file path and its marker design are stale; its corpus and cargo-fuzz paragraphs stand.
