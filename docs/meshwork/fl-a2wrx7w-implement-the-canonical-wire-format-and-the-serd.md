---
id: fl-a2wrx7w
title: "Implement the canonical wire format and the serde feature, with decoders that refuse non-canonical input"
category: crate/wire
labels: [reproducible-floats, rf-critical-path]
docs: [docs/DESIGN.md#wire-format-sp-wire]
needs: [fl-atkbp6q]
seq: 70
verify: run cargo test package=float-abel wire_
status: open
created: 2026-10-11T15:02Z
covers:
  - ref: docs/DESIGN.md#sp-wire
    sha: 10a66b73f9bdbcdef33cbfaca33d2deeff1e32a52775100d4759890a318552f6
handoff: |
  The tree at fec0b3d is green through `./scripts/regression.sh` (exit 0:
  99 tests in the main
  package, 31 lib and 68 in `tests/main.rs`, the million-multiset rounding
  run included under
  `--include-ignored`, plus the release leg, `tests/noalloc`, workspace
  clippy, rustdoc, the MSRV 1.74
  build, cargo-deny). The gate's cargo-deny leg fetches the advisory
  database from github.com, which
  the Bash sandbox blocks, so run the gate with the sandbox off.
  
  Since the last handoff the rounding proof closed. `round::F32` sits
  beside `F64`; `to_f32` is on
  `Wide`, `Fixed` and `Exact` (the last through `Compact::read_f32`), each
  the tier's read path with
  `F32` and never via `f64`; and `ieee::project` returns the `Special` so
  a read takes its own
  format's constant with no conversion instruction.
  `scripts/gen_rounding_vectors.py` (python3
  3.12.13, run with `-I`) writes `tests/data/rounding_vectors.json`
  deterministically: `float()` for
  `f64`, a round-to-odd double packed with `struct` for `f32`, and an
  integer rounder that must agree
  with both; a divisor has to stay below 2^64 because serde_json's
  `as_u64` is how the tests read it.
  `tests/rounding.rs` injects each vector through `Wide::from_parts` and
  `Fixed256::from_limbs`, and
  into `Exact` through the conversion, scaled by an exact power of two
  when the vector lies off
  `Wide`'s grid; a `Held` enum puts the three tiers behind one surface.
  The million-multiset
  agreement is `#[ignore]` and takes 10 s in a debug build. trybuild's
  `.stderr` files are rustc
  1.99.0's wording; `TRYBUILD=overwrite cargo test convert_` rewrites them
  when stable's text moves.
  
  This task is next. What exists: all three types are `repr(C)` with sizes
  pinned by const
  assertions, `tests/layout.rs` already checks that `Wide`'s in-memory
  bytes are the limbs then the
  counters and `Fixed256`'s its limbs, and `Wide::from_parts` and
  `Fixed::from_limbs` are the total
  constructors. `Exact` has no public constructor from parts:
  `Exact::assemble(Compact, [i32; 3])`
  and `Compact::from_parts(lo, hi, exp)` are private, and `from_parts`
  only `debug_assert!`s
  canonical form, so `Exact::from_bytes` must validate before reaching it.
  Nothing of the wire
  exists: no `to_bytes`/`from_bytes`, no `FORMAT.md`, no serde impls. The
  `serde` dependency is
  `default-features = false` with no `derive`, so the impls are
  hand-written and `no_std`-clean:
  `serialize_bytes` of `to_bytes`, and a visitor that runs `from_bytes`,
  so decoding validates in one
  place. bincode is not in Cargo.lock; the fixed-int round-trip test needs
  `bincode = "=1.3.3"` as a
  dev-dependency.
  
  Canonical form the `Exact` decoder refuses, each as a typed error and
  never a panic: an even
  nonzero mantissa; a zero mantissa with a nonzero exponent; an exponent
  outside ±EXACT_MAX_EXP; a
  count equal to i32::MIN; a wrong version byte or length. The task body's
  "compact-representable
  value encoded as the heap tier" case no longer exists, because `Exact`
  is a 32-byte plain word with
  no heap arm (CLAUDE.md standing decision), so the non-canonical cases
  are the ones above.
  `Wide::from_bytes` and `Fixed::from_bytes` are total apart from the
  version byte and the length,
  because every bit pattern of theirs is a value. § Wire says each body
  is the type's `repr(C)` bytes
  on a little-endian machine with a version byte; decide where the byte
  goes in `to_bytes`, since the
  golden-hash row in § Verification takes its SHA-256 over `to_bytes`,
  and write `FORMAT.md` at the
  root in bitrep's style (tests/data/bitrep/README.md describes bitrep's
  layout; its FORMAT.md is in
  the published crate).
  
  After it: fl-6shtzpj (the laws kit), fl-v7t6xhz (Binned). The portfolio
  ask fl-cfx001b is still
  open. `CARGO_TARGET_DIR` is set outside ~/Documents by the repo's
  `.claude/settings.local.json`.
---
Per DESIGN.md § Wire format: `FORMAT.md` in bitrep's style; `to_bytes` and `from_bytes` on all three
exact types; `serde` behind its feature, encoding the same bytes.

Tests `wire_*`:
- round-trip;
- a hand-built non-canonical `Exact` is refused with a typed error, never a panic. Cover an even
  mantissa, a leading zero limb, and a compact-representable value encoded as the heap tier;
- bincode 1.3 fixed-int round-trip under the feature.

## log
- 2026-10-11T15:02Z created
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-wire @29c925976054
- 2026-10-11T18:13Z cover docs/DESIGN.md#sp-wire @ed67fb6c74a2
- 2026-10-11T18:33Z cover docs/DESIGN.md#sp-wire @10a66b73f9bd
- 2026-10-12T12:53Z handoff by claude (be8fad00-93e4-42e3-8b8f-40c127bab5c7)
