---
id: fl-cwwnyas
title: Draw the limb cap's edge in the laws kit's Exact strategy
status: open
category: crate/types
labels: [reproducible-floats]
discovered-from: fl-dchdzd4
relates: [fl-dchdzd4]
verify: run cargo test package=float-abel laws_closure_exact_at_the_cap
docs:
  - docs/DESIGN.md#exactness-bounds-sp-bounds
created: 2026-10-16T18:26Z
handoff: |
  Next session: this is the one buildable task left in `ready`: a strategy
  for tests/laws.rs that
  lands at the limb cap's exact edge, so the laws kit's closure check
  covers what the hand-written
  `exact_cap_closed_*` cases in tests/exact_cap.rs do. Build it from
  `edge_shapes()` there (eleven
  integers at and about the cap, each paired with its `num-bigint` oracle,
  built through
  `two_to(k)` from tests/exact_heap.rs and the checked operators),
  `prop::sample::select` over
  those plus small sums of them, and run `laws::check_closure` on it with
  `small_int()` and with
  itself in a test named `laws_closure_exact_at_the_cap_*`; keep it
  `cfg(not(miri))` with the rest
  of the file. The other two open tasks are not buildable here: fl-qv0vh39
  waits on a machine with
  AVX-512 (neither NEON nor jons-amd-mini nor turing has it), and
  fl-k3g1511, the publish ruling,
  is the owner's and is unblocked now that its three gating tasks are
  done; fl-4fcbtkx (push to
  origin) and fl-mj3tcta (the SIMD product kernel) stay blocked on the
  owner's word.
  
  What stands at e9cc317: the limb cap closes under retraction at its edge
  (6c04216:
  src/exact/big.rs decides `add` on the aligned span plus a carry bit for
  like signs and the span
  alone for unlike, refusing before allocating only when even one limb
  past the cap cannot hold the
  result, and `mul` on `p + q − 1`; DESIGN.md § Exactness bounds
  carries the rule and its pin).
  `bench-baseline.json` holds both machines' entries at 85b93c2 (114 ids
  each), written by
  scripts/bench-baseline.sh, whose `compare` prints every row past 1.5×;
  BENCH.md's intro says what
  it records and how the sitting read. BENCH.md § The end-to-end SUM has
  the heap arm's budget A/B
  (b4aa676): 1.08× typed, 1.09× serialized against the Copy word's
  binary, inside 1.12×/1.20×, and
  DESIGN.md § Exact and docs/OVERFLOW.md quote those. Verified this
  session: `cargo test` 231
  green, clippy and rustdoc clean, every commit through the commit gate;
  the push gate was not
  run, and main is unpushed by the owner's word. The spec audit on
  docs/DESIGN.md lists 34 re-open
  candidates, pins on closed tasks whose clauses moved before this
  session; nobody has ruled on
  repinning them.
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
