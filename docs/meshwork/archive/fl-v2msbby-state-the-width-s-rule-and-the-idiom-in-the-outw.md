---
id: fl-v2msbby
title: "State the heap Exact's contract in the outward docs — README § The contract and § In an engine, DESIGN.md § Exactness bounds, § Exact and § differential-dataflow — the two named bounds, the compact form's rule, the banded fast path, and the panic = \"abort\" note"
category: crate/differential
discovered-from: fl-h41w0tr
verify: "all(contains README.md /panic = \"abort\"/, contains README.md EXACT_MAX_EXP, contains README.md /not .Copy./, contains docs/DESIGN.md 1e-12)"
seq: 50
labels: [reproducible-floats, docs]
docs:
  - README.md#the-contract
  - README.md#in-an-engine
  - docs/DESIGN.md#exactness-bounds-sp-bounds
status: done
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
- 2026-10-16T16:34Z handoff by claude (94f54cd7-858c-4c70-ad69-f1f2ca548e35)
- 2026-10-16T16:47Z open→doing — claimed by claude (84727215-1ab8-4972-b9d9-cd4dd0eb435e)
- 2026-10-16T16:50Z doing→done — verify exit 0 @ 1cc2e00+2

## comments
- 2026-10-16T14:22Z [claude (028f9a17-8bd2-4590-bb38-7e9820e83a28)] 9714758 (fl-xf8hv25) landed part of this scope while stating what the product-fold tests prove: README § In an engine now says a join of two Exact-weighted collections multiplies the weights, that the limb cap is the one panic left inside a dataflow, reachable through nested products alone, and that a consumer who can reach it builds with panic = "abort"; DESIGN.md § differential-dataflow carries the same with the hang's reason, and docs/OVERFLOW.md's opening, § What fits (a heap-arm row), § Why Exact panics and § The banded idiom say which rows describe the compact form alone and what the heap arm holds now. So this task's verify is already green and blind to what remains: README § The contract's promise, DESIGN.md § Exactness bounds and § Exact (the compact form's 127-bit rule as the allocation-free band, Exact not Copy and why Wide and Fixed are), the banded fast path's place once fl-gpjzcqc lands, and OVERFLOW.md's fate. Re-set the verify to a predicate those sentences fail today before starting.
- 2026-10-16T16:50Z [claude (84727215-1ab8-4972-b9d9-cd4dd0eb435e)] 1cc2e00 lands the contract. README § The contract now says, after the honest-boundary paragraph, that a sum never fails on its input: Exact holds every sum and every product, NaN and ±∞ counted; a dataflow with it as the diff cannot panic short of the limb cap (EXACT_MAX_LIMBS, 16 × 2148 bits, about 4.3 KB a weight, sixteen nested float × float joins on extreme values, a const the consumer raises) and the exponent bound (EXACT_MAX_EXP, 2³⁰, about a million nested products of the smallest subnormal); the compact word holds a sum spanning at most 127 bits from first set bit to last, two full-mantissa decimals within about 10²² of each other, 1e11 + 1e-11 at 126 bits in the word and 1e12 + 1e-12 promoting at 132 (the spans re-derived this session from the doubles' bits in integer arithmetic: 126, 132, and 1 + 1e-22 at 126, 1 + 1e-23 at 130, matching docs/OVERFLOW.md § What fits), a key of n rows narrowing the allowance by about log₂ n; past the word one allocation freed by the next op that fits; the banded idiom as the allocation-free SUM path; and panic = "abort" for a multi-worker consumer who can reach a bound. § The trilemma's paragraph ends with why Exact is Clone and not Copy (a word that can hold a pointer leaves an engine's Copy weight slot and a cast over a mapping) and that Wide and Fixed are the Copy words for that slot, Fixed for a declared window besides, without claiming the engine traits fl-pwq0h8n gives them. § In an engine names both bounds where it named the limb cap alone. DESIGN.md § Exactness bounds' fourth bullet carries the pair with its bit counts and the banded path as the allocation-free SUM over the adversary; § Exact gains a paragraph on the Copy trade and what it buys. Every sentence was checked against src/exact/mod.rs's docs (EXACT_MAX_LIMBS, EXACT_MAX_EXP, PastCap) and BENCH.md § The end-to-end SUM; no new Rust block, and the readme doctest module still passes (4 tests). The verify was re-set before the work to predicates the missing sentences failed (EXACT_MAX_EXP and "not `Copy`" in README, 1e-12 in DESIGN.md), red at start, green at close.
  
  OVERFLOW.md's fate: decided for shortening, not folding, and read back to the owner as fl-5h12vf3 before it lands. Why: the doc is cited from twenty places in the tree by section (DESIGN.md three sections, BENCH.md twice, two rustdoc modules, four test files, the bench) and every one points at a section that holds the record; only § Options, whose recommendation for option 4 the ruling for the heap arm overtook, is cited by nothing in the tree and by one store docs: line (fl-pwq0h8n). Shortening replaces § Options with what was decided and moves no citation; folding moves twenty and puts about 150 lines of memo into a 741-line DESIGN.md. The file is untouched this session.
