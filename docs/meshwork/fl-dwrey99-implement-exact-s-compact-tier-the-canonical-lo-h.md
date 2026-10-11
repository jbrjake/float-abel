---
id: fl-dwrey99
title: "Implement Exact's compact tier — the canonical (lo, hi, exp) dyadic, add with promotion detection, negate, Ord, Hash and to_f64"
category: crate/types
labels: [reproducible-floats, rf-critical-path]
docs: [docs/DESIGN.md#exact-sp-exact]
needs: [fl-1dw33g3]
seq: 40
verify: run cargo test package=float-abel compact_
status: open
created: 2026-10-11T15:02Z
covers:
  - ref: docs/DESIGN.md#sp-exact
    sha: 37e69a8f5bc8790986626d0213167f858411aeb661df18d79b1bdcc02ffcca54
---
`src/exact/compact.rs` per DESIGN.md § Exact. Canonical form: mantissa odd, or the zero form. `add`
returns `Err(NeedsWide)` when the aligned span exceeds 127 bits or a special is present.

Tests `compact_*`:
- the group laws on same-scale inputs;
- `a + b − b == a` on 10⁶ random pairs;
- `size_of::<Exact>() == 24` as a `const` assertion. That is the measured layout; if it moves, the
  tier moved.

## log
- 2026-10-11T15:02Z created
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-exact @37e69a8f5bc8
