---
id: fl-dchdzd4
title: "Close the limb cap under retraction at its edge — a heap value whose finite part spans exactly EXACT_MAX_LIMBS limbs decodes but cannot be subtracted from itself or multiplied by 1, because big::add and big::mul refuse on the operands' spans plus a carry bit, one bit past what the result needs"
status: done
category: crate/types
labels: [reproducible-floats]
discovered-from: fl-mrdqpay
verify: run cargo test package=float-abel exact_cap_closed
docs:
  - docs/DESIGN.md#exactness-bounds-sp-bounds
  - docs/DESIGN.md#exact-sp-exact
created: 2026-10-16T13:51Z
covers:
  - ref: docs/DESIGN.md#sp-bounds
    sha: b209d3ddedfdb15d6c4e7f595194175bf47c4e1fb5c6c166f1911f415ea82e63
---

Found while pinning the heap encoding (fl-mrdqpay): `tests/wire_heap.rs` decodes a body whose
four runs each hold `EXACT_MAX_LIMBS` limbs of `u64::MAX` (the decoder's cap is `len >
EXACT_MAX_LIMBS`, so exactly the cap is a value), and on that value `checked_mul_i64(1)` returns
`Err(PastCap::Limbs)` and `v - v` panics with the same cap. The cause is the pre-allocation
check in `src/exact/big.rs`: `add` refuses when `top − low + 2 > CAP_BITS` (a bit for the carry
on top of the aligned span) and `mul` when `lhs.bits() + rhs.bits() > CAP_BITS`, both one bit
more than the result can need when no carry happens, and a sum can land with exactly `CAP_BITS`
bits (a carry out of a `CAP_BITS − 1` span), so such values are reachable through the operations
too, not only by decoding. The group is then not closed under retraction at the cap's edge: a
weight at the edge cannot be negated and added back, which `fl-x8fwfe3`'s closure law would
catch if its strategy reached the edge.

Fix the check without an allocation before the cap is decided (the arm's rule): for `add`, allow
the aligned span plus one when the carry cannot occur, or compute into `cap + 1` limbs only when
the span is exactly at the cap and refuse after normalisation (one extra limb, allocated only at
the edge); for `mul`, refuse on `lhs.bits() + rhs.bits() − 1 > CAP_BITS` and let `check()` refuse
the product that actually spans one more. Either way `v − v` and `v × 1` hold for every `v` the
operations or the decoder produce, and `v + w` is refused only when its result would span more
than the cap.

Tests `exact_cap_closed_*` in tests/exact_cap.rs: the decoded at-cap value (reuse
tests/wire_heap.rs's body) under `checked_mul_i64(1)`, `checked_add(&ZERO)`, `checked_sub(&v)`
and `v - v`; a value built through the operations with exactly `CAP_BITS` bits, retracted; and
the refusal one bit past, by name. Then widen tests/wire_heap.rs's at-cap block back to the
operations' checks it skips today.

## log
- 2026-10-16T13:51Z created
- 2026-10-16T18:15Z open→doing — claimed by claude (2ba3bd95-5552-4160-8df2-82be1495b54b)
- 2026-10-16T18:24Z cover docs/DESIGN.md#sp-bounds @b209d3ddedfd
- 2026-10-16T18:26Z doing→done — verify exit 0 @ 6c04216+2

## comments
- 2026-10-16T18:26Z [claude (2ba3bd95-5552-4160-8df2-82be1495b54b)] Fixed at 6c04216. The rule in src/exact/big.rs: `add` refuses before allocating when the room the result can need (the aligned span plus a carry bit for like signs, the span alone for unlike signs) is more than CAP_BITS + 1, and otherwise computes into ceil(room/64) ≤ cap + 1 limbs and lets `check` refuse after normalisation; `mul` refuses when p + q − 1 > CAP_BITS, the narrower product already past, and otherwise allocates lhs.len + rhs.len ≤ cap + 1 limbs. Why that is exact and not a loosening: for an aligned span of CAP_BITS + 2 or more, the operand holding the low bit spans at most the cap, so the other's top bit sits at least two above it; the result keeps that top bit or the one below (a borrow) and has the low bit set, so it spans at least CAP_BITS + 1 bits and the refusal on the spans is right. The two cases that need the result are a span of CAP_BITS with a carry (like signs: 2^C − 1 plus 1 is 2^C, one limb) and CAP_BITS + 1 with a borrow (unlike signs, the (v + v) − v shape, where the double of a cap-wide value is cap-wide at exponent 1). The edge costs one limb of allocation, HEADER + cap + 1, which `land` takes as the block as it takes any. Tests: five `exact_cap_closed_*` in tests/exact_cap.rs, the sweep being every pairwise sum, difference and product of eleven edge shapes, 363 operations, against num-bigint: held iff the oracle's span fits, every held sum retracted to its left operand, every refusal `PastCap::Limbs`; the decoded at-cap body is now `at_cap_body()` in tests/wire_heap.rs, whose at-cap block runs `checked_mul_i64(1)` and `v − v` as the accepted bodies do. Verified this session: `cargo test` 231 green (36 lib, 190 main, 5 noalloc), clippy `-D warnings` and rustdoc clean, the commit gate green on the commit; the push gate was not run. DESIGN.md § Exactness bounds states the rule and is pinned to this task at b209d3ddedfd; the spec audit's 34 re-open candidates on closed tasks predate this edit. Left open, filed as fl-cwwnyas: tests/laws.rs's `exact_full()` never draws the edge, so the closure law could not have caught this.
