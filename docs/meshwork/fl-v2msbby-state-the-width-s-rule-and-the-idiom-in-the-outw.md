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

## comments
- 2026-10-16T14:22Z [claude (028f9a17-8bd2-4590-bb38-7e9820e83a28)] 9714758 (fl-xf8hv25) landed part of this scope while stating what the product-fold tests prove: README § In an engine now says a join of two Exact-weighted collections multiplies the weights, that the limb cap is the one panic left inside a dataflow, reachable through nested products alone, and that a consumer who can reach it builds with panic = "abort"; DESIGN.md § differential-dataflow carries the same with the hang's reason, and docs/OVERFLOW.md's opening, § What fits (a heap-arm row), § Why Exact panics and § The banded idiom say which rows describe the compact form alone and what the heap arm holds now. So this task's verify is already green and blind to what remains: README § The contract's promise, DESIGN.md § Exactness bounds and § Exact (the compact form's 127-bit rule as the allocation-free band, Exact not Copy and why Wide and Fixed are), the banded fast path's place once fl-gpjzcqc lands, and OVERFLOW.md's fate. Re-set the verify to a predicate those sentences fail today before starting.
