---
id: fl-cwwnyas
title: Draw the limb cap's edge in the laws kit's Exact strategy
status: done
category: crate/types
labels: [reproducible-floats]
discovered-from: fl-dchdzd4
relates: [fl-dchdzd4]
verify: run cargo test package=float-abel laws_closure_exact_at_the_cap
docs:
  - docs/DESIGN.md#exactness-bounds-sp-bounds
  - docs/DESIGN.md#the-laws-kit-sp-laws-kit
created: 2026-10-16T18:26Z
covers:
  - ref: docs/DESIGN.md#sp-bounds
    sha: 97ad60a820b8ae5751cf4fe22c42747ab397cd36a95ad39e250d50842e1899cf
  - ref: docs/DESIGN.md#sp-laws-kit
    sha: e2c0de0ec31d0c8544da9b093c7151164a2a35e3dded4e6bd7a69b4063262f30
---

tests/laws.rs's `exact_full()` is the closure law's strategy for `Exact`: it draws every binade with
specials and products, but never a value within a bit of `EXACT_MAX_LIMBS`, so the closure law
(fl-x8fwfe3) could not have caught fl-dchdzd4's defect, a value at the cap's exact width that
could not be subtracted from itself; the wire tests found it by hand. The law is only as strong as
its strategy.

Add a strategy that lands at the edge: the eleven shapes of tests/exact_cap.rs's `edge_shapes()`
(`2^(C−1) + 3`, `2^C − 1`, `2^(C−1) − 1`, the powers at and above the cap, small integers) and
small sums of them, each a value the operations hold, and run `check_closure` on it with
`small_int()` and with itself in a test named `laws_closure_exact_at_the_cap_*`, so a regression
at the edge is the kit's to catch, not a hand-written case's. The sweep must stay cheap under miri:
a few hundred draws of 537-limb values, or `cfg(not(miri))` like the rest of tests/laws.rs.

## log
- 2026-10-16T18:26Z created
- 2026-10-16T19:10Z handoff by claude (2ba3bd95-5552-4160-8df2-82be1495b54b)
- 2026-10-16T19:11Z handoff by claude (2ba3bd95-5552-4160-8df2-82be1495b54b)
- 2026-10-16T19:13Z open→doing — claimed by claude (03cbad3b-1e00-455d-9d44-3b0d9decee6e)
- 2026-10-16T19:31Z cover docs/DESIGN.md#sp-bounds @97ad60a820b8
- 2026-10-16T19:31Z cover docs/DESIGN.md#sp-laws-kit @e2c0de0ec31d
- 2026-10-16T19:31Z doing→done — verify exit 0 @ 18e42b3+1

## comments
- 2026-10-16T19:31Z [claude (03cbad3b-1e00-455d-9d44-3b0d9decee6e)] Done at 18e42b3, tests/laws_cap.rs. The handoff's sketch, the eleven `edge_shapes()` as the strategy, cannot be a closure strategy: the check pairs and folds every draw, and `2^C − 1` plus `2^(C−1)` is already past the cap, so the law would fail on the fixed code. The budget comes from the kit's constants instead: a case folds up to WEIGHTS = 32 weights in a thousand orders, and thirty-two integers under `2^k` sum under `2^(k + 5)` in every order and every partial, so a weight spans at most CAP_BITS − 5 and the multiplier takes those five bits (|k| ≤ 32); a weight one bit wider, or one multiplier bit more, passes the cap in some order. `exact_at_the_cap()` draws three wide shapes at that budget (every bit set; every bit above the first limb with the bottom one; the top two with the bottom one) and three small ones (1, 3, 2^64 + 1), one draw in sixteen small and one in sixteen negative, so a case's fold climbs onto the cap's last bit within twenty-odd draws and retracts across it, and the odd multipliers 17..31 land the wide shapes there too. Pairwise sums never reach it: two weights at the budget sum four bits under.
  
  Tests, all `laws_closure_exact_at_the_cap_*`: closure by an integer, closure by an `Exact` (so `Mul<Exact>` lands on the last bit), the abelian-group check on the same strategy, and the `UnderCap` control, an `Exact` whose `+`, `−` and `· k` refuse a result spanning exactly CAP_BITS (heap form, EXACT_MAX_LIMBS limbs, top bit set): it fails the closure law on the edge strategy with a fold or a product as the witness, and holds on `exact_full()`, which proves the reach and that the old strategy was blind. 32 cases per sweep (`CASES` in the file, with `edge_runner()` and `unshrunk_for(32)` so the control's reach is proven on exactly the cases the sweeps run), an eighth of the kit's 256, because every operation moves 537 limbs: at 256 the two closure sweeps ran past a minute each in the dev profile; at 32 the five tests finish in 8.5 s wall, and the commit gate's whole suite in 17.6 s.
  
  Watched fail on a deliberate break: `big::add` put back to its pre-fix refusal (`top − low + 2 > CAP_BITS`) fails the three sweeps, 328 cap panics, the two control tests unaffected. `big::mul` put back to `lhs.bits() + rhs.bits() > CAP_BITS` passes all five: the check's independent draws cannot pair a weight and a multiplier whose bits sum to CAP_BITS + 1 while their product fits (with |w| < 2^(C−5) and |k| ≤ 2^5 the bit counts sum to at most C), so the mul pre-check's edge stays with tests/exact_cap.rs's oracle sweep, not the kit's. Not done: the ring check at the edge, which needs a 4-bit multiplier (`(a + b)·k` is one bit wider); one `check_ring(edge_runner(), exact_at_the_cap(), -16..=16)` if wanted. tests/laws.rs lends `runner`, `SEED`, `unshrunk_for`, `exact_full`, `small_int` and `failure` as pub(crate). Verified this session: clippy `-D warnings` and rustdoc clean, the commit gate green on the commit (236 tests); the push gate not run yet at this point.
