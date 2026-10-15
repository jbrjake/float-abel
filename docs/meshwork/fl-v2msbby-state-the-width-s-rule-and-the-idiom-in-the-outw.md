---
id: fl-v2msbby
title: "State the width's rule and the idiom in the outward docs — README § The contract and § In an engine, DESIGN.md § Exactness bounds, § Exact and § differential-dataflow — with the panic = \"abort\" note for multi-worker consumers"
category: crate/differential
discovered-from: fl-h41w0tr
verify: "all(lacks docs/DESIGN.md /the exact tier exists for the latter/, contains README.md /panic = \"abort\"/, contains README.md /127/)"
seq: 50
labels: [reproducible-floats, docs]
needs: [fl-t154dsp]
docs:
  - README.md#the-contract
  - README.md#in-an-engine
  - docs/DESIGN.md#exactness-bounds-sp-bounds
status: open
created: 2026-10-15T15:15Z
---
Gated on fl-t154dsp, because what the docs recommend is the ruling. Today README § The contract
says "the same multiset lands the same bits under any order, worker count, merge tree,
consolidation timing or architecture" and never that a sum can fail; § In an engine documents the
idiom that panics on 1e12 then 1e-12 in one key; DESIGN.md § Exactness bounds still says "Real
columns look like the former; the exact tier exists for the latter", naming a tier the crate no
longer has.

To write, in the owner's ruled shape: the plain rule (docs/OVERFLOW.md § What fits: first 1 to
last 1, 127 bits, so two 53-bit decimals within about 10²² of each other, narrowing by log₂ n),
the idiom the crate recommends for sums of arbitrary doubles, what a plain `Exact` diff is for,
where products go, and that a multi-worker consumer builds with `panic = "abort"` because a
worker's panic otherwise hangs the process (docs/OVERFLOW.md § Why one worker's panic hangs the
rest). CLAUDE.md § Standing decisions gets one clause on the idiom if the ruling makes it one.

## log
- 2026-10-15T15:15Z created
