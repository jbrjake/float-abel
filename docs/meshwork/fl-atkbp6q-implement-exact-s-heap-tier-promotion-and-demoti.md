---
id: fl-atkbp6q
title: "Redesign Exact as a 32-byte Copy plain word — the compact dyadic plus i32 special counts, total From<f64>, Mul<i64> and the integer-amount Mul<Exact>, the width, exponent and count caps, and no heap arm"
category: crate/types
labels: [reproducible-floats, rf-critical-path]
docs: [docs/DESIGN.md#exact-sp-exact, docs/DESIGN.md#nan-and--sp-specials, docs/DESIGN.md#exactness-bounds-sp-bounds, docs/DESIGN.md#operations-and-conversions-sp-ops]
needs: [fl-dwrey99, fl-6zvrgbr, fl-ezsfgkm]
seq: 45
verify: run cargo test package=float-abel exact_ring_
status: doing
created: 2026-10-11T15:02Z
covers:
  - ref: docs/DESIGN.md#sp-exact
    sha: 2bef72cba1cb6c69e8880106982a7f6b62670aca0c89556c1d1e908fbb772cc7
  - ref: docs/DESIGN.md#sp-specials
    sha: addd4f3bbbdf9d2ff253af0022364ac7d0702feb43c1a014a14161878ec9de0f
  - ref: docs/DESIGN.md#sp-bounds
    sha: 0124e19ca3d4e2178981169bdff1c916ab8cf3a75c536ea0e995bbcaf686a122
  - ref: docs/DESIGN.md#sp-ops
    sha: 63e9572f1cf144a73be825703e1b020e3cdbf1f4882dd181f76ef89dadfb91d9
handoff: |
  The compact tier is in and proven; the heap tier is next, and it is
  where `Exact` becomes public.
  
  What exists, at 7a1c500:
  - `src/exact/mod.rs`: `pub(crate) struct Exact(Tier)` over `enum Tier {
  Compact { lo: u64, hi: u64,
  exp: i32 } }`, `Exact::ZERO`, `from_compact`, `compact()` (returns a
  `Compact`; it becomes a match
  arm once `Wide` exists), and the `const` size assertion at 24. The
  module is
  `#[cfg(feature = "alloc")]` in `src/lib.rs` and carries
  `#![cfg_attr(not(test), allow(dead_code))]`
  because nothing public reaches it: delete that line when `Exact` is
  re-exported.
  - `src/exact/compact.rs`: `pub(crate) struct Compact { m: i128, exp: i32
  }` and `NeedsWide`.
  `from_f64`, `from_product`, `from_i64`, `checked_add`, `checked_mul`,
  `Neg`, `Ord` and `Hash`
  (derived on the canonical form), `read() -> (f64, bool)`, `div_f64`,
  `is_zero`, and
  `parts()`/`from_parts()` for the storage form. Every `NeedsWide` is a
  magnitude of `2^127` or more
  in the aligned frame, or a special at construction; `i128::MIN` never
  appears, which is what makes
  `Neg` total.
  - `src/exact/compact/tests.rs`: the ten `compact_*` unit tests, with
  their own `Rng` (unit tests
  cannot see tests/support.rs), `pow2(e)` by bits, and `widest()`, which
  builds `2^127 - 1` through
  the adds that reach it.
  
  Shape for the heap arm (DESIGN.md § Exact, § NaN and ±∞, §
  Exactness bounds, § Operations and
  conversions; the task pins all four):
  - `src/exact/wide.rs`: `WideExact { sign, mant: Vec<u64>, exp, specials:
  [Dyadic; 3] }`, normalised
  (no leading zero limbs, mantissa odd or zero). The special amounts are
  dyadic because `Mul<Exact>`
  scales them by finite factors, and the monoid-algebra product multiplies
  amounts by amounts, so they
  grow like mantissas. The design's `Dyadic` there is D, not
  `ieee::Dyadic` (a u64 mantissa). Read
  § NaN and ±∞ before choosing their representation.
  - `Tier::Wide(Box<WideExact>)`: adding the arm keeps 24 bytes (measured
  on 1.74 and 1.99); the
  assertion fails if it does not.
  - Promotion: `Compact::checked_add` is `Err(NeedsWide)`, so widen both
  operands onto `WideExact` and
  add there. Demotion after every heap op: when the specials are all zero
  and the mantissa fits in
  127 bits, land on `Tier::Compact` through `Compact::from_parts`
  (canonical by construction, so
  keep the mantissa odd). Canonical form is "the smallest tier that fits",
  so structural `Eq` across
  routes holds only if demotion is unconditional; the compact test
  `compact_add_detects_promotion_at_the_127_bit_span` shows why: `2^127`
  arrives by one route as a
  refusal and by another as `(1, 0, 127)`.
  - `EXACT_MAX_LIMBS`: a `const`, default 16 × 2148 bits, so 537 limbs;
  past it a named panic
  (DESIGN.md § Open questions). CLAUDE.md's standing decision makes that
  the crate's only panic.
  - Public surface, all total: `From<f64>` with specials counted;
  `Default, Clone, Debug, PartialEq,
  Eq, PartialOrd, Ord, Hash`; `Add, AddAssign, Sub, SubAssign, Neg, Sum`
  for values and refs;
  `Mul<i64>`, `Mul<isize>`, `Mul<Exact>`; `deposit`, `deposit_product`,
  `is_zero`, `is_finite`,
  `specials()`, `to_f64`, `try_to_f64`, `div_f64`; and a tier observer
  such as `is_compact()`, which
  the bench task needs to count promotions. `Ord` is numeric on the finite
  part, then `(n, p, q)`
  lexicographic, as `Wide::cmp` does it. Re-export `Exact` from
  `src/lib.rs` under `alloc`, and hand
  it a `Debug` in the style of `Wide`'s.
  - `src/limbs.rs` is slice-generic and serves a `Vec<u64>` as it is;
  `round::round` and
  `round::div_f64` take any limb slice, so each heap read is one call, as
  `Wide::read` is.
  
  Tests `exact_*` in `tests/exact.rs` (add `mod exact;` to tests/main.rs,
  or
  scripts/check-test-topics.sh fails): the task body's list, plus
  promotion observed through the tier
  observer: `1e300 + 1e-300` promotes, `0.1 + 0.2` does not, and `(1e300 +
  1e-300) - 1e-300` demotes
  back to `Exact::from(1e300)` structurally and under `hash`.
  `tests/support.rs` has `Rng::any` for
  specials in the multiset.
  
  Owner questions outstanding, in the compact tier task's closing comment
  (archive): whether `std`
  should imply `alloc`
  (a default-features consumer has no `Exact` today); whether the
  exponent, an `i32` that wraps past
  some two million nested products of the smallest double, needs a bound
  of its own; and the
  `Wide::deposit_product` `OutOfWindow` question from the Wide task's
  closing comment, in the archive.
  
  Gates: `./scripts/regression.sh` green at 7a1c500, MSRV build and
  cargo-deny included.
  `CARGO_TARGET_DIR` is set outside ~/Documents by the repo's
  `.claude/settings.local.json` for Claude
  Code sessions; export it by hand elsewhere.
claimed-by: claude (bd2b6c06-c485-4bc5-9de3-2093f3abcd32)
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
