---
id: fl-x8fwfe3
title: "Add a closure law to the laws kit — check_closure: every pairwise sum and fold the strategy draws is defined — which the heap Exact must pass on a 2^±300 strategy, with a capped test newtype over checked_add as the negative control, so a capped type is seen"
category: crate/types
discovered-from: fl-h41w0tr
verify: "all(contains src/laws/closure.rs /pub fn check_closure/, contains src/laws.rs /pub use closure::.check_closure/, run cargo test package=float-abel laws_closure_)"
seq: 60
labels: [reproducible-floats]
docs:
  - docs/DESIGN.md#the-laws-kit-sp-laws-kit
  - docs/OVERFLOW.md#why-exact-panics-in-a-dataflow
status: done
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
- 2026-10-16T16:51Z open→doing — claimed by claude (84727215-1ab8-4972-b9d9-cd4dd0eb435e)
- 2026-10-16T17:02Z close attempt — verify failed (dsl)
- 2026-10-16T17:03Z doing→done — verify exit 0 @ b97550c+3

## comments
- 2026-10-16T17:02Z [claude (84727215-1ab8-4972-b9d9-cd4dd0eb435e)] b97550c. Landed as check_closure in src/laws/closure.rs, re-exported from float_abel::laws with its ClosureCase (weights, multipliers, seed). Per case it runs a + b and a − b over every pair, a · k over every multiplier, and the fold of the weights in the drawn order and under the kit's 1,000 shuffles, each operation inside catch_unwind; the first that does not return is reported as the law "closure" with the operands and the panic's message as the witness, through the same Broken report the other laws use.
  
  The decision the body left open, catch_unwind against a fallible try_plus on Weight: catch_unwind, for three reasons. Closure for an engine means plus_equals returns, so the check observes the operator itself and needs no declaration from the type: a stranger's capped type is seen whether or not it says it has a cap, and a type that lies about one is seen too. One mechanism covers +, −, × and the fold, where try_plus would need a fallible multiply beside it (a TryMul<K> trait, since Mul is foreign) and new checked adds on Wide and Fixed, whose asserted headroom would otherwise answer a default Some and make the law a tautology on them. And it costs no API: Weight is unchanged, so every implementor compiles. The price is std, which the laws feature already requires, and that a caught panic prints through the hook; libtest captures the test thread's output, so a passing test prints nothing. proptest 1.11 catches a panicking test body itself and reports the message, but silences the hook only behind its handle-panics feature, which the crate does not enable; what check_closure adds over that catch is the law's name and the operands, and that every pair, product and order is tried rather than the first check that trips.
  
  Tests, tests/laws.rs, all laws_closure_*: Exact holds on every binade 2^±300 with one in sixteen a special, by a small integer and by another such Exact, and on a multiset of Exact::product row products by an integer Exact; Wide and Fixed256 hold on their strategies. The negative control is Compact(Exact), the build without alloc modelled on the heap build: every operator asserts the result is_compact and panics otherwise. It passes the law on the kit's 56-binade strategy, which is the finding of the spike restated as a test (the law is only as strong as the strategy), fails past the width on the first case drawn with "closure" and "127-bit width" in the reason and more than one weight in the case, and with shrinking on fails on exactly two values whose sum or difference is not compact, asserted on the pair. The deliberate break was the kit swallowing the caught panic as a return: exactly the two control tests failed with "the control did not fail the closure check: Ok(())" and the three positive ones passed; restored, all 19 laws_ tests pass in 4.4 s. fmt, clippy --all-targets -D warnings and rustdoc -D warnings are clean, and the module doctest runs the fifth check on its Count example.
  
  Docs: the module docs list the law and tell a caller to draw the closure strategy from the whole range; DESIGN.md § The laws kit lists it with the compact-only control and names check_closure among the entry points; README § Checking your own weight lists it and the second control; docs/OVERFLOW.md § Why Exact panics now says the kit sees the cap through check_closure on a strategy past the width, where it said the kit does not see it. The compact-only Exact of a build without alloc is documented as not closed, by design, in the rustdoc and DESIGN.md.
- 2026-10-16T17:03Z [claude (84727215-1ab8-4972-b9d9-cd4dd0eb435e)] The verify's first leg named src/laws.rs as the file of pub fn check_closure; the function lives in src/laws/closure.rs, a submodule added so src/laws.rs stays at 458 lines against the 500-line target, and src/laws.rs re-exports it as float_abel::laws::check_closure. The predicate now names that file and the re-export; its substance, a public check_closure in the kit and green laws_closure_ tests, is as filed.
