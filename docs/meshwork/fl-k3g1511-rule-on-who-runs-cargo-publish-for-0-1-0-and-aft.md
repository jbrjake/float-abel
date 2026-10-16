---
id: fl-k3g1511
title: "Rule on who runs cargo publish for 0.1.0 and after — CI on the owner's signed tag through crates.io Trusted Publishing, or the owner's machine with CI packaging and attesting the .crate"
status: open
category: rulings
labels: [reproducible-floats]
discovered-from: fl-fm78m8d
relates: [fl-fm78m8d]
docs:
  - docs/DESIGN.md#release-sp-release
  - CLAUDE.md#standing-decisions
seq: 165
created: 2026-10-14T21:28Z
verify: contains docs/meshwork/fl-k3g1511-rule-on-who-runs-cargo-publish-for-0-1-0-and-aft.md /2026-[0-9][0-9]-[0-9][0-9] owner ruled/
handoff: |
  Next session: this task is the owner's ruling, not a build. The body
  holds the three options and
  the recommendation ((b) for 0.1.0, (a) from 0.1.1 on); read them back to
  the owner in session,
  record the answer in this task file verbatim with the date in the form
  the verify wants
  (`2026-MM-DD owner ruled ...`), and close. If the owner is not in the
  session, leave it and build
  behind it; the ruling counts only from the transcript.
  
  What stands after the weight-traits session (fl-pwq0h8n, closed
  2026-10-16 at ac9c361): `Wide`
  and `Fixed<LSB, LIMBS>` carry differential-dataflow's weight traits
  (src/wide/differential.rs,
  src/fixed/differential.rs: `IsZero`, `Semigroup`, `Monoid`, `Abelian`,
  `Multiply<i64>` and
  `Multiply<isize>` on either side, `From<i8>`, with `Fixed`'s `From<i8>`
  pinned by the
  `HoldsIntegers` const assertion in src/fixed/ops.rs and
  tests/ui/fixed_from_i8_above_one.rs
  blessed on 1.99.0) and DBSP's (src/wide/dbsp.rs, src/fixed/dbsp.rs:
  `HasZero`, `MulByRef<i64>`
  and `<isize>`, `SizeOf`, `IsNone`, `SupportsRoaring`; the archived form
  is the word itself through
  rkyv's derive with `#[archive(as = "...")]`, `Fixed`'s `limbs` field
  carrying `#[omit_bounds]`;
  `Deserialize` and `OrdRepr` hand-written as a copy and `Ord`'s `cmp`).
  Neither word has
  `Multiply<Self>`, `Mul<Self>` or `HasOne`: no carrier for the product of
  two words, so a `join` of
  two collections weighted by one word and dbsp's
  `neg`/`minus`/`differentiate` on a stream they
  weight do not compile, and the docs say so (DESIGN.md § Wide, § Fixed,
  the trilemma,
  § differential-dataflow, § DBSP; README § The trilemma and § In an
  engine; the crate docs). The
  operators moved to src/wide/ops.rs and src/fixed/ops.rs, with
  by-reference forms for dbsp's
  blankets. Tests: tests/differential_words.rs (13,
  `differential_wide_*`/`differential_fixed_*`),
  tests/dbsp_words.rs (12, `dbsp_wide_*`/`dbsp_fixed_*`, all 22 `dbsp_`
  green on
  `cargo +1.99.0 test --features dbsp --test main dbsp_`), by-reference
  tests in tests/wide_group.rs
  and tests/fixed.rs. BENCH.md § The end-to-end SUM has the `Wide` row at
  440ac8c: 4.4–5.1× the
  `Exact` diff, 9.5–13.7× `isize`, 179/244 ns an update
  typed/serialized at one worker, 13.64 GB
  peak RSS on the one-worker serialized row. clippy is now installed on
  the 1.99.0 toolchain, so
  `cargo +1.99.0 clippy --features dbsp --all-targets -- -D warnings`
  lints the feature; it is
  clean. The push gate was not run this session; the commit gate ran on
  every commit. Main is
  unpushed by the owner's word (fl-4fcbtkx).
  
  Two things the owner said this session, recorded on fl-p4dc5w5 and in
  memory: a bench sitting
  does not wait for a load-average threshold on the dev box ("dude there
  is nothing else running
  WHY DO YOU INSIST ON SO LITTLE LOAD TO RUN YOUR TESTS THIS IS MY DEV
  BOX????"); check `ps` once
  for competing work and run, and judge by the `isize` canaries
  afterwards.
  
  Build work behind this ruling, in the order `ready` lists it: fl-jqbhm8j
  (the per-machine
  bench-baseline.json), fl-p4dc5w5 (the heap arm's budget A/B against the
  saved Copy-word control
  binary,
  `~/.cache/cargo-target/float-abel-control/release/deps/dd_sum-ab4b21425246350c`,
  with the
  harness-shape caveat in its comments: compare binaries of one harness
  shape, so build the variant
  from a worktree with the current lib and 03d96e9's benches/dd_sum.rs,
  and run both at once),
  fl-dchdzd4 (the limb cap's edge: `big::add` and `big::mul` in
  src/exact/big.rs refuse one bit
  past what a result needs, so a decoded at-cap value cannot be subtracted
  from itself; the body
  names the fix and the `exact_cap_closed_*` tests). fl-qv0vh39 waits on a
  machine with AVX-512.
---

Owner's call, surfaced from "Prepare the 0.1.0 release" (fl-fm78m8d). Two standing texts read two
ways. DESIGN.md § Release: "CI builds it with `--locked` from a signed tag, with provenance, and
`cargo-semver-checks` is armed from then on. Publishing is the owner's act". CLAUDE.md § Standing
decisions: "`cargo publish` runs only with `--dry-run`. A publish, the crates.io name reservation
included, is the owner's act." Whether the owner's act is the signed tag push that a workflow then
publishes from, or the `cargo publish` command itself on the owner's machine, decides the
workflow's shape: its secrets, its permissions, and what the first release can do at all.

Fixed either way: the name `float-abel` is unreserved on crates.io and reserving it is the owner's
act; crates.io's Trusted Publishing (OIDC from GitHub Actions, no long-lived token) is configured
per crate on crates.io after the crate exists, so it cannot carry the first publish; every `uses:`
is hash-pinned and `scripts/check-actions-pinned.sh` fails both gates otherwise.

Options:
(a) CI publishes. A `release.yml` on tag push `v*`: verify the tag is signed by the owner's key
    (`git verify-tag` against a committed public key), `cargo package --locked`, `cargo publish
    --locked` through Trusted Publishing (`id-token: write`, no secret), attest the `.crate` with
    `actions/attest-build-provenance` and attach it to a GitHub release. The owner's act is the
    signed tag. 0.1.0 is the exception: it runs on the owner's machine with a token, because
    Trusted Publishing needs the crate to exist first, and the workflow carries every release after.
(b) The owner publishes, always. The workflow on tag push verifies the signature, packages with
    `--locked`, dry-runs `cargo publish`, attests the `.crate` and attaches it to a GitHub release,
    and the owner runs `cargo publish --locked` from a clean checkout of the tag. No secret, no
    OIDC, and CLAUDE.md's line holds for CI as it does for sessions.
(c) (a), triggered by `workflow_dispatch` on the tag rather than by the tag push, so a signed tag
    alone never publishes.

Recommendation: (b) for 0.1.0, because the first publish cannot come from CI under any option and
the crate's posture keeps `cargo publish` out of every automated path; (a) from 0.1.1 on, filed
when the owner wants it, because Trusted Publishing then takes the token off the owner's machine.
The owner writes the line this task's verify looks for, dated, with the option.

## log
- 2026-10-14T21:28Z created
- 2026-10-16T18:11Z handoff by claude (cd1130c2-56bf-432b-bcb1-8ed1c474e8fa)
