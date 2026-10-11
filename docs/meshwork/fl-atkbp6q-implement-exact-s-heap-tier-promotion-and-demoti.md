---
id: fl-atkbp6q
title: "Implement Exact's heap tier — promotion and demotion, the special-value algebra, Mul<Exact> and Mul<i64>, and the limb cap"
category: crate/types
labels: [reproducible-floats, rf-critical-path]
docs: [docs/DESIGN.md#exact-sp-exact, docs/DESIGN.md#nan-and--sp-specials, docs/DESIGN.md#exactness-bounds-sp-bounds, docs/DESIGN.md#operations-and-conversions-sp-ops]
needs: [fl-dwrey99]
seq: 50
verify: run cargo test package=float-abel exact_
status: open
created: 2026-10-11T15:02Z
covers:
  - ref: docs/DESIGN.md#sp-exact
    sha: 37e69a8f5bc8790986626d0213167f858411aeb661df18d79b1bdcc02ffcca54
  - ref: docs/DESIGN.md#sp-specials
    sha: addd4f3bbbdf9d2ff253af0022364ac7d0702feb43c1a014a14161878ec9de0f
  - ref: docs/DESIGN.md#sp-bounds
    sha: 943ce56af93ae6ee0abf6c13ab9323ade26f1afc1c47149b7ae4b45bbc2a8fcb
  - ref: docs/DESIGN.md#sp-ops
    sha: cf4faaa1f10025e5cd89f4a8883ed54ca023db8b7d4aa0472b24bc9fdf08e8ea
---
`src/exact/wide.rs` and `src/exact/mod.rs` per DESIGN.md § Exact, § NaN and ±∞ and § Exactness
bounds. `WideExact { sign, mant: Vec<u64>, exp, specials: [Dyadic; 3] }`, normalised. Canonical form
is the smallest tier that fits. Multiplication follows the monoid-algebra product. `EXACT_MAX_LIMBS`
is a `const` with a named panic past it, documented (DESIGN.md § Open questions).

Tests `exact_*`:
- the laws across tiers: a value built through the heap tier equals the same value built through
  `Compact`, structurally;
- distributivity on 10⁵ random triples, specials included;
- `0 · ∞` and `0 · NaN` project `0` (the documented divergence), while `(+∞)·(−∞)` projects `−∞` and
  `(−∞)·(−∞)` projects `+∞`, each asserted so that a change is deliberate;
- the history `1e30`, `1e-10`, `−1e30` reads `1e-10` exactly;
- an adversarial fold over magnitudes 2^±300 stays exact.

## log
- 2026-10-11T15:02Z created
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-exact @37e69a8f5bc8
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-specials @addd4f3bbbdf
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-bounds @943ce56af93a
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-ops @cf4faaa1f100
