---
id: fl-ezsfgkm
title: "Bound the exponent — EXACT_MAX_EXP checked after every exponent-changing op in both tiers, with the one named cap panic"
category: crate/types
labels: [reproducible-floats, rf-critical-path]
relates: [fl-atkbp6q]
docs:
  - docs/DESIGN.md#exact-sp-exact
  - docs/DESIGN.md#exactness-bounds-sp-bounds
verify: run cargo test package=float-abel compact_exponent
seq: 41
status: done
created: 2026-10-11T17:09Z
covers:
  - ref: docs/DESIGN.md#sp-bounds
    sha: 0124e19ca3d4e2178981169bdff1c916ab8cf3a75c536ea0e995bbcaf686a122
  - ref: docs/DESIGN.md#sp-exact
    sha: 0600b5e7e406c068ecceabf3ee34f83ae2890fed4ec4f9b320c5d8ef76af6876
---

Owner's rulings, 2026-10-11: "yes exponents need bounds if they can roll over!", and on the
number, 2^30 over a bound tied to the limb cap's 16-deep chain: "i guess 2^30 if it's more
compatible and allows more stuff". It is: the limb cap bounds width and the exponent bound
bounds position, so the two are independent, and a larger exponent bound costs nothing (the
stored exponent is an `i32` either way) while allowing about a million nested products of the
smallest subnormal instead of thirty-two.

Only multiplication grows an exponent without bound: a sum's exponent stays within the inputs'
range plus the mantissa width, but `checked_mul` and the heap `Mul<Exact>` add exponents, so a
chain of products of one-bit mantissas (the smallest subnormal is `1 · 2^-1074`) walks the `i32`
to overflow without ever touching the limb cap.

The bound: `EXACT_MAX_EXP = 1 << 30`, a `const` in `src/exact/mod.rs` beside the limb cap to come.
Every exponent-changing operation computes in `i64`, normalises, and narrows through one check,
`bounded`, which reports `Overflow::Exponent`; the compact tier's error is the enum
`Overflow { NeedsWide, Exponent }`, so an exponent overflow is never mistaken for a promotion.
The compact tier never promotes on exponent grounds, because the heap tier carries the same
bound. The public operators' named panic past the cap, and the checked API that returns it as an
error, land with `Exact`'s public surface (the no-alloc task); the heap tier applies the same
bound after every op that moves an exponent (the heap task).

Tests `compact_exponent_*` in `src/exact/compact/exponent_tests.rs`: the number; the value at
the bound holds and one past it is refused by name on either side, by a product or by a sum whose
canonical form carries past; a sum across the whole bound is a width overflow, told apart from an
exponent one; a chain of `5e-324` products stops after exactly `EXACT_MAX_EXP / 1074` factors;
and no sum or two-factor product of random finite doubles ever reports the exponent.

Docs: DESIGN.md § Exactness bounds carries the exponent bound beside the limb cap, and § Exact
names both as the one cap.

## log
- 2026-10-11T17:09Z created
- 2026-10-11T17:22Z open→doing — claimed by claude (bd2b6c06-c485-4bc5-9de3-2093f3abcd32)
- 2026-10-11T17:30Z cover docs/DESIGN.md#sp-bounds @0124e19ca3d4
- 2026-10-11T17:30Z cover docs/DESIGN.md#sp-exact @0600b5e7e406
- 2026-10-11T17:30Z doing→done — verify exit 0 @ a3f776f+5
