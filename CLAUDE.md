# float-abel

A `no_std` Rust crate that makes `f64` sums an exact abelian group, so an incremental engine can add,
retract and re-merge floats and land the same bits under any order, worker count or architecture:
`Fixed`, `Wide` and `Exact` tiers, a batch-only `Binned`, a `laws` test kit, and differential and
DBSP adapters behind features. It is a sums crate, not an engine: ordering (keys, `MIN`/`MAX`) needs
no group and stays out, and so does `f32`.

## Commands

```bash
cargo test                                             # what every task verify runs; the self dev-dependency turns the test features on
cargo +1.99.0 test --features dbsp --test main dbsp_   # dbsp's toolchain; 1.97 and 1.98 ICE inside it
cargo +1.99.0 test --test main ui_                     # the compile-fail wording, blessed on stable 1.99 with rust-src; any other compiler runs the pass cases alone and says so
cargo +nightly miri test --test main                   # the suite under miri: sweeps at a hundredth, trybuild and timely ignored
cargo fuzz run sum_vs_oracle --target-dir "$CARGO_TARGET_DIR/fuzz" -- -max_total_time=60   # one target; CI runs each ten minutes nightly
./scripts/smoke.sh                                     # commit gate (pre-commit hook)
./scripts/regression.sh                                # push gate (pre-push hook)
git config core.hooksPath .githooks                    # run the gates from git
./scripts/bench-baseline.sh compare                    # after a cargo bench: every row past 1.5× of this machine's bench-baseline.json; `write` re-records it
git push origin dev                                    # where work lands; a push runs no ci (the push gate ran here); main takes only the dev→main PR
gh workflow run ci.yml --ref dev                       # the whole suite on demand; nightly it runs by itself, only when dev moved outside docs/meshwork/
./scripts/release-pr.sh                                # from this clone on dev at origin/dev, under your gh login: commits the version the log counts to and the entry onto dev, pushes, and opens or refreshes the one PR, dev→main; --dry-run prints it first
./scripts/version-from-log.sh                          # the version the log counts to at HEAD; prepare-release.sh writes it and the CHANGELOG entry
```

A `run` verify dies at 300 s, so warm `target/` before `meshwork close`. Keep `CARGO_TARGET_DIR`
outside `~/Documents`.

## Overrides

1. Hygiene, "never add license, author or copyright fields": `license = "MIT OR Apache-2.0"` in
   `Cargo.toml`, with `LICENSE-MIT` and `LICENSE-APACHE` at the root, because the crate publishes
   under that licence.
2. Rust, "`thiserror` in libraries": error types are hand-written, because the crate has zero
   required dependencies.
3. Rust, "`tests/<crate>/main.rs`": the one target is `tests/main.rs`, because task verifies name
   target `main`.

## Standing decisions

- Warnings fail the gate only: no `-D warnings` in `.cargo/config.toml`, whatever a task body says
  (owner: "make warnings only fail the gate").
- `#![deny(unsafe_code)]` crate-wide, lifted only in `src/simd/` and in `src/exact/heap.rs`, where
  the heap form owns its block (owner: "yes i'm okay with one more unsafe to save that many bits on
  every value"), each block with its `// SAFETY:` line; with `simd` and `alloc` off the crate is
  `unsafe`-free end to end.
- MSRV 1.74 on the default features, asserted in CI on exactly that set, because the crate is
  general-purpose, not one engine's.
- No floating-point instruction in any group op or read (only `to_bits`, shifts, carries), because
  that is what makes the bits architecture-independent.
- `Wide`'s and `Fixed`'s headroom budgets and `Wide`'s special counters are asserted in every build,
  never only under `debug_assertions`, because a wrapped sum is silently wrong (owner: "Yes, check
  in release"). The push gate runs those tests under `--release`.
- Every op canonicalises and every decoder refuses non-canonical bytes, because structural `Eq` must
  be numeric equality for differential's consolidation.
- `Wide`, `Fixed` and `Binned` are `Copy` plain words and never allocate (owner: "i cannot stress
  enough the paramount importance of zero-copy here"). `Exact` is a 32-byte word with a heap form
  behind `alloc`, reached by promotion past the 127-bit width and left by demotion after every op
  (owner: "yes let's do heap and allow product fols inside a dataflow"), in the niche layout that
  keeps the word at 32 bytes rather than a 40-byte enum; the compact form keeps `i32` counts, the
  heap form carries dyadic amounts, and the no-allocator build is the compact form with the width
  as its cap (owner: "Cap panic plus checked API").
- `From<f64>` is total on `Wide` and `Exact`: NaN and ±∞ are counted and projected, never refused.
  The only panic is the named `Exact` cap: the limb cap, `EXACT_MAX_LIMBS`, and the exponent bound
  at 2³⁰ (owner: "exponents need bounds if they can roll over"); without `alloc`, the width and the
  counts. The checked methods return the cap instead. `Mul<Exact>` stays, with dyadic special
  amounts (owner: "we absolutely need exact multiplication").
- The heap-armed `Exact`'s one-worker same-scale `dd_sum` rows stay within 1.12× the `Copy` word
  typed and 1.20× serialized, measured in one sitting against a control build; a row past that is a
  design question for the owner, not a build to ship.
- `Binned<K>` never implements an incremental trait, because it is not a group. A `trybuild` test
  pins that.
- `differential` adapts differential-dataflow 0.25 on a caret, never an exact pin, so a consumer's
  own 0.25.x unifies with it (owner: "0.25.1"). The closure's two unmaintained advisories, bincode
  through timely and paste through columnation, are ignored by id in `deny.toml` with the dragging
  crate named, never by `unmaintained = "workspace"`, so a third advisory still fails the gate
  (owner: "Ignore both by id").
- `dbsp` is on a caret, `"0.367"`, like differential's, so a consumer's own 0.367.x unifies with
  it, and its probe runs at the floor of the range (owner: chose "Caret: dbsp = "0.364"" over the
  exact pin, then "Bump dbsp to 0.367" past zip 6's advisories, the floor moving with the caret
  kept). Its closure is checked by `deny-dbsp.toml` in the dbsp CI job and the push gate
  while `deny.toml` checks every other feature, so the crate's own posture stays as strict as it
  is (owner: "follow your rec on dbsp's CI job"). An advisory on a crate dbsp itself pins with no
  in-range fix is ignored by id with dbsp named, and both gates run `check -D
  advisory-not-detected`, so the ignore fails the gate the day the crate moves on (owner: "we need
  something to auto-trigger to remove the exception when dbsp updates zstd").
- Speed targets are measured into `BENCH.md`, never gated, because a missed target is a design
  question, not a broken build.
- Releases are cut on the `dev`→`main` merge at the version counted from 0.0.0 over every commit
  since the first, breaking +major, `feat` +minor, `fix` +patch, by `scripts/version-from-log.sh`
  (owner: "YOU NEED TO BE COUNTING FROM 0.0.0 ON FROM _THE VERY FIRST COMMIT_"; "of course feats
  bump the minor"); the changelog entry is `scripts/changelog-entry.sh`'s, and the release workflow
  runs on git, cargo, gh and the two GitHub-owned actions alone (owner: "THIS IS A MINIMALIST
  PROJECT THAT IS SUPPOSED TO HAVE NO DEPENDENCIES IT DOESN'T NEED"). The release is one pull
  request, `dev`→`main`, cut from a clone by `scripts/release-pr.sh` under the developer's own gh
  login, never by a runner, because a commit and a pull request carry no identity a runner could add
  (owner: "why can't it be something done from a clone using gh?"; "i want it to be a single PR,
  dev to main."). Every GitHub Release carries
  the `.crate` with a GitHub artifact attestation in place of a signed tag, and a ruleset pins `v*`
  tags (owner chose "Attestation + tag ruleset"). crates.io is outside the release until the owner
  says (owner: "I WILL TELL YOU WHEN I WANT IT AND THEN YOU WILL ADD IT TO THE RELEASE PROCESS").
- A session runs `cargo publish` only with `--dry-run`. `publish.yml` publishes on the owner's
  `workflow_dispatch` alone, never on a tag push (owner: "make it a ci publish on my manual
  trigger"), and nothing in the release runs it; the first publish, the crates.io name reservation
  included, carries the owner's token, and every later one Trusted Publishing, because crates.io
  grants OIDC only to a crate that exists.
- Every `uses:` in a workflow names a commit hash, with the tag it stood for in a trailing comment,
  and `scripts/check-actions-pinned.sh` fails both gates on a tag or a branch, because a tag can be
  moved to code nobody reviewed (owner: "use git hashes to reference dependencies not git labels
  that can be manipulated"; "security is absolutely critical here").
- Dependabot refreshes those hashes by PR, one `github-actions` entry in `.github/dependabot.yml`
  on a weekly schedule, and nothing merges on its own (owner: "yes have dependabot open prs that
  update the pinned hashes"). The golden jobs' tarball digests are not actions and move by hand.
- `ci.yml` is the one test suite, run nightly on `dev` only when a commit since the last nightly
  reached anything outside `docs/meshwork/`, and again on the pull request into `main`; a push runs
  nothing (owner: "testing to occur nightly, ONLY when there have been code changes since the last
  run. a middle path. then, yes, again on the PR"; "A PUSH IS NOT SUPPOSED TO BE AN INTENSIVE
  TEST!"). On the pull request, miri and the fuzz targets run only when they have not run green
  since the code last changed, which `scripts/probes-due.sh` decides (owner: "FUZZ NEEDS TO RUN IF
  IT HASN'T YET SINCE THE CODE LAST CHANGED!"; "I DO NOT WANT CUTTING A PR TO TAKE AN HOUR OF MY
  TIME just to re-run fuzz tests that are SUPPOSED to run every night"). `release.yml` cuts the
  release and `publish.yml` publishes; there is no fourth.
- The golden `aarch64 macOS` job runs whenever the suite runs, whatever it bills, because the
  committed hash is blessed on that platform (owner: "keep the mac leg").

## Boundaries

- The crate names no consuming engine. An engine's adapter lives in that engine's repo, except the
  differential and DBSP adapters, which live here behind features.
- Do not file PRs or issues upstream.

## Pointers

- bitrep's `FORMAT.md` and `conformance/vectors.json`: the limb layout `Wide` shares, and its oracle.

<!-- baseline:begin -->
# Engineering baseline

## Evidence
Done means you watched it work, this session, on this tree: an exit 0, or `gh run view <id>` showing
`conclusion=success`. "Should pass", "likely fixed" and "CI in flight" are not reports. Say what you
didn't verify and whether you used `--no-verify`. A number carries what it was measured on (pin,
corpus, machine, arm; batch or incremental), and a doc's sentence about a result is not the result:
read the artifact.

## Owner intent
Quote rulings in the owner's words. What you derive from one is unratified until the owner confirms
it, so read back what you will and won't build before building. Dropping anything the design treats as
core is a question, not a decision. When the owner points at something as the model, its mechanism is
the spec.

## Work
The worklist is the repo's meshwork store, used as the meshwork skill says. File work the moment you
find it: a harness task list isn't in git and dies with the session.

## Tests
Red first: no production code without a failing test behind it. A test must be able to fail, so no
tautologies, no "didn't panic", no mocking away the layer under test, and assert counts before
equality. Check every fast path against a slow, obviously correct oracle. External input gets
adversarial fixtures. Reproduce a bug in a failing test before fixing it. A flaky test is a bug, and a
threshold is never lowered to pass.

## Gates
Commit gate (`scripts/smoke.sh`): under 60 s, scoped to the diff. Push gate (`scripts/regression.sh`):
deterministic correctness only (strict lint, build, all tests with `--include-ignored`). Benchmarks run
on demand or in CI, never in a git hook. Enable hooks with `git config core.hooksPath .githooks`.
Never re-run a gate on an unchanged tree. Gates fail closed and skip loudly.

## Performance
Criterion benches against a committed per-machine `bench-baseline.json`: median of 7+ runs after
warm-up, plus peak RSS. A regression past 1.5× needs the owner's approval. Run A/Bs in one sitting
against a control, and count rather than time where you can.

## Hygiene
Conventional Commits (`type(scope): subject`), atomic, with docs in the same commit. Commit lockfiles,
pin load-bearing deps `=x.y.z`, and keep generated data deterministic. Never add license, author or
copyright fields. Secrets come from the environment, and untrusted sizes get a ceiling. Edit files with
Edit/Write, never `sed -i` or heredocs. Cite code by symbol or `// ANCHOR: <name>`, never by line number.

## This file
Above this baseline, a repo's CLAUDE.md says only what the tree can't: what the repo is, its commands,
the owner's standing decisions (each with a one-clause why), its boundaries, pointers, and numbered
overrides of this baseline. Current state only: no dates, history or incident stories. Under 4 KB.

## Rust
Give a new workspace this setup with its first crate: `[profile.dev] debug = "line-tables-only"` and
`split-debuginfo = "unpacked"`, `[profile.dev.package."*"] debug = false`, one integration-test target
per crate (`tests/<crate>/main.rs`, with each file a `mod`), and `CARGO_TARGET_DIR` outside
`~/Documents`. Warnings fail the gate, not the build: the gate runs
`cargo clippy --all-targets -- -D warnings` and `RUSTDOCFLAGS="-D warnings" cargo doc`. Never export
`RUSTFLAGS` (that forks `target/`), and never put `-D warnings` in `.cargo/config.toml` or in source.
Lints go in `[workspace.lints]` (clippy `all` deny, `pedantic` warn), and `#[allow]` goes at the
narrowest scope with a reason. Every `unsafe` block states its invariant in `// SAFETY:`. Use
`thiserror` in libraries and `anyhow` at entry points, with no `.unwrap()` outside tests. Files target
500 lines, with a hard ceiling of 750.
<!-- baseline:end -->
