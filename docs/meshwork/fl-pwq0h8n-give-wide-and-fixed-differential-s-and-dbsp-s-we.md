---
id: fl-pwq0h8n
title: "Give Wide and Fixed differential's and DBSP's weight traits — IsZero, Semigroup, Monoid, Abelian, Multiply<isize>/<i64> and the DBSP set — for an engine that needs a Copy diff or a declared window, with the 3-worker test and the criterion row"
category: crate/differential
discovered-from: fl-h41w0tr
verify: "all(contains src/wide.rs /differential/, run cargo test package=float-abel differential_wide_)"
seq: 70
labels: [reproducible-floats]
docs:
  - docs/OVERFLOW.md#what-fits
  - docs/DESIGN.md#wide-sp-wide
  - docs/DESIGN.md#fixed-sp-fixed
status: open
created: 2026-10-15T15:15Z
needs: [fl-th7g69s]
handoff: |
  Next session gives `Wide` and `Fixed` the engines' weight traits. What
  stands after this
  session: README § The trilemma and DESIGN.md § Exact (1cc2e00) say
  `Wide` and `Fixed` are the
  `Copy` words for an engine's `Copy` weight slot and a cast over a
  mapping, worded so as not to
  claim the traits; when this lands, README § In an engine, both trilemma
  tables (DESIGN.md's has
  a `Copy` column) and DESIGN.md § Wide and § Fixed say which traits
  each carries. The laws kit's
  closure law (b97550c, `laws::check_closure`) already holds on the
  `wide()` and `fixed256()`
  strategies of tests/laws.rs, so the kit's five laws are green on both
  words before an engine
  sees them. The ruling the body quotes stands: owner, 2026-10-15, "yes
  let's do heap and allow
  product fols inside a dataflow", so `Wide` is no longer the carrier for
  product folds inside an
  engine; this task is the `Copy` diff and the declared window only.
  
  Files and symbols. The shape to copy is src/exact/differential.rs:
  `IsZero`, `Semigroup`
  (`plus_equals` as `+=`), `Monoid` (`zero`), `Abelian` (`negate`),
  `Multiply<i64>`,
  `Multiply<isize>`, and the orphan-legal `impl Multiply<Wide> for i64`
  and `isize`, each a few
  lines over the ops both words already have (`impl Mul<i64> for Wide` and
  `Mul<isize>` beside it
  in src/wide.rs; the same pair in src/fixed.rs, generic over `LSB,
  LIMBS`). No `Multiply<Wide>
  for Wide`: the rustdoc states the bit count (two 2176-bit words' product
  has no carrier) and
  points at `deposit_product` as the door. `From<i8>` for `count_core` and
  `distinct_core`
  outputs. `ExchangeData` wants `Serialize + Deserialize`, which
  src/wire/serde_impls.rs already
  gives both words behind `serde` (`differential` implies `serde`), and
  `Ord + Hash + Debug +
  Clone + Send + 'static`, which both have. The DBSP set is
  src/exact/dbsp.rs's: `HasZero`,
  `HasOne`, `MulByRef<i64>` and `<isize>`, `SizeOf`, and rkyv's `Archive`,
  `Serialize` and
  `Deserialize` with `OrdRepr` on the archived form; for a `Copy`
  `repr(C)` word with no invalid
  bit pattern the archived form can be the word itself, as the compact
  `Exact` is, without the
  heap branch. Check `HasOne` on `Fixed` before building it: a window
  whose unit is above 1 holds
  no 1, so either the impl is bounded to windows that hold it or the
  rustdoc says which `Fixed`
  can be a DBSP weight. The dbsp leg builds only on 1.99.0 (`cargo +1.99.0
  test --features dbsp
  --test main dbsp_`). The verify wants `contains src/wide.rs
  /differential/`: a
  `#[cfg(feature = "differential")] mod differential;` declared from
  src/wide.rs with the impls
  in src/wide/differential.rs satisfies it, and src/wide.rs is 562 lines
  already, past the
  500-line target, so the impls do not go inline; src/fixed.rs (435 lines)
  the same way for
  symmetry.
  
  Tests, named `differential_wide_*` and `differential_fixed_*` (the
  verify filters on
  `differential_wide_`), in a new topic file with its `mod` line in
  tests/main.rs
  (scripts/check-test-topics.sh fails the commit gate otherwise), reusing
  tests/differential.rs's
  `pub(crate)` helpers (`netted`, `sorted`, `share`, `sum_dataflow`) and
  tests/differential_products.rs's `pairs()` and `feed_epochs`: the five
  histories of
  docs/OVERFLOW.md on both pairs complete on 3 workers with `Wide` as the
  diff and read the small
  value exactly; with `Fixed256` on the 1e12 and 1e-12 pair, while
  `Fixed256::try_from(2^1000)` is
  refused at the door (`AboveCeiling`) before any dataflow starts,
  asserted as the per-value check
  doing its job; a product fold through `deposit_product` at the door; a
  retraction epoch netting
  to nothing; `join` with `isize` in either order. Mark the 3-worker tests
  `#[cfg_attr(miri, ignore = "three timely workers exchanging serialized
  batches")]` as the
  others are. Red first: the verify's `differential_wide_` filter runs no
  test today.
  
  Bench: a `Wide` row on the same-scale fixture in benches/dd_sum.rs,
  recorded in BENCH.md § The
  end-to-end SUM beside the plain and banded rows; the rough figure is
  docs/OVERFLOW.md § Cost's
  172–236 ns an update, 8× the `Exact` row and 16× `isize`. A sitting
  needs the host quiet
  (`uptime`, and `ps` for other cargo or criterion processes, first) and
  is measure, record,
  explain, stop: a row past expectations is a finding for the owner, not a
  build to change.
  
  Docs in the same commits: DESIGN.md § Wide and § Fixed, both trilemma
  rows, README § In an engine
  on when the `Copy` diff is the one to reach for (an engine whose weight
  slot must be `Copy` or
  that casts over a mapping; a declared window with a per-value door
  check).
  
  Settled this session: docs/OVERFLOW.md is shortened on the owner's
  ruling (fl-5h12vf3,
  fl-62b32y9), its § Options replaced by § What was decided, which names
  this task as what gives
  `Wide` and `Fixed` the traits; the push of main to origin for leras
  (fl-4fcbtkx) is blocked at
  the owner's word, "ignore the push to main for now", and stays blocked
  until the owner reopens
  it. Neither touches this task.
---

With the heap arm (owner, 2026-10-15: "yes let's do heap and allow product fols inside a
dataflow") `Exact` holds every sum and every product fold as a diff, so `Wide` is no longer the
carrier for product folds inside an engine. What remains for this task: `Wide` as the `Copy` diff
for an engine whose weight slot must be `Copy` or castable from a mapping (the role the plan gave
it: leras's `Monoid` is `Copy`), and `Fixed<LSB, LIMBS>` as the 32-byte declared window with a
per-value door check and no allocation ever. Neither has an engine trait today; the scratch
crates gave each a newtype with five of differential's.

To build, after the heap arm lands: the differential impls on `Wide` and on `Fixed<LSB, LIMBS>`
in the shape of src/exact/differential.rs (`Multiply<Wide> for isize`/`i64` too; no
`Wide × Wide`, stated in the rustdoc with the bit count), the DBSP set in the shape of
src/exact/dbsp.rs behind `dbsp`; `tests/differential.rs` tests named `differential_wide_*` and
`differential_fixed_*`: the five histories on both pairs complete on 3 workers, a product fold
through `deposit_product` at the door, and `Fixed256` refusing 2¹⁰⁰⁰ at the door; a
`benches/dd_sum.rs` row for `Wide` on the same-scale fixture recorded in BENCH.md § The
end-to-end SUM (the rough figure is 198 ns an update against `isize`'s 12.8, docs/OVERFLOW.md
§ Cost). DESIGN.md § Wide and § Fixed and the trilemma row say which engine traits each carries.

## log
- 2026-10-15T15:15Z created
- 2026-10-16T17:06Z handoff by claude (84727215-1ab8-4972-b9d9-cd4dd0eb435e)
- 2026-10-16T17:24Z handoff by claude (84727215-1ab8-4972-b9d9-cd4dd0eb435e)
