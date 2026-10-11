---
id: fl-atkbp6q
title: "Implement Exact's heap tier — promotion and demotion, the special-value algebra, Mul<Exact> and Mul<i64>, and the limb cap"
category: crate/types
labels: [reproducible-floats, rf-critical-path]
docs: [docs/DESIGN.md#exact-sp-exact, docs/DESIGN.md#nan-and--sp-specials, docs/DESIGN.md#exactness-bounds-sp-bounds, docs/DESIGN.md#operations-and-conversions-sp-ops]
needs: [fl-dwrey99, fl-6zvrgbr, fl-ezsfgkm]
seq: 50
verify: run cargo test package=float-abel exact_heap_
status: open
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
---

`src/exact/wide.rs` and `src/exact/mod.rs` per DESIGN.md § Exact, § NaN and ±∞ and § Exactness
bounds. `WideExact { sign, mant: Vec<u64>, exp, specials: [Dyadic; 3] }`, normalised, behind the
`alloc` feature as `Tier::Wide(Box<WideExact>)`; `Exact`'s public surface already exists in its
compact-only form (the no-alloc task) and this task extends it: promotion when the compact tier
reports `Overflow::NeedsWide`, unconditional demotion after every heap op so that canonical form is
the smallest tier that fits, the special-value algebra, and `Mul<Exact>` by the monoid-algebra
product. `EXACT_MAX_LIMBS` is a `const` with the named cap panic past it, documented (DESIGN.md §
Open questions), and the heap tier applies `EXACT_MAX_EXP` after every op that moves an exponent,
through the same `PastCap` the compact tier and the operators already use. The checked API
returns `PastCap::Width` and `PastCap::Exponent` from the heap tier as it does from the compact
one, and `PastCap::Special` never occurs with `alloc` on.

Tests `exact_heap_*`:
- the laws across tiers: a value built through the heap tier equals the same value built through
  `Compact`, structurally;
- distributivity on 10⁵ random triples, specials included;
- `0 · ∞` and `0 · NaN` project `0` (the documented divergence), while `(+∞)·(−∞)` projects `−∞` and
  `(−∞)·(−∞)` projects `+∞`, each asserted so that a change is deliberate;
- the history `1e30`, `1e-10`, `−1e30` reads `1e-10` exactly;
- an adversarial fold over magnitudes 2^±300 stays exact;
- promotion and demotion observed through `is_compact`: `1e300 + 1e-300` promotes, `0.1 + 0.2` does
  not, and `(1e300 + 1e-300) - 1e-300` demotes back to `Exact::from(1e300)` structurally and under
  `hash`;
- the limb cap and the exponent bound each panic by name one step past, in every build, and the
  checked API returns them.

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
