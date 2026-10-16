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

## comments
- 2026-10-16T15:40Z [claude (94f54cd7-858c-4c70-ad69-f1f2ca548e35)] 2026-10-16, the banded idiom's session: the Copy word's control binary (float-abel-control/release/deps/dd_sum-ab4b21425246350c) ran --bench '1w' right after the variant's 20-row sitting at 16db257, but the host was shared with the owner's other project's applied experiment (load 7–15) and the control's isize/1w/serialized canary read 263 ms against 207.6 idle, so those rows are not the A/B. The shape that works: sitting.sh (waits for load under 3 and no cargo, criterion, dd_sum-, marasi-applied or chrome-headless-shell process, then runs the newest dd_sum-* binary) chained with control.sh (waits for the sitting's done marker, runs the control on --bench '1w' with CRITERION_HOME apart); both in the 2026-10-16 session's scratchpad and described in the build-environment memory. The variant's exact/1w rows from a clean re-take of 16db257 are in BENCH.md § The end-to-end SUM once it lands; the control half still needs a quiet window.
- 2026-10-16T16:33Z [claude (94f54cd7-858c-4c70-ad69-f1f2ca548e35)] Harness-shape caveat, measured 2026-10-16 in one quiet window (load 2.41): the 9714758 dd_sum binary (8 rows, one fixture) and the 16db257 binary (20 rows, three fixtures) ran the one-worker typed rows back to back; isize/1w/typed read 121.6 ms against 128.5 and exact/1w/typed 283.7 against 318.8, with the isize dataflow and fixture byte for byte the same in both. So the harness shape alone moves rows 6–12%, and this task's A/B must compare binaries of one harness shape: the control is the 8-row harness at 03d96e9, so build the variant from a worktree with the current lib and that commit's benches/dd_sum.rs (or run both the control and the variant through the current 20-row harness by rebuilding the control from a worktree at 03d96e9 with the current benches/), and say which in BENCH.md.
- 2026-10-16T18:10Z [claude (cd1130c2-56bf-432b-bcb1-8ed1c474e8fa)] 2026-10-16 owner, in session, on a sitting script that was waiting for the one-minute load to fall under 3 with nothing else running: "dude there is nothing else running WHY DO YOU INSIST ON SO LITTLE LOAD TO RUN YOUR TESTS THIS IS MY DEV BOX????". So this task's "one quiet sitting" means: check ps once for competing work (another cargo, criterion or dd_sum binary, the other project's experiment binaries) and, finding none, run at once; no load-average threshold. Judge the sitting afterwards by the isize canaries. The wide-row sitting at 440ac8c ran that way (started at load 2.75 by chance) and its isize rows read within 1-5% of 16db257's and 9714758's.
