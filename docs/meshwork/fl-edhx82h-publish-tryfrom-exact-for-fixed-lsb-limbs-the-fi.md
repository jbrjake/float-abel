---
id: fl-edhx82h
title: "Publish TryFrom<&Exact> for Fixed<LSB, LIMBS>: the finite part onto the window's grid from the compact and the heap form, refused by name where try_deposit refuses"
status: doing
category: crate/ops
answers: leras#le-y8jn8md
verify: "all(contains src/exact/convert.rs /TryFrom<&Exact> for Fixed/, run cargo test target=main convert_exact_to_fixed)"
docs:
  - docs/DESIGN.md#operations-and-conversions-sp-ops
seq: -20
created: 2026-11-23T16:22Z
claimed-by: claude (088ad122-d06d-463d-8c3f-01d8f3ef0f7f)
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

## comments
- 2026-11-23T16:27Z [claude (088ad122-d06d-463d-8c3f-01d8f3ef0f7f)] 2026-11-23 owner ruled, in session, on what a nonzero special amount does. Asked with the example of a group folding 1.5 and one NaN; the owner chose "Refuse + finite_part() (Recommended)": Fixed::try_from(&e) returns Err(OutOfWindow::NotFinite), the name Fixed::try_deposit(NaN) gives, and a new reader Exact::finite_part() returns the same value with the amounts zeroed, so a consumer converts e.finite_part() and reads the counts from e.specials().
