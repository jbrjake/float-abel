---
id: fl-6shtzpj
title: "Publish the laws test kit as a feature and run every exact tier through it, with a naive f64 control that must fail"
category: crate/laws
labels: [reproducible-floats, rf-critical-path]
docs: [docs/DESIGN.md#the-laws-kit-sp-laws-kit, docs/DESIGN.md#what-an-incremental-engine-asks-of-a-weight-sp-laws]
needs: [fl-n7beqqc, fl-at6nh41, fl-atkbp6q]
seq: 80
verify: "all(exists src/laws.rs, run cargo test package=float-abel laws_)"
status: done
created: 2026-10-11T15:02Z
covers:
  - ref: docs/DESIGN.md#sp-laws-kit
    sha: 70b9088299a74a0d8bc6a0d9b0f6341f6ada7efb68338473bfed30a0f1b5a213
  - ref: docs/DESIGN.md#sp-laws
    sha: d04a257c60cce5cd8d14fd90d26d72979fabd4bf84b6d91f25a22d89077bc56f
relates: [fl-cfx001b, fl-v7t6xhz]
---
Per DESIGN.md § The laws kit: `float_abel::laws::{check_abelian_group, check_ring, check_canonical,
check_consolidation_invariance}`, generic over a `Strategy`. Its docs show a stranger running it
against their own type.

`laws_*` tests run the kit against `Fixed256`, `Wide` and `Exact`. They also run it against a naive
`f64` wrapper as the negative control, which must **fail** associativity and inverse.

## log
- 2026-10-11T15:02Z created
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-laws-kit @2246c38eec81
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-laws @d04a257c60cc
- 2026-10-12T13:23Z handoff by claude (f24d940b-27ff-4906-b27c-e6a7b31fddaf)
- 2026-10-12T13:25Z handoff by claude (f24d940b-27ff-4906-b27c-e6a7b31fddaf)
- 2026-10-12T13:41Z open→doing — claimed by claude (51dfb626-8f8b-48f4-9e4a-68528d76dabd)
- 2026-10-12T13:55Z cover docs/DESIGN.md#sp-laws-kit @70b9088299a7
- 2026-10-12T13:58Z doing→done — verify exit 0 @ ece665a+3

## comments
- 2026-10-12T13:58Z [claude (51dfb626-8f8b-48f4-9e4a-68528d76dabd)] Closed at ece665a, verify exit 0. Built: src/laws.rs (the Weight trait with impls for Fixed, Wide and Exact; runner(cases, seed); Case and RingCase; the four checks) and src/laws/report.rs (the seeded splitmix shuffler, fold, hash, the pruning merge tree, and Broken, the per-case report naming every law broken with one witness each). tests/laws.rs: 9 laws_* tests, 256 fixed-seed cases per check per tier, plus the Naive(f64) control asserted to fail associativity and inverse (both named in one unshrunk report), distributivity (the first witness is zero · -1 = -0.0, a second zero), canonicality (two histories of one sum differ) and consolidation invariance (the witness is a multiset that cancels to 8.7e-19 in one order and 0.0 pruned). Facts: proptest is in the lock at 1.11.0 by --precise because 1.12's rand 0.10 closure is edition 2024 and cargo 1.74 parses every locked manifest on a normal edge, optional features included (comment on both adapter tasks); the three tests/ui .stderr files were re-blessed because proptest's RngSeed::Fixed makes the name Fixed non-unique, so rustc prints float_abel::Fixed in its notes. DESIGN.md § The laws kit carries the as-built sentence and the clause is re-pinned.
