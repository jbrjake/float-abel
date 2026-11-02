---
id: fl-9rczhm8
title: "Make DESIGN.md's bitrep oracle claim true: check Exact::to_f64 against bitrep::SumF64::value on random multisets, or say bitrep checks Wide"
status: open
category: crate/hardening
verify: "all(contains tests/wide.rs /exact.to_f64/, run cargo test target=main bitrep)"
docs:
  - docs/DESIGN.md#verification
seq: 85
created: 2026-10-24T16:33Z
parent: fl-b9qtzty
---

DESIGN.md § Verification's cross-tier row claims `Exact::to_f64 == bitrep::SumF64::value` on
random finite multisets, under a count of 10⁶, and § Reads and rounding says "bitrep itself is a
dev-dependency differential oracle for `Exact::to_f64` on random multisets". No test does that.

What the tests do:

- The only test that calls bitrep is `wide_limbs_and_value_match_bitrep_on_random_multisets` in
  `tests/wide.rs`. It folds `Wide`, not `Exact`, over `rounds(300)` multisets, a third of them
  from `rng.any()`, so specials are in and the set is not finite-only.
- The 10⁶ is `rounding_tiers_agree_on_a_million_random_multisets` in `tests/rounding.rs`, which
  crosses `Fixed256`, `Wide` and `Exact` inside `Fixed256`'s window and never calls bitrep.

So `Exact` meets bitrep only transitively, and only inside that window. Nothing checks an `Exact`
read against an independent implementation across the full double range or in the heap form.

Recommended, since DESIGN.md is the spec: make the claim true. Fold the same multisets into
`Exact` in the bitrep test (renamed so the name says both, keeping the `wide_` prefix the file's
test filter uses), assert `to_f64` bit-equal to `theirs.value()`, and count the multisets that put
`Exact` in its heap form so the leg is shown to reach it. Watch the new assertion fail on a
deliberate break before trusting it. Then give the bitrep clause its own count in the row, since
the 10⁶ belongs to the tier crossing.

The alternative, if the owner prefers it: drop the `Exact` claim from both places and say bitrep
checks `Wide`. That needs `--verify` replaced, since this one is written for the test.

## log
- 2026-10-24T16:33Z created
