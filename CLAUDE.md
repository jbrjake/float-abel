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
./scripts/smoke.sh                                     # commit gate (pre-commit hook)
./scripts/regression.sh                                # push gate (pre-push hook)
git config core.hooksPath .githooks                    # run the gates from git
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
- `#![deny(unsafe_code)]` crate-wide, lifted only in `src/simd/`, so the crate with `simd` off is
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
- Every type is a `Copy` plain word and nothing allocates (owner: "i cannot stress enough the
  paramount importance of zero-copy here"; "in general i approve dropping the heap arm"). `Exact` is
  32 bytes with `i32` special counts, and there is no `alloc` feature.
- `From<f64>` is total on `Wide` and `Exact`: NaN and ±∞ are counted and projected, never refused.
  The only panic is the named `Exact` cap: the width, the exponent bound at 2³⁰ (owner: "exponents
  need bounds if they can roll over"), and the counts. The checked methods return the cap instead
  (owner: "Cap panic plus checked API"). `Mul<Exact>` stays, with integer special amounts (owner:
  "we absolutely need exact multiplication").
- `Binned<K>` never implements an incremental trait, because it is not a group. A `trybuild` test
  pins that.
- `differential` adapts differential-dataflow 0.25 on a caret, never an exact pin, so a consumer's
  own 0.25.x unifies with it (owner: "0.25.1"). The closure's two unmaintained advisories, bincode
  through timely and paste through columnation, are ignored by id in `deny.toml` with the dragging
  crate named, never by `unmaintained = "workspace"`, so a third advisory still fails the gate
  (owner: "Ignore both by id").
- `dbsp` is pinned at `=0.364.0`, the version its probe ran at, and its closure is checked by
  `deny-dbsp.toml` in the dbsp CI job and the push gate while `deny.toml` checks every other
  feature, so the crate's own posture stays as strict as it is (owner: "follow your rec on dbsp's
  CI job").
- Speed targets are measured into `BENCH.md`, never gated, because a missed target is a design
  question, not a broken build.
- `cargo publish` runs only with `--dry-run`. A publish, the crates.io name reservation included, is
  the owner's act.
- Every `uses:` in a workflow names a commit hash, with the tag it stood for in a trailing comment,
  and `scripts/check-actions-pinned.sh` fails both gates on a tag or a branch, because a tag can be
  moved to code nobody reviewed (owner: "use git hashes to reference dependencies not git labels
  that can be manipulated"; "security is absolutely critical here").

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
