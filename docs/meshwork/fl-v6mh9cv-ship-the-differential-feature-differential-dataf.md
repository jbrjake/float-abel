---
id: fl-v6mh9cv
title: "Ship the differential feature — differential-dataflow 0.24 trait impls, the three-worker integration test and the explode-and-count example"
category: crate/adapters
labels: [reproducible-floats, rf-critical-path]
docs: [docs/DESIGN.md#differential-dataflow-sp-differential]
needs: [fl-a2wrx7w]
seq: 90
verify: run cargo test package=float-abel target=main differential_
status: open
created: 2026-10-11T15:02Z
covers:
  - ref: docs/DESIGN.md#sp-differential
    sha: 08fc72836ea13c6f93a6666e076c1960cd90057ddd07f23b369fb245d847a74a
handoff: |
  The tree is green through `./scripts/regression.sh` at ece665a (exit 0:
  31 lib tests, 87 in
  `tests/main.rs` with `--include-ignored`, the doctest, the release leg,
  `tests/noalloc`, the four
  trybuild cases, workspace clippy, rustdoc, the MSRV 1.74.0 build of the
  default and of no features,
  cargo-deny clean). cargo-deny fetches from github.com, which the Bash
  sandbox blocks, so the gate
  runs with the sandbox off; crates.io resolves inside it.
  
  Since the last handoff the laws kit task closed ("Publish the laws test
  kit as a feature and run
  every exact tier through it, with a naive f64 control that must fail").
  `src/laws.rs` holds `laws::Weight` (the
  operators by value, structural `Eq` and `Hash`, `zero`, `is_zero`;
  implemented for `Fixed`, `Wide`
  and `Exact`), `laws::runner(cases, seed)`, `Case`/`RingCase`, and the
  four checks, each taking a
  proptest `TestRunner` and returning proptest's minimal failing case
  under a reason naming every law
  broken; `src/laws/report.rs` holds the seeded shuffler, the pruning
  merge tree and the report.
  `tests/laws.rs` runs 256 fixed-seed cases per check per tier and asserts
  the `Naive(f64)` control's
  failures by law name. `Weight` is the kit's own trait, not an engine's:
  this task implements
  differential's traits on the tiers, and the kit stays a test of the laws
  those traits assume.
  
  This task is next. Two facts to carry in:
  
  1. Supply chain, ruling pending. differential-dataflow 0.24.0 needs
  timely 0.30.0, which depends on
  bincode 1.3.3 (RUSTSEC-2025-0141, unmaintained). deny.toml ignores
  nothing, and the owner ruled,
  in the session that built the wire format, that unmaintained code is a
  code smell (the quote is
  on this task's comments). Adding the feature brings the advisory back,
  so the first move is to
  read back the options (ignore the advisory by id with the reason that it
  enters only through
  timely; or hold the feature) and get the owner's ruling before `cargo
  deny` is made to pass.
  This is owner-scoped: surface it, do not decide it.
  
  2. Lockfile editions, checked and clear. cargo 1.74 parses every locked
  manifest on a normal edge,
  optional features included, so an edition-2024 crate anywhere in a
  feature's closure fails the
  MSRV job even though that build never compiles it (that is why
  Cargo.lock holds proptest 1.11.0
  by `--precise`; the manifest comment says so). A scratch resolve of
  differential-dataflow 0.24
  today lands 28 packages with no edition-2024 crate, and both
  differential-dataflow and timely
  declare rust-version 1.86, which only matters for building the feature
  itself: stable here is
  1.99.0. Re-check with `cargo metadata` after adding the dependency,
  since the resolve may move.
  
  Wiring: `differential = ["dep:differential-dataflow", "serde"]`, since
  DESIGN.md § Features lists
  `serde` beside differential-dataflow 0.24 for this feature; the
  dev-dependency on the crate itself
  already lists `differential`, so a bare `cargo test` builds it, and
  `tests/posture.rs` already
  asserts the feature is on. The test topic is `tests/differential.rs` as
  `mod differential;` of
  `tests/main.rs`, tests named `differential_*` (the verify filters on
  that), on the three-worker
  `ProcessBinary(3)` the body names. Adding a
  dependency can change rustc's path trimming in the trybuild
  expectations: with proptest in the
  graph the name `Fixed` stopped being unique and the three
  `tests/ui/*.stderr` notes gained a
  `float_abel::` prefix, re-blessed with `TRYBUILD=overwrite` after
  reading the diff. Expect the same
  if differential's closure carries a `Wide` or an `Exact`, and bless only
  after the diff shows
  nothing but path spelling.
  
  DESIGN.md § differential-dataflow is the spec; `cover` pins it on this
  task, so `cover --repin`
  after any rewording. After it: fl-v7t6xhz (Binned), fl-06sggdc (dbsp,
  which the same lockfile
  constraint may bite, per its comment), fl-9kp61j0 (the golden hash). The
  portfolio ask fl-cfx001b
  is still open. `CARGO_TARGET_DIR` is set outside ~/Documents by the
  repo's
  `.claude/settings.local.json`.
---
The impls per DESIGN.md § differential-dataflow. In `tests/main.rs`, a `mod differential` behind the
feature: `ProcessBinary(3)`, every operator listed there, the `explode` + `count_total` idiom, an
`antijoin` with `isize`, and a retraction epoch whose netted output is empty. Plus
`examples/dd_sum.rs`.

## log
- 2026-10-11T15:02Z created
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-differential @08fc72836ea1
- 2026-10-12T13:59Z handoff by claude (51dfb626-8f8b-48f4-9e4a-68528d76dabd)
- 2026-10-12T14:00Z handoff by claude (51dfb626-8f8b-48f4-9e4a-68528d76dabd)

## comments
- 2026-10-12T13:23Z [claude (f24d940b-27ff-4906-b27c-e6a7b31fddaf)] Supply-chain fact to rule on before this feature lands: differential-dataflow 0.24.0 depends on timely ^0.30, and timely 0.30.0 lists bincode ^1.3 as a normal dependency (crates.io dependency lists read 2026-10-12). bincode 1.3.3 carries RUSTSEC-2025-0141 (unmaintained; its team calls 1.3.3 complete), and deny.toml runs with all-features, so cargo-deny will fail on this feature's closure. The owner ruled unmaintained code out when the wire task used bincode as a dev-dependency (fl-a2wrx7w's comments), so this is a conflict to surface, not an ignore to add: the choices are the owner's, an ignore scoped to timely's transitive use, waiting on upstream, or not shipping the feature at that pin.
- 2026-10-12T13:25Z [claude (f24d940b-27ff-4906-b27c-e6a7b31fddaf)] The owner's words, session f24d940b, on bincode 1.3.3 as a dev-dependency: "using unmaintained code is a code smell. seriously nothing we can use that doesn't have a security advisory warning against its use???" That is the ruling the comment above paraphrases; timely 0.30's own bincode ^1.3 dependency is the conflict it leaves for this task.
- 2026-10-12T13:55Z [claude (51dfb626-8f8b-48f4-9e4a-68528d76dabd)] Lockfile constraint found while wiring the laws feature, 2026-10-12: cargo 1.74 parses every manifest in Cargo.lock that sits on a normal edge, optional features included, so the MSRV job (cargo +1.74.0 build --locked on the default features) fails with 'this version of Cargo is older than the 2024 edition' as soon as any feature's dependency closure contains an edition-2024 crate, even though that build never compiles it. proptest 1.12's rand 0.10 closure did exactly that; the lock pins proptest 1.11.0. Before this feature lands, check differential-dataflow 0.24's and timely 0.30's closure editions at the versions the lock would take; if any is 2024, the MSRV job needs a different shape (a separate lockfile for the MSRV build, or the owner's ruling on raising the MSRV), and that is a conflict to surface, not a decision.
