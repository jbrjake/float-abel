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
handoff: |
  Exact's compact tier is next, and the tree now holds both fixed-width
  carriers to borrow from.
  
  Shape (DESIGN.md § Exact): `enum Exact { Compact { lo: u64, hi: u64,
  exp: i32 }, Wide(Box<WideExact>) }`
  with `size_of::<Exact>() == 24` as a `const` assertion. The 24 comes
  from rustc's field reordering:
  the tag sits at offset 0, `exp` at 4, `lo` at 8 and `hi` at 16, and the
  `Box` arm fits after the
  tag. `(i128, i32)` would be 32 because i128 forces 16-byte alignment.
  The heap arm is fl-atkbp6q's;
  this task lays out `src/exact/mod.rs` and `src/exact/compact.rs` (the
  heap task names
  `src/exact/wide.rs`, a different path from the top-level `wide` module).
  Decide, and read back to
  the owner, what `Exact` does without `alloc` when the compact add
  returns `NeedsWide`: a named
  panic mirrors the limb-cap decision, or the compact task exposes only
  `Compact` and `Exact`'s
  public surface lands with the heap tier. `From<f64>` must end up total,
  and a special input is
  `NeedsWide` on the compact tier by design, so the answer shapes this
  task's API.
  
  Reuse, not rewrite:
  - `src/ieee.rs`: `decompose(f64) -> Option<Dyadic>` is already the odd
  dyadic the compact form
  wants: `m = ±mant`, `exp` as given; zero is `{0, 0, 0}`. `special(x)`
  names the non-finite case.
  - `src/round.rs`: `round(negative, &[lo, hi], exp, false, F64)` is
  `to_f64` on the two-limb
  magnitude, `try_to_f64` is its inexact flag, and `div_f64(negative,
  &mag, exp, n)` is `div_f64`.
  Both `Fixed` and `Wide` read this way; see `Wide::read` and
  `Wide::div_f64` in `src/wide.rs`.
  - `src/limbs.rs` is slice-generic, but at two limbs plain `i128` is
  clearer: `checked_add`,
  `checked_shl` bounds and `trailing_zeros` do the add, the span check and
  the canonicalisation.
  - `src/fixed.rs` and `src/wide.rs` are the trait-surface pattern: hand
  `Default` and `Debug`
  (`fixed::Hex` prints a limb), `Ord` numeric, `Mul<i64>`/`Mul<isize>`,
  `Sum` for values and refs.
  
  The add: align to the smaller exponent, refuse with `NeedsWide` when the
  shifted span would pass
  127 bits (so the sum stays inside i128 and `i128::MIN` never appears,
  which keeps negate total),
  then canonicalise by moving trailing zeros into `exp` and mapping a zero
  sum to the zero form.
  `Ord` without overflow: sign first, then `bit_length(|m|) + exp`, then
  align only when those tie,
  which shifts by at most the bit-length difference. `Eq` and `Hash`
  derive on the canonical form.
  
  Tests `compact_*` in `tests/compact.rs` (add `mod compact;` to
  tests/main.rs, or
  scripts/check-test-topics.sh fails): the laws on same-scale inputs
  (`Rng::on_grid(-20, 20)` from
  `tests/support.rs` keeps a pair's span under 100 bits), `a + b - b == a`
  on 10^6 random pairs,
  the size assertion, promotion detection (`1e300 + 1e-300` is
  `NeedsWide`, `0.1 + 0.2` is not, a
  special is), canonical equality (`0.5 + 0.25 == 0.75` structurally and
  under `hash`), order
  against `partial_cmp` over the whole finite range (`Rng::finite`), and
  the reads against IEEE
  `a + b` and against `Wide` on the same pair, which costs one line now
  that `Wide` exists. The
  `powi` trap stands: `2f64.powi(e)` for `e <= -1024` returns 0; use
  `f64::from_bits`.
  
  Proven so far: `Fixed` and `Wide` against the hardware oracles, bitrep's
  vectors and bitrep itself;
  `round` on ties, subnormals, overflow and sticky; `div_window` against
  IEEE division. Gates:
  `./scripts/smoke.sh` and `./scripts/regression.sh` green at 1a182d4
  (MSRV build and cargo-deny
  included). `CARGO_TARGET_DIR` is set outside ~/Documents by the repo's
  `.claude/settings.local.json`
  for Claude Code sessions; export it by hand elsewhere.
  
  Owner question pending, raised in the Wide task's closing comment (in
  the archive):
  `Wide::deposit_product` refuses a product below 2^-1074 or at 2^1037 as
  `OutOfWindow`, where
  DESIGN.md § Operations and conversions reads as if only `Fixed`
  refuses. The tier conversions are
  filed as their own task, fl-te1748m.
relates: [fl-te1748m]
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
- 2026-10-11T16:32Z handoff by claude (ffdbc5f1-8414-4a1d-8060-cf0fb191411f)
- 2026-10-11T16:33Z handoff by claude (ffdbc5f1-8414-4a1d-8060-cf0fb191411f)
