---
id: fl-mewakns
title: "Poison the last entry, not index 2999, in the binned product refusal test so it runs under miri"
status: done
category: crate/hardening
relates: [fl-jphvxrs]
verify: lacks tests/binned_product.rs /poisoned_lhs\[2999\]/
docs:
  - docs/DESIGN.md#hardening-sp-hardening
seq: 58
created: 2026-10-20T18:03Z
---

Under miri, `binned_product_deposit_products_is_the_pairwise_fold_or_nothing` in tests/binned_product.rs builds its two slices with `rounds(3000)`, 30 entries, and then poisons index 2999 of each (`poisoned_lhs[2999] = pow2(1000)`), so the test panics with "index out of bounds: the len is 30 but the index is 2999" (run 39463529492, miri job 118768785023, the first ci run whose miri step reached the end of the suite: 135 passed, 1 failed, 39 ignored in 3856 s). Natively the slices are 3,000 long and the index is the last entry. Poison the last entry by the slices' own length (`let last = lhs.len() - 1`), confirm the test passes under `cargo +nightly miri test --test main binned_product_deposit_products_is_the_pairwise` here and natively, then watch the miri job go green on the pushed commit and record the run id.

## log
- 2026-10-20T18:03Z created
- 2026-10-20T18:03Z open→doing — claimed by claude (30b8d01c-d5ec-456d-a066-16edcae1c161)
- 2026-10-20T18:49Z doing→done — verify exit 0 @ 1aa76c5+2

## comments
- 2026-10-20T18:49Z [claude (30b8d01c-d5ec-456d-a066-16edcae1c161)] Green on the pushed commit: run 39471909369 on 68415e0 (c385a17 is the fix), miri job 118797851505, the `cargo miri test --locked --test main` step 38m55s (18:08:58 to 18:47:53): 136 passed, 0 failed, 39 ignored, 3855 s in the harness, and the miri cache saved for the first time (miri-dc0215dc…). The whole run is the first green `ci` on main, every job included. The test itself ran under miri in 4 s here.
