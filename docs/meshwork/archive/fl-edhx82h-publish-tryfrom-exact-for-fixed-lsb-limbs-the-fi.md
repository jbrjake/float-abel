---
id: fl-edhx82h
title: "Publish TryFrom<&Exact> for Fixed<LSB, LIMBS>: the finite part onto the window's grid from the compact and the heap form, refused by name where try_deposit refuses"
status: done
category: crate/ops
answers: leras#le-y8jn8md
verify: "all(contains src/exact/convert.rs /TryFrom<&Exact> for Fixed/, run cargo test target=main convert_exact_to_fixed)"
docs:
  - docs/DESIGN.md#operations-and-conversions-sp-ops
seq: -20
created: 2026-11-23T16:22Z
covers:
  - ref: docs/DESIGN.md#sp-ops
    sha: 2e9d7489c556aa10670234651d9f005ff06eb7ac95890adddc4bd0dd7e870f5e
---

Answers leras#le-y8jn8md. leras-dd's exact product lane wants to fold `Exact` (the independent
carrier) and convert at the read into `Fixed<-2148, 68>`, the engine's product window. The crate
has `TryFrom<&Fixed> for Exact`, `TryFrom<&Wide> for Exact` and `TryFrom<&Exact> for Wide`, and no
`TryFrom<&Exact> for Fixed`; `Wide` refuses a product below 2^-1074.

Build `impl<const LSB: i32, const LIMBS: usize> TryFrom<&Exact> for Fixed<LSB, LIMBS>` in
`src/exact/convert.rs`, `Error = OutOfWindow`: the finite part placed on the window's grid from
the compact form and from the heap form, `OffGrid` for a set bit below `2^LSB`, `AboveCeiling` for
a magnitude at or past `2^CEILING_EXP`, where `Fixed::try_deposit` refuses, and no partial state.
What a nonzero special amount does is the owner's ruling, asked in session before the build.

Tests in tests/convert.rs, named `convert_exact_to_fixed_*`: round trip with `Exact::try_from(&Fixed)`
on random states of `Fixed256` and `Fixed<-2148, 68>`, compact and heap; each refusal by name at
its edge; the fold of products in `Exact` converted against the same fold in `Fixed`. DESIGN.md
§ Operations and conversions and `Fixed`'s doc name the conversion.

## log
- 2026-11-23T16:22Z created
- 2026-11-23T16:27Z open→doing — claimed by claude (088ad122-d06d-463d-8c3f-01d8f3ef0f7f)
- 2026-11-23T16:38Z cover docs/DESIGN.md#sp-ops @2e9d7489c556
- 2026-11-23T16:38Z doing→done — verify exit 0 @ c9fed1e+2

## comments
- 2026-11-23T16:27Z [claude (088ad122-d06d-463d-8c3f-01d8f3ef0f7f)] 2026-11-23 owner ruled, in session, on what a nonzero special amount does. Asked with the example of a group folding 1.5 and one NaN; the owner chose "Refuse + finite_part() (Recommended)": Fixed::try_from(&e) returns Err(OutOfWindow::NotFinite), the name Fixed::try_deposit(NaN) gives, and a new reader Exact::finite_part() returns the same value with the amounts zeroed, so a consumer converts e.finite_part() and reads the counts from e.specials().
- 2026-11-23T16:38Z [claude (088ad122-d06d-463d-8c3f-01d8f3ef0f7f)] Done at c9fed1e. For leras (le-y8jn8md, and le-sekcebp which needs it), what the crate now publishes:
  
  - `impl<const LSB: i32, const LIMBS: usize> TryFrom<&Exact> for Fixed<LSB, LIMBS>`, `Error = OutOfWindow`. The finite part lands on the window's grid from the compact word and from the heap form (a Σ a·b over 1e300 beside 1e-300 converts). Refusals, in this order: `NotFinite` when any special amount is nonzero; `OffGrid` for a set bit below `2^LSB`; `AboveCeiling` at or past `2^CEILING_EXP`. A refusal builds nothing.
  - `Exact::finite_part(&self) -> Exact`: the same value with the amounts left behind, canonical (compact when it fits). So the read of a group that may hold a NaN or ±∞ is `Fixed::try_from(&e.finite_part())` for the limbs and `e.specials()` for the three counts (`None` there means a dyadic amount or one past `i64`, which a product of two doubles never makes: its amounts are unit counts).
  - `Fixed<-2148, 68>` takes every finite product of two doubles: pinned on `5e-324 · 5e-324` and `f64::MAX · −f64::MAX`, and on 300 random product folds with retractions against the same fold in the window.
  
  Verified here: `cargo test` 210 + 38 + 14 green; `-p float-abel-noalloc` 7 green (the compact path with no features); `cargo clippy --workspace --all-targets -- -D warnings`, both `--lib` lints and `RUSTDOCFLAGS="-D warnings" cargo doc` clean; the four `convert_exact_to_fixed_*` tests green under miri. Each test was watched failing on a deliberate break: the specials check removed, the heap ceiling off by one, the heap sign dropped, the heap grid check dropped, `finite_part` keeping its amounts.
  
  Not verified: leras's own build against it (their verify, in their tree), the dbsp-feature lint and the x86 lints beyond what the push gate runs.
