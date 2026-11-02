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
