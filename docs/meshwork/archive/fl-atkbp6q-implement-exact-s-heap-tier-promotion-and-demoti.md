---
id: fl-atkbp6q
title: "Redesign Exact as a 32-byte Copy plain word — the compact dyadic plus i32 special counts, total From<f64>, Mul<i64> and the integer-amount Mul<Exact>, the width, exponent and count caps, and no heap arm"
category: crate/types
labels: [reproducible-floats, rf-critical-path]
docs: [docs/DESIGN.md#exact-sp-exact, docs/DESIGN.md#nan-and--sp-specials, docs/DESIGN.md#exactness-bounds-sp-bounds, docs/DESIGN.md#operations-and-conversions-sp-ops]
needs: [fl-dwrey99, fl-6zvrgbr, fl-ezsfgkm]
seq: 45
verify: run cargo test package=float-abel exact_ring_
status: done
created: 2026-10-11T15:02Z
covers:
  - ref: docs/DESIGN.md#sp-exact
    sha: 4abe01d06111f39d7632a44e18614ad267263ee7eb118c4c0c474555a880d7e4
  - ref: docs/DESIGN.md#sp-specials
    sha: 970a012ac1267e79250bece05c7a57ab4a8c62fbb42eb000dc567b5b3b5ad665
  - ref: docs/DESIGN.md#sp-bounds
    sha: c3c9133fb5763f6d2e6e53fe2dc3bd77cc0125e0cd428ca8f288a14113bf7d09
  - ref: docs/DESIGN.md#sp-ops
    sha: f7da3b4f752d48550b0d1253177e1a361e53f1ae423d10b36e1c23a62a107928
---

Owner's rulings, 2026-10-11: zero-copy is paramount and "copy/alloc has to be limited to only the
bare necessities"; "in general i approve dropping the heap arm"; "we absolutely need exact
multiplication. it's incredibly useful"; specials "counted and projected, never refused".

`Exact` is a 32-byte `Copy` plain word: `#[repr(C)] struct Exact { lo: u64, hi: u64, exp: i32,
nan: i32, pos_inf: i32, neg_inf: i32 }`, no padding, pinned by `const` assertions. The finite part
is the compact tier's canonical dyadic; the three counts are the special vector of DESIGN.md § NaN
and ±∞, signed so that a retraction subtracts. `From<f64>` is total, `Mul<i64>` scales the finite
part and the counts, `Mul<Exact>` is the monoid-algebra product with the amounts kept integer: a
product whose special amount would stop being an integer, or would leave `i32`, is the cap. The
caps are `PastCap { Width, Exponent, Count }`, named on the operators and returned by the checked
API. There is no heap arm, no `alloc` feature and no limb cap; the `noalloc` member keeps pinning
the no-feature build. `Ord` is numeric on the finite part, then `(n, p, q)`. Projection at read is
`Wide`'s rule, shared.

Tests `exact_ring_*` in `tests/exact_ring.rs`:
- the group laws with specials in the multiset, and `Mul<i64>` on the counts;
- distributivity and associativity of `Mul<Exact>` on 10⁵ random triples, specials included, on
  inputs whose products fit the width;
- `0 · ∞` and `0 · NaN` are the zero, `(+∞)·(−∞)` projects `−∞` and `(−∞)·(−∞)` projects `+∞`,
  `2 · (one NaN)` is two NaN and `2.5 · (one NaN)` is `PastCap::Count`, each asserted so that a
  change is deliberate;
- the history `1e30`, `1e-10`, `−1e30` reads `1e-10` exactly;
- a fold over magnitudes `2^±300` is `PastCap::Width` from the checked API, and `Wide` holds it;
- `size_of == 32`, `align_of == 8`, and `Exact` is `Copy`.

Docs: DESIGN.md § Exact, § The trilemma, § NaN and ±∞, § Exactness bounds, § Operations and
conversions, § Features, § Crate posture, § Wire format, § Open questions and § Non-goals; CLAUDE.md's
standing decisions.

## log
- 2026-10-11T15:02Z created
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-exact @37e69a8f5bc8
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-specials @addd4f3bbbdf
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-bounds @943ce56af93a
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-ops @cf4faaa1f100
- 2026-10-11T16:56Z handoff by claude (bd2b6c06-c485-4bc5-9de3-2093f3abcd32)
- 2026-10-11T16:57Z handoff by claude (bd2b6c06-c485-4bc5-9de3-2093f3abcd32)
- 2026-10-11T17:30Z cover docs/DESIGN.md#sp-exact @0600b5e7e406
- 2026-10-11T17:30Z cover docs/DESIGN.md#sp-specials @addd4f3bbbdf
- 2026-10-11T17:30Z cover docs/DESIGN.md#sp-bounds @0124e19ca3d4
- 2026-10-11T17:30Z cover docs/DESIGN.md#sp-ops @cf4faaa1f100
- 2026-10-11T17:41Z cover docs/DESIGN.md#sp-exact @2bef72cba1cb
- 2026-10-11T17:41Z cover docs/DESIGN.md#sp-specials @addd4f3bbbdf
- 2026-10-11T17:41Z cover docs/DESIGN.md#sp-bounds @0124e19ca3d4
- 2026-10-11T17:41Z cover docs/DESIGN.md#sp-ops @63e9572f1cf1
- 2026-10-11T18:03Z open→doing — claimed by claude (bd2b6c06-c485-4bc5-9de3-2093f3abcd32)
- 2026-10-11T18:12Z cover docs/DESIGN.md#sp-exact @4abe01d06111
- 2026-10-11T18:12Z cover docs/DESIGN.md#sp-specials @970a012ac126
- 2026-10-11T18:12Z cover docs/DESIGN.md#sp-bounds @c3c9133fb576
- 2026-10-11T18:12Z cover docs/DESIGN.md#sp-ops @f7da3b4f752d
- 2026-10-11T18:12Z doing→done — verify exit 0 @ 3c2a0d0+5
