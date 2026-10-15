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
status: open
created: 2026-10-15T15:15Z
---
Found in fl-h41w0tr's spike. `raise` in src/exact/compact.rs returns `Overflow::NeedsWide` when
`bits + by > 127`, before the add, so an operand that would need exactly 128 bits at the aligned
exponent is refused even when the other operand cancels it back under 2¹²⁷. Run this session
(`repro`, attached to fl-h41w0tr): `Exact::from(2f64.powi(127)).checked_add(&Exact::from(-1.0))`
is `Err(PastCap::Width)`, and `Exact::try_from(&(Wide::from(2f64.powi(127)) + Wide::from(-1.0)))`
is `Ok(1.7014118346046923e38)`: 2¹²⁷ − 1 is 127 one-bits, inside the width. `2^127 + 1` is refused
by both routes, as it should be.

Only a sum whose raised operand overshoots by exactly one bit and whose result lies under 2¹²⁷ is
affected; an overshoot of two or more bits cannot cancel back inside the width. The fix: when
`bits + by == 128`, raise by `by − 1` and add that operand twice (each add is inside `i128`, and
an overflow on either is a true `NeedsWide`), or add in a 129-bit intermediate. Red first: a test
named `exact_cap_a_sum_that_spans_exactly_the_width_fits` in tests/exact_cap.rs that asserts
`checked_add` agrees with `Exact::try_from(&Wide)` on `2^127 + (−1)`, on `−2^127 + 1`, and on a
sweep of `±2^k ± 1` and `±(2^k + 2^j) ∓ 2^j` across the boundary; then the fix; then the
`a + b − b == a` fuzz target still green.

## log
- 2026-10-15T15:15Z created
