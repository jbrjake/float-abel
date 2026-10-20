---
id: fl-sq2sv0n
title: "Give the miri job headroom: scale the cap-edge oracle cross under miri"
status: doing
category: crate/hardening
relates: [fl-jphvxrs, fl-mewakns]
verify: contains tests/exact_cap.rs /cfg(_attr)?\(miri/
docs:
  - docs/DESIGN.md#hardening-sp-hardening
seq: 62
created: 2026-10-20T18:05Z
claimed-by: claude (30b8d01c-d5ec-456d-a066-16edcae1c161)
---

The ci miri job's `cargo miri test --locked --test main` step first ran to the end of the suite on 2026-10-20 (run 39463529492, job 118768785023) and took 49m26s of its 60-minute ceiling (17:12:18 to 18:01:44, 175 tests, 135 passed, 39 ignored, the one failure being fl-mewakns). Gaps between consecutive test-completion lines in that log, a lower bound on the test named since the harness runs tests in parallel: `exact_cap::exact_cap_closed_sums_and_products_at_the_edge_agree_with_the_oracle` 961.6 s, `exact_heap_ring::exact_heap_ring_a_dot_product_of_mixed_pairs_matches_the_oracle_both_ways` 205.9 s, `wide_group::wide_folds_are_order_independent` 176.2 s, `exact_ring::exact_ring_mul_is_a_commutative_ring_with_integer_special_amounts` 82.1 s, then a tail of 50 to 65 s each (simd extend_from_slice, binned_product's truncation sweep, the conformance vectors, the merge-partition fold, the rounding slice). The edge test is not a `rounds()` sweep: it crosses every pair of the 11 cap-edge shapes in tests/exact_cap.rs's `edge_shapes()` through sum, difference and product against the BigInt oracle, each operand a thousand-odd limbs, which the interpreter runs some thousand times slower than the machine. Ten minutes of headroom on a shared runner is not enough; the fix is either a subset of the shapes under `cfg(miri)` (the first and last few, so the cap edge still crosses once), or `#[cfg_attr(miri, ignore = ...)]` on the edge test with the reason, as the laws kit is. Measure first: run the suite here with `cargo +nightly miri test --test main -- --test-threads=1` through the `date +%T` prefix loop in the build-environment memory so each test's own time is known, record the top ten in a comment, then scale, push and record the miri step's duration on the pushed commit.

## log
- 2026-10-20T18:05Z created
- 2026-10-20T18:49Z open→doing — claimed by claude (30b8d01c-d5ec-456d-a066-16edcae1c161)

## comments
- 2026-10-20T19:15Z [claude (30b8d01c-d5ec-456d-a066-16edcae1c161)] Measured here on 2026-10-20 at d43282d, `cargo +nightly miri test --test main -- --test-threads=1` with a clock prefix per line (miri-profile.sh in this session's scratchpad), M5 Max, nothing else of ours running: the suite is 1312 s over 163 non-ignored tests (136 passed, 39 ignored; 21m52s wall for the step). Each test's own time, top ten:
  
  - 400 s exact_cap::exact_cap_closed_sums_and_products_at_the_edge_agree_with_the_oracle
  - 110 s exact_heap_ring::exact_heap_ring_a_dot_product_of_mixed_pairs_matches_the_oracle_both_ways (already rounds(100_000), 1000 pairs under miri)
  - 80 s wide_group::wide_folds_are_order_independent (2000 draws the seed assertion pins, rounds(50) partitions)
  - 41 s exact_ring::exact_ring_mul_is_a_commutative_ring_with_integer_special_amounts
  - 29 s simd::simd_wide_extend_from_slice_lands_the_deposit_loops_bits_on_vectors_and_multisets
  - 28 s binned_product::binned_product_is_the_exact_products_truncated_below_the_window_and_the_grid
  - 27 s rounding::rounding_vectors_past_exacts_width_are_held_in_the_heap_form
  - 25 s wide::wide_reproduces_every_bitrep_conformance_vector
  - 24 s rounding::rounding_vector_file_covers_every_boundary
  - 24 s rounding::rounding_tiers_agree_on_a_slice_of_random_multisets
  
  So the cap-edge cross is 30% of the suite on its own and the only test not already through rounds(); the runner ran it about 2.4× slower (961 s lower bound). The cut: under cfg(miri) the cross runs over six of the eleven shapes (the two widest, the half-span one, 2^(C+64), 1 and 2^64 + 1), 108 ops for 363, with the identity loop still over all eleven; the count assertion follows n.
