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
handoff: |
  Next session builds the heap arm here. Nothing is unratified: the
  owner's words are in the body
  and the two comments, and the closed layout measurement (search `Layout
  findings`) settled the block shape and
  the budget: shape B, the whole heap value in one block behind the
  pointer, and the one-worker
  same-scale `dd_sum` rows within 1.12× the Copy word typed and 1.20×
  serialized, measured in one
  sitting against a control build (the A/B recipe is in the
  build-environment memory). Read that
  comment and docs/OVERFLOW.md § Cost first; the probe in its attachments
  (`probe-src-bin-cost-niche.rs`
  and `-big.rs`) is the reference implementation of the word: `is_heap` as
  `lo & 1 == 0 && lo != 0`,
  `parts()` reading the block's header, `heap()` allocating one
  `Box<[u64]>` and `Drop` freeing it
  by the length in `lo`, `land()` demoting iff the bit length is at most
  127, and the compact fast
  paths the budget assumes: `ptr::write` over a compact self in
  `plus_equals` and `negate`,
  `mem::forget` of a compact by-value self in `multiply`, `clone_from` as
  a copy when both sides
  are compact, `is_zero` as the bitwise zero word, `eq` by the word first.
  Its heap path allocates
  a fresh block per result; the real arm should add in place when a heap
  self's block already holds
  the result's length, which is where the promoting bands' 1.4–2.0×
  over the banded idiom can come
  down.
  
  What is proven in the tree: `Compact::checked_add` (the closed boundary
  fix, search `carried_at_one_exponent`) now refuses a sum iff its
  canonical span exceeds 127 bits, the same rule `Exact::try_from(&Wide)`
  applies, so a
  `NeedsWide` from it is exactly "this value is not compact": promote on
  it, and demotion after a
  heap op lands compact iff the bit length is at most 127, with no
  spurious promotion at the
  boundary. `Overflow::NeedsWide`'s doc already says it is the heap tier's
  job.
  
  Where `Copy` is pinned today, all of which the arm changes:
  `#[derive(Clone, Copy, ...)]` on
  `Exact` in src/exact/mod.rs (and `rkyv::Archive` behind `dbsp`, which
  cannot derive over a raw
  pointer: the archived form becomes fl-mrdqpay's, so behind `dbsp` the
  arm needs a hand-written
  `Archive`, or the feature waits on the wire task); by-value copies in
  src/exact/ops.rs (`-*rhs`,
  `*rhs * self`, `*self += &rhs`), ring.rs (`product(*self, *rhs)`),
  convert.rs, differential.rs
  (`*self = -*self`), dbsp.rs; `Compact` stays `Copy`. tests/layout.rs
  pins `size_of == 32` and
  `align_of == 8`, which hold. src/lib.rs has `#![deny(unsafe_code)]`
  crate-wide; src/simd/ lifts
  it at module level, and src/exact/heap.rs does the same with
  `#![allow(unsafe_code)]` and a
  `// SAFETY:` on every block (clippy `undocumented_unsafe_blocks` is
  deny). tests/posture.rs
  checks the feature set, not the unsafe posture, so the "unsafe-free with
  simd off" sentence in
  CLAUDE.md § Standing decisions is the one to rewrite in the owner's
  words ("yes i'm okay with one
  more unsafe to save that many bits on every value"). tests/noalloc is a
  workspace member building
  the lib with no features (scripts/regression.sh runs it, CI runs it
  under miri): the `alloc`
  feature gates heap.rs and the promotion arm, `std` implies it, the self
  dev-dependency lists it,
  and noalloc keeps the compact-only word with the width cap and the
  checked API.
  
  One API point to settle before the dyadic amounts land, and to read back
  to the owner if it
  changes a public signature: `Exact::specials()` returns `(i32, i32,
  i32)` and `PastCap::Count`
  exists because the compact form's amounts are integer counts. With
  dyadic amounts in the heap
  form a value's amounts may be non-integer (`2.5 · NaN`), so the read
  side needs a shape: keep
  `specials()` for the integer case and add an amounts accessor, or return
  the amounts as a small
  dyadic type. `PastCap::Count` goes (the body says so); `Exact::narrow`
  goes with it. The
  projection rule (nonzero NaN amount reads NaN; one infinity reads by the
  sign of its amount) is
  unchanged, since only the sign and nonzero-ness of an amount matter at
  read.
  
  Suggested order, each step red first with `exact_heap_*` tests in
  tests/exact_heap.rs (add
  `mod exact_heap;` to tests/main.rs, or scripts/check-test-topics.sh
  fails): the `alloc` feature
  and heap.rs with the block, the form switch, `Clone`, `Drop`, `Eq`,
  `Ord`, `Hash`, `is_compact()`;
  promotion and demotion through `checked_add`'s `NeedsWide` and the five
  histories; `Mul<i64>` and
  `Neg`; `Mul<Exact>` with the dyadic amounts, the schoolbook multiply and
  `EXACT_MAX_LIMBS` with
  its named panic and checked return; the serde encoding of a heap value
  (fl-mrdqpay owns the
  design, but differential's exchange needs something on day one: a tagged
  byte string whose
  decoder refuses a heap body that fits the compact form); the noalloc
  build; miri over promote,
  clone, drop and demote; then the A/B sitting against the budget with the
  control's `dd_sum`
  binary saved first; then DESIGN.md's sections, CLAUDE.md § Standing
  decisions, and the ask to
  portfolio. Keep src files under 500 lines: heap.rs for the form, the
  multiply in its own file.
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
niche, `lo` even and nonzero carrying the limb count, `hi` the pointer to one allocation holding a header and the limbs (fl-nhcdk92's shape B: a
header of sign, exponent and length for the finite part and for each of the three amounts, then
the limb runs; the word's four trailing fields zero, which the decoder and `Eq` demand).
`Exact` is not `Copy`; `Clone` is a copy on the compact form and an allocation on the heap form,
`Drop` frees the block. `src/exact/heap.rs` is the one module beside `src/simd/` where
`#![deny(unsafe_code)]` is lifted, each block with its `// SAFETY:` line stating the invariant
(the tag bit decides which form; a heap `hi` is a pointer this module allocated with the length
in `lo`; no other code reads the fields), `cargo +nightly miri test` covers it, and CLAUDE.md
§ Standing decisions says so in the owner's words. The one-worker same-scale `dd_sum` rows
must land within fl-nhcdk92's budget, 1.12× the Copy word typed and 1.20× serialized, measured in
one sitting against a control build of the Copy word; the probe's compact fast paths (a write
over a compact self without drop glue, `clone_from` as a copy, a bitwise `is_zero`) are what that
budget assumes.

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
- 2026-10-15T16:37Z handoff by claude (3f239526-744d-4f69-8242-cd45409a169a)
- 2026-10-15T16:37Z handoff by claude (3f239526-744d-4f69-8242-cd45409a169a)

## comments
- 2026-10-15T15:36Z [claude (6cbb3399-1134-4aff-94e4-f8c1137cf60f)] Owner, 2026-10-15, in session, on the layout: "yes i'm okay with one more unsafe to save that many bits on every value". The 32-byte niche form is ruled; src/exact/heap.rs is the one module beside src/simd/ where #![deny(unsafe_code)] is lifted, each block with its // SAFETY: line, and CLAUDE.md § Standing decisions carries these words when the arm lands.
- 2026-10-15T15:38Z [claude (6cbb3399-1134-4aff-94e4-f8c1137cf60f)] Owner, 2026-10-15, in session, confirming the derived design this task builds (fl-t154dsp's readback points 1, 2, 3 and 5: the limb cap as the one residual panic, the banded fast path, dyadic special amounts in the heap form, the compact-only no-alloc build): "yes confirmed". Nothing in the body above is unratified.
