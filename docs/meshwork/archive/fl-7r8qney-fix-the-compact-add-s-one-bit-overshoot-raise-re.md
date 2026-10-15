---
id: fl-7r8qney
title: "Fix the compact add's one-bit overshoot — raise refuses a sum whose result spans 127 bits when the raised operand alone needs 128, so 2^127 + (−1) is PastCap::Width while Exact::try_from(&Wide) accepts the same value"
category: crate/types
discovered-from: fl-h41w0tr
verify: "all(contains tests/exact_cap.rs /exact_cap_a_sum_that_spans_exactly_the_width_fits/, run cargo test package=float-abel exact_cap_)"
seq: 20
labels: [bug, reproducible-floats]
docs:
  - docs/DESIGN.md#exact-sp-exact
  - docs/OVERFLOW.md#what-fits
status: done
created: 2026-10-15T15:15Z
---

Found in fl-h41w0tr's spike. `raise` in src/exact/compact.rs returns `Overflow::NeedsWide` when
`bits + by > 127`, before the add, so an operand that would need exactly 128 bits at the aligned
exponent is refused even when the other operand cancels it back under 2¹²⁷. Run on 2026-10-15
(`repro`, attached to fl-h41w0tr): `Exact::from(2f64.powi(127)).checked_add(&Exact::from(-1.0))`
is `Err(PastCap::Width)`, and `Exact::try_from(&(Wide::from(2f64.powi(127)) + Wide::from(-1.0)))`
is `Ok(1.7014118346046923e38)`: 2¹²⁷ − 1 is 127 one-bits, inside the width. `2^127 + 1` is refused
by both routes, as it should be.

Only a sum whose raised operand overshoots by exactly one bit and whose result lies under 2¹²⁷ is
affected; an overshoot of two or more bits cannot cancel back inside the width. On the 32-byte
word it is a spurious cap; with the heap arm (fl-heap) it becomes a spurious promotion, an
allocation and a demotion for a result that fit, so it is worth fixing either way. The fix: when
`bits + by == 128`, raise by `by − 1` and add that operand twice (each add is inside `i128`, and
an overflow on either is a true `NeedsWide`), or add in a 129-bit intermediate. Red first: a test
named `exact_cap_a_sum_that_spans_exactly_the_width_fits` in tests/exact_cap.rs that asserts
`checked_add` agrees with `Exact::try_from(&Wide)` on `2^127 + (−1)`, on `−2^127 + 1`, and on a
sweep of `±2^k ± 1` and `±(2^k + 2^j) ∓ 2^j` across the boundary; then the fix; then the
`a + b − b == a` fuzz target still green. Independent of the heap work; can land before it.

## log
- 2026-10-15T15:15Z created
- 2026-10-15T16:25Z open→doing — claimed by claude (3f239526-744d-4f69-8242-cd45409a169a)
- 2026-10-15T16:35Z doing→done — verify exit 0 @ 1c76193+2

## comments
- 2026-10-15T16:35Z [claude (3f239526-744d-4f69-8242-cd45409a169a)] Fixed, with the sibling case the test found. The compact add now refuses a sum iff the sum's own canonical magnitude reaches 2^127, which is what `Exact::try_from(&Wide)` accepts, on two cold paths beside the unchanged common one in `Compact::checked_add`: a raised operand that needs exactly 128 bits at the aligned exponent is raised by one less and added twice (each half inside the width; an overflow on either add is a sum past it whichever way it is formed), so `2^127 + (−1)` is the widest odd word; and two odd mantissas at one exponent whose `i128` sum overflows (or lands on `i128::MIN`) go through `carried_at_one_exponent`, since their sum is even and its trailing zeros bring it back inside the width, so `(2^127 − 1) + 1` is `1 · 2^127` and the widest odd word plus itself is its double. The second case was the compact unit test's documented "aligned frame" refusal, written when the heap tier's demotion was expected to bring the one-bit form back; with the add deciding on the sum, both routes land one form. A raised operand two or more bits over cannot cancel back, and a raised operand makes the sum odd, so a carry there stays refused.
  
  Red first: `exact_cap_a_sum_that_spans_exactly_the_width_fits` failed on the unfixed add at the found case, then at `(2^127 − 1) + 1` after the first half of the fix. It asserts `checked_add`, the swapped add and `checked_sub` of the negation agree with the `Wide` route on `2^127 + (−1)`, `−2^127 + 1`, the widest odd word plus `2^k + 1` for every k ≤ 126 both ways (fits for k ≥ 1, refused for k = 0 where the addend is even and raised), ±2^k ± 1 for k in 120..=130, and ±(2^k + 2^j) ∓ (2^j + 1) across the boundary, 402 pairs counted. `compact_add_detects_promotion_at_the_127_bit_span` now pins the fixed forms by their storage parts. `cargo test` green (36 lib, 130 integration, 4 doc), clippy clean, `cargo +nightly fuzz run sub_roundtrip` and `sum_vs_oracle` a minute each clean (4.0M and 2.1M runs). docs/OVERFLOW.md § What fits and DESIGN.md § Exact say the cap is on the sum's span. The common path's instructions are unchanged, so the compact add's bench row is not re-taken.
