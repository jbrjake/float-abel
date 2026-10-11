---
id: fl-dwrey99
title: "Implement Exact's compact tier — the canonical (lo, hi, exp) dyadic, add with promotion detection, negate, Ord, Hash and to_f64"
category: crate/types
labels: [reproducible-floats, rf-critical-path]
docs: [docs/DESIGN.md#exact-sp-exact]
needs: [fl-1dw33g3]
seq: 40
verify: run cargo test package=float-abel compact_
status: done
created: 2026-10-11T15:02Z
covers:
  - ref: docs/DESIGN.md#sp-exact
    sha: 37e69a8f5bc8790986626d0213167f858411aeb661df18d79b1bdcc02ffcca54
  - ref: docs/DESIGN.md#sp-posture
    sha: 04d039091108bd8502167ce4738d21b9f68f75eb031533737343239b0e7d300e
relates: [fl-te1748m]
---
`src/exact/compact.rs` per DESIGN.md § Exact. Canonical form: mantissa odd, or the zero form. `add`
returns `Err(NeedsWide)` when the aligned span exceeds 127 bits or a special is present.

Tests `compact_*`:
- the group laws on same-scale inputs;
- `a + b − b == a` on 10⁶ random pairs;
- `size_of::<Exact>() == 24` as a `const` assertion. That is the measured layout; if it moves, the
  tier moved.

## log
- 2026-10-11T15:02Z created
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-exact @37e69a8f5bc8
- 2026-10-11T16:32Z handoff by claude (ffdbc5f1-8414-4a1d-8060-cf0fb191411f)
- 2026-10-11T16:33Z handoff by claude (ffdbc5f1-8414-4a1d-8060-cf0fb191411f)
- 2026-10-11T16:43Z open→doing — claimed by claude (bd2b6c06-c485-4bc5-9de3-2093f3abcd32)
- 2026-10-11T16:56Z cover docs/DESIGN.md#sp-posture @04d039091108
- 2026-10-11T16:56Z doing→done — verify exit 0 @ 7a1c500+1

## comments
- 2026-10-11T16:56Z [claude (bd2b6c06-c485-4bc5-9de3-2093f3abcd32)] Decisions taken in this task, derived from DESIGN.md and the standing decisions, unratified until the
  owner confirms them:
  
  1. `Exact` exists only with the `alloc` feature. Without `alloc` there is no `Exact`; `Fixed` and
     `Wide` serve. A no-alloc `Exact` would need a named panic on promotion and on every NaN input,
     which "the only panic is the named `Exact` limb cap" and "`From<f64>` is total on `Wide` and
     `Exact`" rule out. DESIGN.md § Features and Cargo.toml already say `alloc` provides `Exact`.
  2. `Exact` is `pub struct Exact(Tier)` over a private `enum Tier { Compact { lo, hi, exp },
     Wide(Box<WideExact>) }`. An enum's variant fields are as visible as the enum, so a public enum
     would let a consumer build `Exact::Compact { lo: 2, .. }`, a non-canonical value that breaks
     structural `Eq`. The newtype keeps 24 bytes: measured 24 on rustc 1.74.0 and 1.99.0 for the
     struct variant and for the newtype over it, 32 for a nested `Compact` struct (padding is never a
     niche) and 32 for an `i128` field.
  3. This task's surface is crate-internal, with the `compact_*` tests as unit tests under
     `src/exact/compact/tests.rs` rather than `tests/compact.rs`: no public item could be total before
     the heap arm lands, and a `pub(crate)` tier is invisible to integration tests. `src/exact/mod.rs`
     carries `#![cfg_attr(not(test), allow(dead_code))]` until something public reaches it.
  4. `alloc` joins the self dev-dependency's features and the posture test, so a bare `cargo test`
     reaches the tiers. `std` stays independent of `alloc`, as DESIGN.md § Features says, so a
     default-features consumer has no `Exact`. Question for the owner: should `std` imply `alloc`?
  5. The compact tier judges `NeedsWide` on the aligned frame: `2^127` is refused as `(2^127 - 1) + 1`
     at exponent 0 and computed as `(2^127 - 2) + 2` at exponent 1. The heap tier's unconditional
     demotion closes that gap, so `Exact`'s canonical form is unaffected; the test pins both routes.
  6. The tier also carries `from_product`, `from_i64` and `checked_mul`, the compact halves of
     `deposit_product`, `Mul<i64>` and `Mul<Exact>`, since each is a few lines on `i128` and the heap
     task needs them. Exponents add in `checked_mul` as `i32`, wrapping in release past some two
     million nested products of the smallest double, with a `debug_assert!` in debug builds, as
     `Wide`'s headroom does. The design's limb cap bounds the mantissa, not the exponent. Question for
     the owner: does the exponent need a bound of its own?
  
  `sp-posture`: the dev-dependency sentence gained `alloc`. This task covers the moved clause; the
  bootstrap task's pin shows as a re-open candidate in `spec audit`, which is the record that the
  clause moved after it closed.
