---
id: fl-th7g69s
title: "Restore the heap arm as the 32-byte niche form — Exact promotes past the 127-bit width to one allocation of limbs tagged in the mantissa's odd-only niche and demotes back after every op, holds every sum and every product fold, carries dyadic special amounts, panics only at the limb cap and the exponent bound, lifts unsafe in src/exact/heap.rs alone, and builds compact-only without alloc"
category: crate/types
labels: [reproducible-floats, rf-critical-path]
discovered-from: fl-t154dsp
needs: [fl-nhcdk92]
seq: 12
verify: run cargo test package=float-abel exact_heap_
docs:
  - docs/DESIGN.md#exact-sp-exact
  - docs/DESIGN.md#nan-and--sp-specials
  - docs/DESIGN.md#exactness-bounds-sp-bounds
  - docs/OVERFLOW.md#options
status: open
created: 2026-10-15T15:33Z
---

Owner, 2026-10-15, in session: "yes let's do heap and allow product fols inside a dataflow."
This reverses the 2026-10-11 drop ("in general i approve dropping the heap arm"), whose
consequence was not read back as a SUM question (fl-t154dsp's comments carry the record). On the
layout, same session: "yes i'm okay with one more unsafe to save that many bits on every value":
the 32-byte niche form, with `unsafe` in one module.

The design to build is DESIGN.md § Exact as first committed (622ad8a), with what the 32-byte word
got right kept: the heap form is reached by promotion only, canonical form is the smallest form
that represents the value (mantissa odd or zero in the compact form, no leading zero limbs in the
heap form, demotion after every heap op), structural `Eq` stays numeric, `Ord` numeric across
forms, `Hash` consistent. Specials: the compact form keeps today's inline `i32` counts; the heap
form carries the design's dyadic amounts, so `2.5 · NaN` and a count past `i32` promote instead
of capping, and `PastCap::Count` goes away. Products: `Mul<Exact>` on every pair of forms, the
monoid algebra of § NaN and ±∞ with dyadic amounts, so a `float × float` join and a fold of its
products hold; the multi-limb multiply is schoolbook over at most the cap's limbs. `Mul<i64>`,
`Neg` total, `From<f64>` and `product(a, b)` total. The caps: `EXACT_MAX_LIMBS`, a `const` the
consumer can raise, default 16 × 2148 bits ≈ 4.3 KB, reachable only through ≥ 16 nested
`float × float` joins on extreme values, the one named panic past the exponent bound
(`EXACT_MAX_EXP`, kept); the checked API returns them. Memory per weight is bounded by
`k · 2148 + 64` bits for the deepest join chain of depth k (§ Exactness bounds).

The layout, ruled: today's `repr(C)` 32-byte word unchanged for the compact form, whose canonical
mantissa is odd or the zero, so an even nonzero `lo` is not a value; the heap form takes that
niche, `lo` even and nonzero carrying the limb count, `hi` the pointer to one allocation of
limbs, `exp` and the counts or amounts inline or in the block as fl-nhcdk92's findings settle.
`Exact` is not `Copy`; `Clone` is a copy on the compact form and an allocation on the heap form,
`Drop` frees the block. `src/exact/heap.rs` is the one module beside `src/simd/` where
`#![deny(unsafe_code)]` is lifted, each block with its `// SAFETY:` line stating the invariant
(the tag bit decides which form; a heap `hi` is a pointer this module allocated with the length
in `lo`; no other code reads the fields), `cargo +nightly miri test` covers it, and CLAUDE.md
§ Standing decisions says so in the owner's words. The same-scale `dd_sum` row must land within
the budget fl-nhcdk92 measures (a few percent of the Copy word).

Features: `alloc` returns, `std` implies it, and the self dev-dependency turns it on. Without
`alloc` the compact-only `Exact` stands with the width as its cap and the checked API, per the
owner's 2026-10-11 "Cap panic plus checked API"; `tests/noalloc` pins that build.

Tests `exact_heap_*` in tests/exact_heap.rs: a value built through the heap form equals the same
value built through the compact form, structurally and by `hash`; promotion and demotion observed
through `is_compact()`: `1e300 + 1e-300` promotes, `0.1 + 0.2` does not, and
`(1e300 + 1e-300) − 1e-300` demotes to `Exact::from(1e300)` structurally; the five histories of
docs/OVERFLOW.md through `+=` in every order, each reading the small value exactly; a fold over
2^±300 with retractions equals a `num-bigint` oracle (a dev-dependency, as the fuzz target has
it); a dot product of 10⁵ mixed-magnitude pairs through `deposit_product` and through
`Exact::product` folded by `+=` equals the oracle; a join chain of depth 3 of mixed-magnitude
weights multiplies and distributes bit for bit; `2.5 · (one NaN)` is a heap value reading NaN and
`−1 · (one +∞)` reads −∞; the limb cap panics by name one step past in every build and the
checked API returns it with the state untouched; the group and ring laws through the kit on a
2^±300 strategy, which fl-x8fwfe3's closure law must now pass on `Exact`; `size_of == 32` and
`align_of == 8` pinned; no leak under miri across promote, clone, drop and demote. Keep src files
under 500 lines: `src/exact/heap.rs` for the form, the multiply in its own file if it crosses.

Docs in the same commits: DESIGN.md § Exact, § The trilemma (the row returns to "exact + full
range", not `Copy`, 32 bytes), § NaN and ±∞, § Exactness bounds (the sentence "the exact tier
exists for the latter" is true again), § Operations and conversions, § Features, § Crate posture
(the second `unsafe` module), § Non-goals (the heap-tier bullet goes), § Open questions (the limb
cap's alternative, as at 622ad8a); CLAUDE.md § Standing decisions rewritten in the owner's
2026-10-15 words. Then file the ask to portfolio to amend PLAN-reproducible-floats.md for the
settled design (fl-cfx001b was dropped because its premise reversed).

## log
- 2026-10-15T15:33Z created

## comments
- 2026-10-15T15:36Z [claude (6cbb3399-1134-4aff-94e4-f8c1137cf60f)] Owner, 2026-10-15, in session, on the layout: "yes i'm okay with one more unsafe to save that many bits on every value". The 32-byte niche form is ruled; src/exact/heap.rs is the one module beside src/simd/ where #![deny(unsafe_code)] is lifted, each block with its // SAFETY: line, and CLAUDE.md § Standing decisions carries these words when the arm lands.
