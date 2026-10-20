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
