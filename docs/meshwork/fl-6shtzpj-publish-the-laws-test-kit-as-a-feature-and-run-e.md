---
id: fl-6shtzpj
title: "Publish the laws test kit as a feature and run every exact tier through it, with a naive f64 control that must fail"
category: crate/laws
labels: [reproducible-floats, rf-critical-path]
docs: [docs/DESIGN.md#the-laws-kit-sp-laws-kit, docs/DESIGN.md#what-an-incremental-engine-asks-of-a-weight-sp-laws]
needs: [fl-n7beqqc, fl-at6nh41, fl-atkbp6q]
seq: 80
verify: "all(exists src/laws.rs, run cargo test package=float-abel laws_)"
status: open
created: 2026-10-11T15:02Z
covers:
  - ref: docs/DESIGN.md#sp-laws-kit
    sha: 2246c38eec815c9342db2654d61ce495f9e6e64acbc783b5ba59d0d91427cb28
  - ref: docs/DESIGN.md#sp-laws
    sha: d04a257c60cce5cd8d14fd90d26d72979fabd4bf84b6d91f25a22d89077bc56f
handoff: |
  The tree is green through `./scripts/regression.sh` (exit 0: 31 lib and
  77 `tests/main.rs` tests
  with the million-multiset run under `--include-ignored`, the release
  leg, `tests/noalloc` at 5, the
  four trybuild cases, workspace clippy, rustdoc, the MSRV 1.74 build,
  cargo-deny). cargo-deny fetches
  from github.com, which the Bash sandbox blocks, so run it and the gate
  with the sandbox off.
  
  Since the last handoff the wire format closed (fl-a2wrx7w). `to_bytes`,
  `from_bytes`, `BYTES` and
  `WIRE_VERSION` sit on all three types; `WireError` is exported from
  `src/wire.rs`, which also holds
  the `Wide` and `Fixed` codecs and the byte helpers; `src/exact/wire.rs`
  holds `Exact`'s validating
  decoder; `src/wire/serde_impls.rs` the serde impls, a fixed-length byte
  tuple so a format that
  writes tuple elements back to back emits `to_bytes` itself.
  `tests/wire.rs` proves that with
  postcard 1.1.3, a dev-dependency chosen because bincode 1.3.3 carries
  RUSTSEC-2025-0141
  (unmaintained) and the owner ruled unmaintained code out; deny.toml
  ignores nothing. Timely 0.30,
  which differential-dataflow 0.24 requires, depends on bincode 1.3
  itself, so the `differential`
  feature will bring that advisory back for the owner to rule on; a
  comment on the adapter task says
  so. `FORMAT.md` is at the root with vectors `tests/wire.rs` parses and
  asserts. The trybuild harness
  is now `tests/ui.rs`, one `TestCases` for every `tests/ui/*.rs`.
  
  This task is next. Nothing of the kit exists: `src/laws.rs` is absent
  (the verify needs it), the
  `laws` feature is `laws = []` in Cargo.toml, and proptest is not in
  Cargo.lock. `std` is the
  feature that "adds the test kit" per § Crate posture, so wire `laws =
  ["dep:proptest", "std"]` and add
  `proptest = { version = "1", optional = true }`; take it with
  `default-features = false,
  features = ["std"]` to keep `rusty-fork`, `tempfile` and `wait-timeout`
  out of the tree, because
  deny.toml denies duplicate majors and allows only MIT and Apache-2.0,
  and every new licence in
  proptest's closure (`rustix`'s LLVM exception, `unicode-ident`'s
  Unicode-3.0, if they come in) has to
  be added there with the crate that drags it in. `tests/posture.rs`
  already asserts `laws` is on
  under a bare `cargo test`, so the kit's own tests go in `tests/laws.rs`
  as `mod laws;` of
  `tests/main.rs`, named `laws_*`, and scripts/check-test-topics.sh fails
  if the `mod` line is missing.
  
  What § The laws kit asks for: `check_abelian_group`, `check_ring`,
  `check_canonical` and
  `check_consolidation_invariance`, generic over a proptest `Strategy` for
  the weight, with
  associativity and commutativity bit-exact under 1,000 shuffles of a
  fold, identity, `a + (−a) == 0`
  and `a + b − b == a`, structural `Eq` ⇔ numeric equality with `Hash`
  consistent, distributivity of
  the multiply, and the same multiset summed in random partitions with
  `is_zero` pruning between them
  landing the same bits; § Verification says 256 proptest cases with a
  fixed seed in CI, so take a
  `Config` or a seed rather than proptest's default runner. Suggestions,
  unratified: have the entry
  points return proptest's `Result<(), TestError<T>>` from a
  `TestRunner::run` instead of panicking,
  so the negative-control tests assert `is_err()` without `catch_unwind`,
  and the doc example for a
  stranger ends in `.unwrap()`. `check_ring` needs a second strategy for
  the multiplier, because
  `Wide` and `Fixed` have `Mul<i64>` only while `Exact` has `Mul<Exact>`
  as well; `(a + b) · k ==
  a·k + b·k` is the law differential's join relies on either way. The
  `f64` control is a `Naive(f64)`
  newtype whose `Eq` and `Hash` go through `to_bits`; same-scale inputs
  such as 0.1, 0.2 and 0.3
  already break associativity and `a + b − b == a`, and the test must
  assert the kit reports the
  failure, since a control that passes means the kit is blind. Strategies
  per tier: `Fixed256` wants
  doubles on its grid (leading bit in −44..95, as tests/support.rs's
  `Rng::on_grid` draws them);
  `Wide` takes any double, specials included; `Exact` stays inside its
  width with same-scale doubles
  or a few decades of exponent, and its `product` is the ring's second
  input.
  
  The module needs rustdoc with an example a stranger can run against
  their own type; doctests
  compile with the lib's resolved features, so it builds under a bare
  `cargo test`. Both covered
  clauses (`#sp-laws-kit`, `#sp-laws`) are pinned on this task; `cover
  --repin` after any rewording.
  
  After it: fl-v7t6xhz (Binned), then the adapters. The portfolio ask
  fl-cfx001b is still open.
  `CARGO_TARGET_DIR` is set outside ~/Documents by the repo's
  `.claude/settings.local.json`.
relates: [fl-cfx001b, fl-v7t6xhz]
---
Per DESIGN.md § The laws kit: `float_abel::laws::{check_abelian_group, check_ring, check_canonical,
check_consolidation_invariance}`, generic over a `Strategy`. Its docs show a stranger running it
against their own type.

`laws_*` tests run the kit against `Fixed256`, `Wide` and `Exact`. They also run it against a naive
`f64` wrapper as the negative control, which must **fail** associativity and inverse.

## log
- 2026-10-11T15:02Z created
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-laws-kit @2246c38eec81
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-laws @d04a257c60cc
- 2026-10-12T13:23Z handoff by claude (f24d940b-27ff-4906-b27c-e6a7b31fddaf)
