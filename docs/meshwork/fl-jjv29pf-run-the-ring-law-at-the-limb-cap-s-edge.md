---
id: fl-jjv29pf
title: Run the ring law at the limb cap's edge
status: doing
category: crate/types
labels: [reproducible-floats]
discovered-from: fl-cwwnyas
relates: [fl-cwwnyas]
verify: run cargo test package=float-abel laws_ring_exact_at_the_cap
docs:
  - docs/DESIGN.md#the-laws-kit-sp-laws-kit
created: 2026-10-16T19:42Z
claimed-by: claude (4a8ded2c-6eb4-4638-a7f1-ee6da67e86ba)
---

tests/laws_cap.rs runs the closure and group laws at the limb cap's edge on `exact_at_the_cap()`,
a strategy at the closure check's budget (a weight under `2^(CAP_BITS − 5)`, multipliers inside
those five bits). The ring law is not run there: `check_ring` forms `(a + b)·k` and `(a − b)·k`,
one bit wider than a weight, so its multiplier budget is four bits, `|k| ≤ 16`, and the five-bit
`edge_int()` would pass the cap in some case.

Add `check_ring` on `exact_at_the_cap()` with a four-bit integer multiplier and with the same
integers as `Exact` multipliers, in tests named `laws_ring_exact_at_the_cap_*`, on `edge_runner()`
(32 cases, the file's `CASES`), so distributivity of a scalar and of an `Exact` multiply is checked
bit for bit on products that land on the cap's last bit, as the closure and group laws already are.
State the four-bit budget beside the five-bit one in the file's `HEADROOM` comment. Keep the sweep
inside the commit gate's budget: measure the test's wall time in the dev profile and record it in
the task's close comment.

## log
- 2026-10-16T19:42Z created
- 2026-10-20T15:22Z open→doing — claimed by claude (4a8ded2c-6eb4-4638-a7f1-ee6da67e86ba)
