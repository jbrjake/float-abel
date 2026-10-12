---
id: fl-w9mk4jc
title: Fix the noalloc member's assign-op clippy error so the push gate is green
status: done
category: crate/hardening
relates: [fl-v7t6xhz]
verify: lacks tests/noalloc/main.rs /finite = finite \+ Binned/
docs:
  - scripts/regression.sh#the-push-gate
seq: 105
created: 2026-10-12T15:30Z
---

`cargo clippy --quiet --workspace --all-targets -- -D warnings`, the push gate's third clippy leg, fails at `tests/noalloc/main.rs` in `noalloc_a_binned_fold_lands_in_wide_without_std`: `finite = finite + Binned::from(0.0)` is `clippy::assign_op_pattern`, denied through `clippy::all`. The line came in with "Build Binned<K>" (fl-v7t6xhz), whose session ran clippy on the root package alone, which never compiles the noalloc member. Found while running the gate for the golden hash. The fix is `finite += Binned::from(0.0)`: `Add` is already exercised by the backward fold two lines above, so nothing is lost.

## log
- 2026-10-12T15:30Z created
- 2026-10-12T15:30Z open→doing — claimed by claude (687d89e4-ae5c-4805-bf5e-f65f0e4a2b1a)
- 2026-10-12T15:33Z doing→done — verify exit 0 @ a6a17f3+2
