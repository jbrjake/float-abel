---
id: fl-dchdzd4
title: "Close the limb cap under retraction at its edge — a heap value whose finite part spans exactly EXACT_MAX_LIMBS limbs decodes but cannot be subtracted from itself or multiplied by 1, because big::add and big::mul refuse on the operands' spans plus a carry bit, one bit past what the result needs"
status: open
category: crate/types
labels: [reproducible-floats]
discovered-from: fl-mrdqpay
verify: run cargo test package=float-abel exact_cap_closed
docs:
  - docs/DESIGN.md#exactness-bounds-sp-bounds
  - docs/DESIGN.md#exact-sp-exact
created: 2026-10-16T13:51Z
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
