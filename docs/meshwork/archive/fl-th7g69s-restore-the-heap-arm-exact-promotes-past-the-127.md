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
status: done
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
- 2026-10-15T16:53Z open→doing — claimed by claude (fa2db8b2-1866-4c7c-86e2-74f6ae52f278)
- 2026-10-15T17:47Z doing→done — verify exit 0 @ 09c92b5+3

## comments
- 2026-10-15T15:36Z [claude (6cbb3399-1134-4aff-94e4-f8c1137cf60f)] Owner, 2026-10-15, in session, on the layout: "yes i'm okay with one more unsafe to save that many bits on every value". The 32-byte niche form is ruled; src/exact/heap.rs is the one module beside src/simd/ where #![deny(unsafe_code)] is lifted, each block with its // SAFETY: line, and CLAUDE.md § Standing decisions carries these words when the arm lands.
- 2026-10-15T15:38Z [claude (6cbb3399-1134-4aff-94e4-f8c1137cf60f)] Owner, 2026-10-15, in session, confirming the derived design this task builds (fl-t154dsp's readback points 1, 2, 3 and 5: the limb cap as the one residual panic, the banded fast path, dyadic special amounts in the heap form, the compact-only no-alloc build): "yes confirmed". Nothing in the body above is unratified.
- 2026-10-15T17:47Z [claude (fa2db8b2-1866-4c7c-86e2-74f6ae52f278)] Findings: built at 09c92b5, the commit gate and the push gate green (scripts/regression.sh:
  clippy on every configuration, rustdoc, the suite with the ignored tests, the release every_build
  tests, noalloc, golden, the MSRV build, the dbsp tests and example on 1.99.0, both cargo-deny
  configs), and `cargo +nightly miri test` green over `exact_heap_` (19 tests, 433 s), the lib's
  unit tests and the noalloc member, no leak reported.
  
  The shape, as the layout measurement settled it: the word is unchanged for the compact form; a
  heap word has `lo` = the block's capacity in limbs doubled (even, nonzero), `hi` the pointer, the
  four trailing fields zero; the block is one `Vec<u64>` allocation whose first four limbs pack
  (sign, exponent, limb count) for the finite part and the three amounts, then the four normalised
  runs. `Drop` rebuilds the Vec by its capacity, `Clone` copies the used prefix, `Eq` and `Hash`
  read the used prefix, `Ord` walks aligned 64-bit windows without allocating. Promotion goes
  through `Overflow::NeedsWide | Overflow::Count` from the compact paths into
  `value::Value::{add, mul, mul_i64}` over borrowed views (`big::View`) and lands through
  `Exact::land`, which writes the compact word when the finite part fits 127 bits and every amount
  is an `i32` count, else takes the result's buffer as the block (allocated with the header's room
  in front, so a promoted op is one allocation and one free). The limb cap is decided on the
  operands' spans before any allocation. src/exact/heap.rs is the one new `unsafe` module (five
  blocks); big.rs, value.rs, read.rs and form.rs are safe.
  
  API points read back here, built under the stated assumption and reversible: `specials()` now
  returns `Option<(i64, i64, i64)>`, `None` for a dyadic amount or one past `i64`, both only in the
  heap form (the old `(i32, i32, i32)` had no honest answer for `2.5 · NaN`); `PastCap` keeps
  `Width` and `Count` on every feature set, because removing variants with `alloc` on would make
  the feature non-additive, and the doc says the no-allocator build alone returns them;
  `PastCap::Limbs` and `EXACT_MAX_LIMBS` (a `pub const`, raised by editing it) are new;
  `is_finite` is no longer `const fn`.
  
  Day-one limits, each the wire task's (fl-mrdqpay): `to_bytes` panics by name on a heap value;
  serde carries the heap form under the tag `0x04` (the body is in the commit message), which
  `from_bytes` refuses and the serde decoder validates (cap before allocation, normalised runs, a
  value that fits the compact form refused as `WireError::Form`; `WireError::Limbs` and `Form` are
  new variants); dbsp's archived form stays the compact word and a heap word reaching it panics by
  name, so a promoted weight lives in dbsp's in-memory batches (tested); the golden transcript
  records a heap value by its reads under tag 0x24 and was re-blessed on aarch64 macOS
  (56de810b…, BYTES 65,385,705; the other six targets are CI's).
  
  Not done: the A/B sitting against the budget, because the host's load stayed at 4.5–7 for the
  hour the sitting waited; it is its own task now, with the control binary saved and the script
  written (search `heap arm's budget`). Also left: in-place add when a heap self's block has room,
  a `trybuild` pin that `Exact` is not `Copy`, and CLAUDE.md is over the baseline's 4 KB (it was
  before this task).
