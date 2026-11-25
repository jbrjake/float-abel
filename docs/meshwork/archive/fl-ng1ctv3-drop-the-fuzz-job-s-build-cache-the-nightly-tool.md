---
id: fl-ng1ctv3
title: "Drop the fuzz job's build cache: the nightly toolchain moves under it, so a restored fuzz/target is rebuilt whole"
category: crate/hardening
labels: [reproducible-floats]
parent: fl-b9qtzty
discovered-from: fl-c8jpmpm
docs:
  - docs/DESIGN.md#hardening-sp-hardening
verify: "lacks .github/workflows/ci.yml fuzz/target"
status: done
created: 2026-11-24T23:46Z
---
ci.yml's `fuzz` job caches `~/.cargo/registry/index`, `~/.cargo/registry/cache` and `fuzz/target`
under `fuzz-${{ hashFiles('fuzz/Cargo.lock') }}`, and installs whatever `nightly` is that day. The
key names the lockfile and not the compiler, so a hit restores another nightly's artifacts and
cargo rebuilds every crate; and a hit is never saved again, so the cache holds the first night's
compiler until the lockfile moves or GitHub evicts it.

Measured, the `fuzz sum_vs_oracle` leg on ubuntu-latest:

- 2026-11-14, job 132419583670, nightly 5f4effd57: `Cache restored from key: fuzz-d66e676f…`, then
  25 `Compiling` lines; the run step took 633 s for 601 s of fuzzing.
- 2026-11-24, job 138492077833, nightly 20fe935d5: `Cache not found for input keys: fuzz-d66e676f…,
  fuzz-` (evicted: the suite had skipped ten nights), then the same 25 `Compiling` lines,
  `Finished … in 19.23s`; the run step took 620 s.
- Across the eight suite runs since 2026-11-02 the run step is 618 to 650 s, so the build is 17
  to 49 s whether the cache was restored or not.

The cache is 58 MB, its restore 1 to 4 s, and three legs race to save one key (`Failed to save:
Unable to reserve cache with key fuzz-d66e676f…`). Remove the step: the job then reads crates.io
for the fuzz package's closure every run, as `cargo install cargo-fuzz` already does for its own
56 crates. Keying it by `rustc -V` instead would help only two runs on one nightly, a pull request
on the day of a night's run, and save that run 19 s of an eleven-minute job.

The miri job's cache has the same key shape (`miri-${{ hashFiles('Cargo.lock') }}` under a moving
nightly) and was not examined.

## log
- 2026-11-24T23:46Z created
- 2026-11-24T23:46Z open→doing — claimed by claude (4aebe618-581f-48ef-9ee8-943721730ef9)
- 2026-11-25T00:30Z doing→done — verify exit 0 @ 99b943b+3

## comments
- 2026-11-25T00:30Z [claude (4aebe618-581f-48ef-9ee8-943721730ef9)] 2026-11-25 at be97e42: the step is out of ci.yml's fuzz job, with a comment where it stood saying why there is none. Watched on run 45512975046 (dispatch on dev at 37e3431, ubuntu-latest, nightly of 2026-11-24), all three legs green with no build cache: the fuzz target built in 27.88 s (sum_vs_oracle), 19.16 s (decode_robustness) and 27.79 s (sub_roundtrip), the run steps took 630, 620 and 629 s for 601 s of fuzzing, and the jobs 669, 652 and 664 s. With the cache, across the eight suite runs since 2026-11-02, the run step was 618 to 650 s. The last `fuzz-d66e676f…` entry, 58 MB on refs/heads/dev, is read by nothing now and stays until GitHub evicts it. The miri job's cache was not examined.
