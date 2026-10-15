---
id: fl-p4dc5w5
title: "Measure the heap arm's budget in one quiet sitting — the one-worker same-scale dd_sum rows of the heap Exact at 09c92b5 against the Copy word's control binary, within 1.12× typed and 1.20× serialized, recorded in BENCH.md"
status: open
category: crate/measure
labels: [reproducible-floats]
discovered-from: fl-th7g69s
verify: contains BENCH.md /heap arm/
docs:
  - BENCH.md#the-end-to-end-sum
  - docs/OVERFLOW.md#cost
created: 2026-10-15T17:47Z
---

The control is the Copy word's dd_sum bench binary at 03d96e9, saved in ~/.cache/cargo-target/float-abel-control/release/deps/dd_sum-ab4b21425246350c; the variant is cargo bench --bench dd_sum --no-run in the main tree. Run both on --bench '1w' with CRITERION_HOME apart in one window with the one-minute load under 3 and no other cargo (the build-environment memory has the recipe and the sitting script's shape); the budget is fl-nhcdk92's Layout findings comment: within 1.12× the control typed and 1.20× serialized on the exact/1w rows, the isize rows the canaries. Record the four rows and the ratios in BENCH.md § The end-to-end SUM with the machine and the commit. A row past the budget is a design question for the owner, not a build to ship.

## log
- 2026-10-15T17:47Z created
