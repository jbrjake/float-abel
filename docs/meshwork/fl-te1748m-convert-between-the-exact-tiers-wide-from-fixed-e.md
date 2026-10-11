---
id: fl-te1748m
title: "Convert between the tiers — Wide: From<Fixed<..>> exact; Exact: TryFrom<&Wide> and TryFrom<&Fixed<..>> refused past the width; Wide::try_from(&Exact) refused past the sum range"
status: open
category: crate/types
labels: [reproducible-floats]
needs: [fl-atkbp6q]
relates: [fl-at6nh41]
verify: run cargo test package=float-abel convert_
docs:
  - docs/DESIGN.md#operations-and-conversions-sp-ops
seq: 55
created: 2026-10-11T16:32Z
covers:
  - ref: docs/DESIGN.md#sp-ops
    sha: 7675d055f5a4e08218fb87c0e966dbe6b45fa34b6c986dbf4cb55c086bcce4dc
handoff: |
  The tree at 98b5ca2 is green through `./scripts/regression.sh` (exit 0:
  84 tests in the main
  package plus the release leg, `tests/noalloc`, workspace clippy,
  rustdoc, the MSRV 1.74 build,
  cargo-deny). Since the last handoff three tasks closed: every limb and
  counter overflow on `Wide`
  and `Fixed` is an unconditional `assert!` with a named message, the five
  `*_in_every_build` tests
  run under `--release` in the push gate; DESIGN.md § ops and § Wide say
  `deposit_product` is refused
  by name on `Fixed` and `Wide` outside the window and that `Exact` is
  total; `Wide` and `Fixed` are
  `#[repr(C)]` with a `const` assertion on size and alignment,
  `Wide::from_parts(limbs, [n, p, q])`
  and `Fixed::from_limbs(limbs)` are the total inverses of the accessors,
  and `tests/layout.rs`
  rebuilds values from `to_le_bytes` of each limb without `unsafe`.
  
  This task is next. Its body's first paragraph predates the POD `Exact`:
  the title and DESIGN.md
  § ops are current, so build `Wide: From<Fixed<LSB, LIMBS>>` (exact,
  with the `const` assertion on
  `(LSB, LIMBS)`), `Exact: TryFrom<&Wide>` and `Exact: TryFrom<&Fixed<LSB,
  LIMBS>>` returning
  `PastCap::Width` past the 127-bit width (and `PastCap::Count` when a
  `Wide` counter leaves `i32`),
  and `Wide::try_from(&Exact)` refusing by name below the unit, at the
  ceiling, or on a non-integer
  special amount. `Wide::from_parts` and `Fixed::from_limbs` make the limb
  shifts expressible from
  outside the modules; `Exact`'s fields are private to `src/exact/mod.rs`
  and the compact dyadic to
  `src/exact/compact.rs`, so the `Exact` ends of the conversions live in
  `src/exact/`. The unit of
  `Wide` is `2^-1074`, `Fixed`'s is `2^LSB`, and the shift between them is
  `LSB + 1074` bits,
  sign-extended; `src/limbs.rs` has `set_shifted`, `negate`, `add_assign`,
  `abs_into`, `is_negative`.
  
  Tests `convert_*` in `tests/convert.rs` (add `mod convert;` to
  `tests/main.rs`): `Fixed256` states
  round-trip through `Wide` and `Exact` bit for bit on random multisets
  (the splitmix in
  `tests/layout.rs` is a usable stream); `Wide` states with specials
  round-trip through `Exact`;
  `try_from(&Exact)` refuses by name past the range and accepts at the
  edge; and the `const`
  assertion is a `trybuild` compile-fail case for `Fixed<-1076, 20>`.
  `trybuild` is not yet a
  dev-dependency and `tests/ui/` does not exist: add it pinned `=x.y.z`
  (CLAUDE.md's Binned decision
  already names `trybuild`, so the dependency is settled), and keep the
  compile-fail target out of
  the smoke gate's time budget if it proves slow.
  
  After it: fl-z91b7pq (correct rounding against CPython vectors),
  fl-a2wrx7w (the wire format;
  § Wire now says each body is the `repr(C)` bytes, so `to_bytes` is
  `to_le_bytes` of the limbs then
  the counters, and `from_bytes` on `Wide` and `Fixed` can be total while
  `Exact`'s validates),
  fl-6shtzpj (the laws kit), fl-v7t6xhz (Binned). The portfolio ask
  fl-cfx001b is still open.
  `CARGO_TARGET_DIR` is set outside ~/Documents by the repo's
  `.claude/settings.local.json`.
---

DESIGN.md § Operations and conversions names four conversions between the exact tiers that no
other task implements: `Wide: From<Fixed<LSB, LIMBS>>`, `Exact: From<Wide>` and
`Exact: From<Fixed<LSB, LIMBS>>`, all exact, and `Wide::try_from(&Exact)`, which returns `Err`
past the sum range. Nothing lossy gets a `From`; the `Binned` conversions belong to the Binned task.

`From<Fixed>` into `Wide` is exact only when `LSB ≥ -1074` (a finer grid holds values below
`Wide`'s unit) and `Fixed`'s ceiling lies inside `Wide`'s range, so the impl carries a `const`
assertion on `(LSB, LIMBS)` rather than a runtime refusal. The shift is `LSB + 1074` limbs-and-bits
onto `Wide`'s grid, sign-extended. `Wide::try_from(&Exact)` refuses a finite part below the unit or
at the ceiling, and a special amount that is not an integer count, by name.

Tests `convert_*` in `tests/convert.rs`: every `Fixed256` state round-trips through `Wide` and
`Exact` bit for bit on random multisets; `Wide` states with specials round-trip through `Exact`;
`try_from(&Exact)` refuses by name past the range and accepts at the edge; the `const` assertion is
a `trybuild` compile-fail case for `Fixed<-1076, 20>`.

## log
- 2026-10-11T16:32Z created
- 2026-10-11T16:32Z cover docs/DESIGN.md#sp-ops @cf4faaa1f100
- 2026-10-11T17:41Z cover docs/DESIGN.md#sp-ops @63e9572f1cf1
- 2026-10-11T18:12Z cover docs/DESIGN.md#sp-ops @f7da3b4f752d
- 2026-10-11T18:28Z cover docs/DESIGN.md#sp-ops @7675d055f5a4
- 2026-10-11T18:35Z handoff by claude (0f58a8d2-1e20-4d5a-ab10-caf961d3912f)
