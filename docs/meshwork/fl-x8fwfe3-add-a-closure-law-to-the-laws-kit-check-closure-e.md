---
id: fl-x8fwfe3
title: "Add a closure law to the laws kit — check_closure: every pairwise sum and fold the strategy draws is defined — which the heap Exact must pass on a 2^±300 strategy, with a capped test newtype over checked_add as the negative control, so a capped type is seen"
category: crate/types
discovered-from: fl-h41w0tr
verify: "all(contains src/laws.rs /pub fn check_closure/, run cargo test package=float-abel laws_closure_)"
seq: 60
labels: [reproducible-floats]
docs:
  - docs/DESIGN.md#the-laws-kit-sp-laws-kit
  - docs/OVERFLOW.md#why-exact-panics-in-a-dataflow
status: open
created: 2026-10-15T15:15Z
needs: [fl-th7g69s]
---

Found in fl-h41w0tr's spike: tests/laws.rs draws `Exact` from `double(-28, 28)`, "doubles whose
leading bits lie within 56 binary decades of each other", inside the width by construction, and
the kit has no law for closure, the first group axiom. The kit was tuned to the type's domain and
so could not see the one law the 32-byte `Exact` fails. With the heap arm (fl-th7g69s) `Exact`
is closed under `+` and `×` short of the limb cap, and this law is what proves it and what keeps
a stranger's capped type from passing the kit.

To build, after the heap arm lands: `check_closure` in src/laws.rs over a strategy, asserting
that `a + b`, `a − b`, `a × k` and a fold of a drawn multiset in a drawn order are all defined;
`Weight` needs a fallible add for it (`try_plus` returning `Option<Self>`, with `Exact` mapping
`checked_add`, and the fixed-width tiers answering `Some` inside their headroom), or the check
runs under `catch_unwind` in the `std` kit, decide which and say why; tests named
`laws_closure_*` in tests/laws.rs: `Exact` passes on a 2^±300 strategy with specials and on
products of pairs, `Wide` and `Fixed256` pass on their domains, and a test-local newtype that
adds through the compact tier alone (`Exact::checked_add`, refusing past the width) fails,
asserted as a failure value so the control's failing is the test. DESIGN.md § The laws kit lists
the law; the no-allocator build's compact-only `Exact` is documented as not closed, by design.

## log
- 2026-10-15T15:15Z created
