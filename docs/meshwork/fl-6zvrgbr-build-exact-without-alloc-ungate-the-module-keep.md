---
id: fl-6zvrgbr
title: "Build Exact without alloc — ungate the module, keep only the heap arm behind the feature, and make the inline width the cap when there is no allocator"
category: crate/types
labels: [reproducible-floats, rf-critical-path]
relates: [fl-atkbp6q]
docs:
  - docs/DESIGN.md#exact-sp-exact
  - docs/DESIGN.md#crate-posture-sp-posture
  - docs/DESIGN.md#features-sp-features
verify: "all(lacks src/lib.rs /(?s)cfg\\(feature = \"alloc\"\\)\\]\\s*mod exact;/, run cargo test package=float-abel compact_)"
seq: 42
status: open
created: 2026-10-11T17:09Z
---

Owner's rulings, 2026-10-11: "i don't believe we should require alloc, no. doesn't that block
zero-copy use?", and on what `Exact` does without an allocator past the inline width or on a
special input: "Cap panic plus checked API".

`Exact` builds and works on every feature set. `mod exact;` in `src/lib.rs` loses its
`#[cfg(feature = "alloc")]`; only the heap arm, `Tier::Wide(Box<WideExact>)` and `src/exact/wide.rs`,
sits behind the feature. Without an allocator the compact width is the cap: a promoting add and a
special input are past it, and past the cap is the one named `Exact` panic (DESIGN.md § Exact), the
same panic the limb cap and the exponent bound raise. `From<f64>` is total with `alloc`, and the
docs say on which feature sets it is. The `const` size assertion holds in both configurations (a
single-arm enum is 24 bytes too, measured on 1.74 and 1.99).

The public surface this task lands, in its compact-only form, which the heap task extends rather
than replaces:
- `pub struct Exact(Tier)`, re-exported from `src/lib.rs`; `Exact::ZERO`, `Default`, `Clone`,
  `Debug` in `Wide`'s style, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`;
- `pub enum PastCap { Width, Exponent, Special }` with `Display` and `std::error::Error`: the one
  cap, named by which bound was crossed. `Width` is the inline width without `alloc` and
  `EXACT_MAX_LIMBS` with it; `Special` is a NaN or ±∞ reaching a build without `alloc`, and never
  occurs with it;
- the checked API, total: `checked_add`, `checked_sub`, `checked_mul` (by `Exact` and by `i64`),
  `checked_deposit(f64)`, `checked_deposit_product(f64, f64)` and `try_from_f64`, each returning
  `Result<_, PastCap>` and leaving the state untouched on `Err`. `Neg` is total on every tier, so
  it has no checked form;
- the operators, which panic by name past the cap: `From<f64>`, `Add`, `AddAssign`, `Sub`,
  `SubAssign`, `Neg`, `Sum` for values and refs, `Mul<i64>`, `Mul<isize>`, `Mul<Exact>`, `deposit`,
  `deposit_product`;
- the reads: `to_f64`, `try_to_f64`, `div_f64`, `is_zero`, `is_finite`, `is_compact` (the tier
  observer the bench task needs), and `EXACT_MAX_EXP` made `pub`.

Tests `exact_cap_*` in `tests/exact_cap.rs` (add `mod exact_cap;` to tests/main.rs): the operators
panic by name past the exponent (a chain of products); each checked method returns the same
`PastCap` and leaves the state as it was; the size assertion through the public type; and the
compact-only group laws through the operators. The heap task's `exact_*` filter must not be green
before its work, so its verify moves to `exact_heap_` and its tests carry that prefix.

The no-alloc behaviour cannot be reached by any `cargo test` of this package: the self
dev-dependency's `simd` feature pulls `std`, and `std` implies `alloc` (a manifest change this task
makes, `std = ["alloc"]`, since a std build always has an allocator and a default-features consumer
must get the whole of `Exact`; read back to the owner). So the no-alloc panics are exercised from a
workspace member, `tests/noalloc/` (package `float-abel-noalloc`, `default-features = false`,
`publish = false`), whose `#[should_panic]` tests pin `1e300 + 1e-300`, `Exact::from(f64::NAN)` and
`checked_add` returning `PastCap::Width`/`PastCap::Special`, run as
`cargo test package=float-abel-noalloc`, and the regression gate and CI gain that leg. The root
manifest gains `[workspace] members = ["tests/noalloc"]`; `include` already keeps `tests/` out of
the published crate.

Docs: DESIGN.md § Exact (the newtype, the checked API, the no-alloc cap), § Features (`std` pulls in
`alloc`; the `alloc` row says what the feature adds, not that it gates `Exact`), § Crate posture,
and the crate docs in `src/lib.rs`. The CI `msrv` job already builds `--no-default-features` and
`--features alloc`; the regression gate's clippy legs cover the same three sets.

## log
- 2026-10-11T17:09Z created
