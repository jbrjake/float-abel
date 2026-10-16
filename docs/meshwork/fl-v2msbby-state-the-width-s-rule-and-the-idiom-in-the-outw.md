---
id: fl-v2msbby
title: "State the heap Exact's contract in the outward docs — README § The contract and § In an engine, DESIGN.md § Exactness bounds, § Exact and § differential-dataflow — the two named bounds, the compact form's rule, the banded fast path, and the panic = \"abort\" note"
category: crate/differential
discovered-from: fl-h41w0tr
verify: "all(contains README.md /panic = \"abort\"/, contains README.md /EXACT_MAX_LIMBS|limb cap/)"
seq: 50
labels: [reproducible-floats, docs]
docs:
  - README.md#the-contract
  - README.md#in-an-engine
  - docs/DESIGN.md#exactness-bounds-sp-bounds
status: open
created: 2026-10-15T15:15Z
needs: [fl-xf8hv25]
handoff: |
  Next session states the heap `Exact`'s contract in the outward docs.
  What stands after the
  banded idiom landed (16db257, with its bench rows in the commit after
  it; findings on the closed
  task `search "Ship the banded SUM idiom"`): README § In an engine shows
  both idioms, says when
  to reach for `banded_sum`, and already carries the limb-cap sentence and
  `panic = "abort"`;
  DESIGN.md § differential-dataflow has the banded paragraph; OVERFLOW.md
  § The banded idiom is in
  the shipped form and § Cost ends with the probe's table. So this task's
  verify is green and
  blind, as its comment says: re-set it first to a predicate the missing
  sentences fail today, for
  instance `all(contains README.md /EXACT_MAX_LIMBS/, contains
  docs/DESIGN.md /not .Copy./)` after
  checking each is absent where you mean to write it (`grep -n` both files
  before choosing; README
  § The contract has no `EXACT_MAX_LIMBS` today).
  
  What remains, in the body's order. README § The contract: the promise
  is "the same multiset
  lands the same bits", and nothing says a sum can fail; add that `Exact`
  holds every sum and every
  product and that a dataflow with it as the diff cannot panic short of
  two named bounds, the limb
  cap (`EXACT_MAX_LIMBS`, 16 × 2148 bits, about 4.3 KB a weight, reached
  only through sixteen
  nested `float × float` joins on extreme values; a `const` the consumer
  raises by editing it) and
  the exponent bound (`EXACT_MAX_EXP`, 2³⁰); and what the compact form
  holds without allocating
  (first set bit to last, 127 bits: two 53-bit values within about 2⁷⁰
  of each other, which is
  why `1e12 + 1e-12` promotes and `1e11 + 1e-11` does not, the spike's
  pairs in the findings of
  the closed task `search "Spike the Exact panic"`), so that the
  banded idiom is the allocation-free path for sums over mixed magnitudes.
  Say that `Exact` is
  `Clone` and not `Copy`, and that `Wide` and `Fixed` are the `Copy` words
  for an engine's `Copy`
  slot or a declared window (fl-pwq0h8n gives them the weight traits).
  DESIGN.md § Exactness
  bounds and § Exact carry the compact rule already (the fourth bullet
  and "the smallest form that
  holds the value"); check each sentence against src/exact/mod.rs's docs
  rather than the plan.
  Then OVERFLOW.md's fate: the body asks you to decide between shortening
  it to the rule and the
  measurements or folding it into DESIGN.md and deleting it, and to say
  why. The record it holds
  (the five histories, the hang, § Cost's probe table that § Exact and
  BENCH.md cite) is cited
  from DESIGN.md, README and the store, so shortening is the safer of the
  two; a deletion moves
  every citation. Either way the owner reads the choice back before it
  lands: it is a doc the
  design cites.
  
  Where the numbers come from: BENCH.md § The end-to-end SUM's banded
  table (same-scale and mixed
  2^±300, one worker and eight, both exchanges) is the source for any
  speed sentence; README § In
  an engine's closing paragraph already quotes its shape and should quote
  nothing BENCH.md does
  not hold. The README is a doctest (`src/lib.rs`'s `readme` module), so a
  new Rust block there
  must run under `cargo test`.
---

Gated on the heap arm and its proof in both engines, because what the docs promise is what those
tasks land. Today README § The contract says "the same multiset lands the same bits under any
order, worker count, merge tree, consolidation timing or architecture" and never that a sum can
fail; § In an engine documents the idiom that panics on 1e12 then 1e-12 in one key.

To write, once fl-heap and the adapters task are done: `Exact` holds every sum and every
product, and a dataflow with it as the diff cannot panic on input short of two named bounds, the
limb cap (`EXACT_MAX_LIMBS`, 4.3 KB a weight, reached only through ≥ 16 nested `float × float`
joins on extreme values, a `const` the consumer can raise) and the exponent bound; what the
compact form holds without allocating (first 1 to last 1, 127 bits: two 53-bit decimals within
about 10²² of each other) and that past it a weight allocates, so the banded idiom
(fl-gpjzcqc) is the allocation-free path for sums over wildly mixed magnitudes; that `Exact` is
not `Copy` and why `Wide` and `Fixed` are; and that a multi-worker consumer builds with
`panic = "abort"`, because a worker's panic at the limb cap otherwise hangs the process
(docs/OVERFLOW.md § Why one worker's panic hangs the rest). DESIGN.md § Exactness bounds, § Exact
and § differential-dataflow carry the same; CLAUDE.md § Standing decisions already carries the
ruling from fl-heap. docs/OVERFLOW.md then becomes a record of the resolved question: shorten it
to the rule and the measurements, or fold it into DESIGN.md and delete it, decide which and say
why.

## log
- 2026-10-15T15:15Z created
- 2026-10-16T16:34Z handoff by claude (94f54cd7-858c-4c70-ad69-f1f2ca548e35)

## comments
- 2026-10-16T14:22Z [claude (028f9a17-8bd2-4590-bb38-7e9820e83a28)] 9714758 (fl-xf8hv25) landed part of this scope while stating what the product-fold tests prove: README § In an engine now says a join of two Exact-weighted collections multiplies the weights, that the limb cap is the one panic left inside a dataflow, reachable through nested products alone, and that a consumer who can reach it builds with panic = "abort"; DESIGN.md § differential-dataflow carries the same with the hang's reason, and docs/OVERFLOW.md's opening, § What fits (a heap-arm row), § Why Exact panics and § The banded idiom say which rows describe the compact form alone and what the heap arm holds now. So this task's verify is already green and blind to what remains: README § The contract's promise, DESIGN.md § Exactness bounds and § Exact (the compact form's 127-bit rule as the allocation-free band, Exact not Copy and why Wide and Fixed are), the banded fast path's place once fl-gpjzcqc lands, and OVERFLOW.md's fate. Re-set the verify to a predicate those sentences fail today before starting.
