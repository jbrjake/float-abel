---
id: fl-9rczhm8
title: "Make DESIGN.md's bitrep oracle claim true: check Exact::to_f64 against bitrep::SumF64::value on random multisets, or say bitrep checks Wide"
status: done
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
- 2026-11-25T00:13Z open→doing — claimed by claude (4aebe618-581f-48ef-9ee8-943721730ef9)
- 2026-11-25T00:18Z doing→done — verify exit 0 @ e2caba7+2

## comments
- 2026-11-25T00:18Z [claude (4aebe618-581f-48ef-9ee8-943721730ef9)] 2026-11-24 at e2caba7, the recommended path: `wide_and_exact_match_bitrep_on_random_multisets` in tests/wide.rs (the old bitrep test, renamed) folds each of its 300 multisets into an `Exact` beside the `Wide`, split at the same cut and merged, and asserts `exact.to_f64()` bit-equal to `theirs.value()` (bitrep 0.5.1). The draws are unchanged, so a third of the multisets come from `rng.any()` with NaN and ±∞ in; the claim's 'finite' is gone from DESIGN.md with them. 299 of 300 leave `Exact` in its heap form (the other is the one-value multiset at round 44), counted and asserted at nine in ten. Watched failing on two deliberate breaks in the test's own fold: the last value left out of the `Exact` fold fails the read at round 32, and the count inverted fails at 1 of 300. DESIGN.md § Verification's cross-tier row gives the bitrep check its own count, 300, beside the tiers' 10⁶; § Reads and rounding's sentence stands as written. Under miri the test runs three rounds, all promoted, and takes 24 s of wall where it took 5 s (aarch64 macOS, nightly 11177f223): a data point for fl-wpgpd4q.
