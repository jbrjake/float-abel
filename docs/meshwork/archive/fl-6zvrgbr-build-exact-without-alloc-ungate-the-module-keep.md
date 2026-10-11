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
status: done
created: 2026-10-11T17:09Z
covers:
  - ref: docs/DESIGN.md#sp-exact
    sha: 2bef72cba1cb6c69e8880106982a7f6b62670aca0c89556c1d1e908fbb772cc7
  - ref: docs/DESIGN.md#sp-posture
    sha: 5f999b023881b121c3e419c645f4be0b68c2170e45367674deb8021a740d1df4
  - ref: docs/DESIGN.md#sp-features
    sha: ea0b8b465f36ca3c8afb33870f9a322af46279ed1f9d034f432ff0220ae6654f
---

Owner's rulings, 2026-10-11: "i don't believe we should require alloc, no. doesn't that block
zero-copy use?", and on what `Exact` does without an allocator past the inline width or on a
special input: "Cap panic plus checked API".

`Exact` builds and works on every feature set; only the heap arm sits behind `alloc`. Without it the
compact width is the cap, a special input is past it, and past the cap is the one named `Exact`
panic that the limb cap and the exponent bound also raise. The checked methods return the cap as
`PastCap { Width, Exponent, Special }` with the state untouched. The public surface, the newtype
and the cap are in DESIGN.md § Exact; the feature wiring (`std` implies `alloc`) in § Features and
§ Crate posture.

The no-alloc behaviour is reachable by no `cargo test` of this package, because the self
dev-dependency's `simd` pulls `std`, which pulls `alloc`; the workspace member `tests/noalloc`
(`float-abel-noalloc`, `default-features = false`) pins it, and the regression gate runs it.

Tests: `exact_cap_*` in `tests/exact_cap.rs` through the public surface, `noalloc_*` in
`tests/noalloc/main.rs` on the allocator-free build. The heap task's tests are `exact_heap_*`, so
its verify is not green before its work.

## log
- 2026-10-11T17:09Z created
- 2026-10-11T17:31Z open→doing — claimed by claude (bd2b6c06-c485-4bc5-9de3-2093f3abcd32)
- 2026-10-11T17:41Z cover docs/DESIGN.md#sp-exact @2bef72cba1cb
- 2026-10-11T17:41Z cover docs/DESIGN.md#sp-posture @5f999b023881
- 2026-10-11T17:41Z cover docs/DESIGN.md#sp-features @ea0b8b465f36
- 2026-10-11T17:41Z doing→done — verify exit 0 @ 1c81906+3

## comments
- 2026-10-11T17:41Z [claude (bd2b6c06-c485-4bc5-9de3-2093f3abcd32)] Built per the rulings above. Read back, derived and unratified until the owner confirms:
  
  1. `std = ["alloc"]`: a std build always has an allocator, and a default-features consumer must get
     the whole of `Exact`, so the default build carries the heap tier. DESIGN.md § Features' `std`
     row now says it pulls in `alloc`.
  2. The allocator-free build is tested from a workspace member, `tests/noalloc/`, because the self
     dev-dependency's `simd` feature pulls `std`, so no `cargo test` of the main package can build the
     library without `alloc`. The root manifest gained `[workspace]` and the lints moved to
     `[workspace.lints]` so both members share them. A bare `cargo test` still runs the main package
     alone. `scripts/regression.sh` runs the member and lints the whole workspace.
  3. DESIGN.md § Exact's code block now shows the newtype over the private tier enum, with the
     reason, and corrects a number: `(i128, i32)` is 32 bytes with or without the `Box` arm (the text
     said 48), measured on rustc 1.74.0 and 1.99.0.
  4. CLAUDE.md's standing decisions carry the three rulings in the owner's words.
  5. In this tree no build has the heap tier yet, so `From<f64>` of a special and a sum past the
     inline width hit the cap on every feature set; the code docs say so, and the heap task changes
     the `alloc` branch.
