---
id: fl-06sggdc
title: "Ship the dbsp feature — dbsp 0.364 impls, OrdRepr, the IsNone derive, the WSet and aggregate_linear tests, and the CREATE LINEAR AGGREGATE example"
category: crate/adapters
labels: [reproducible-floats, rf-tail]
docs: [docs/DESIGN.md#dbsp-sp-dbsp]
needs: [fl-a2wrx7w]
seq: 130
verify: cargo +1.99.0 test -p float-abel --features dbsp --test main dbsp_ 2>&1 | grep -qE 'ok. [1-9][0-9]* passed'
status: open
created: 2026-10-11T15:02Z
covers:
  - ref: docs/DESIGN.md#sp-dbsp
    sha: 333ac4495b0d8de25950bbf75aa10f3ad8bcae9ad69833a7e81a47581f01f083
relates: [fl-v7t6xhz, fl-ceyy1h6]
handoff: |
  The tree is at 0ae1731 plus the store commit after it; the simd feature
  is closed and
  nothing of the dbsp feature is in the tree yet: Cargo.toml's `dbsp = []`
  is an empty placeholder,
  and its two comments ("builds only on rustc 1.96.1") are stale per this
  task's first comment, which
  probed 0.364.0 building on 1.96.1 and 1.99.0 with 1.97 and 1.98 ICEing.
  Toolchains here: 1.99.0 and
  1.74.0 installed (`rustup toolchain list`); `.github/workflows/ci.yml`
  already has a `dbsp` job
  pinned to 1.99.0 with the ICE comment at its head, so the job needs its
  steps checked against what
  lands, not created.
  
  The differential adapter is the model for the shape:
  src/exact/differential.rs holds the trait
  impls behind `#[cfg(feature = "differential")]`, src/exact/mod.rs
  declares it, src/lib.rs re-exports
  nothing extra for it, and tests/differential.rs plus examples/dd_sum.rs
  are its runtime proof. Mirror
  that: src/exact/dbsp.rs, a `mod dbsp` in tests/main.rs with every test
  named `dbsp_*`, and
  examples/dbsp_uda/.
  
  Order, with the owner-scoped points called out:
  
  1. Re-run the design's probe at the pin before writing an impl, in a
  scratch crate in the scratchpad
  depending on dbsp 0.364.0 under `cargo +1.99.0`: `OrdIndexedWSet`
  consolidation of `0.1 + 0.2`
  and its exact cancellation on merge, `aggregate_linear` scaling by
  `MulByRef<i64>` and
  retracting a group on an exact zero, and the three deliberate breaks
  (`IsNone`/`SupportsRoaring`,
  `OrdRepr`, `HasOne`/`Mul`/`Neg`). The design's recorded probe was on
  0.356.0 and 1.96.1.
  2. The dependency line is owner-scoped: differential is on a caret
  (owner: "0.25.1") so a consumer's
  own minor unifies; the same argument applies to dbsp, but it is a
  derivation until the owner
  says so. Ask with the probe's facts in hand, in one question, together
  with point 3.
  3. The lockfile: after `cargo update` pulls dbsp's closure, `cargo
  +1.74.0 build --locked` on the
  default features will likely fail on an edition-2024 manifest somewhere
  in it (second comment).
  Run that build right after adding the dependency and surface the result
  as a conflict (a
  separate MSRV lockfile, or the owner's ruling on the MSRV), never a
  decision.
  4. Impls per DESIGN.md § DBSP on `Exact` only; `OrdRepr`
  property-tested against `Ord` on 10⁵ values,
  mixed tiers and specials included; the `IsNone` derive.
  5. Owed from "Build Binned<K>", this task's third comment: the
  compile-fail case "Binned<3> fails to compile as a DBSP OrdIndexedWSet
  weight" in the trybuild harness beside
  tests/ui/binned_as_differential_diff.rs, with the same
  fragment on `Exact` as the pass case, under `#[cfg(feature = "dbsp")]`.
  Note fl-ceyy1h6: the
  .stderr snapshots match rustc 1.99 only, which is the toolchain this
  feature runs on anyway.
  6. examples/dbsp_uda/ with the accumulator type, `_map` and `_post` per
  Feldera's udf.md; the
  CLAUDE.md command line `cargo +1.99.0 test --features dbsp --test main
  dbsp_` is this task's
  verify and is already in the file.
  
  Also open, filed this session: the SIMD product-kernel question (search
  "deposit_products gets"),
  an owner-gated hold with a recommendation to wait for Measure's row;
  quote it to the owner when
  they are in session. Measure is unblocked now that the simd feature is
  closed.
---

The impls per DESIGN.md § DBSP, against dbsp 0.364.0 on rustc 1.99.0. The design's recorded probe ran
against 0.356.0 on 1.96.1, so re-run it at the pin first: `OrdIndexedWSet` consolidation of
`0.1 + 0.2` and its exact cancellation on merge, `aggregate_linear` scaling by `MulByRef<i64>` and
retracting a group on an exact zero, and the three deliberate breaks. In `tests/main.rs`, a
`mod dbsp`, with the three breaks kept as negative cases (`IsNone`/`SupportsRoaring`, `OrdRepr`,
`HasOne`/`Mul`/`Neg`). `OrdRepr` is property-tested against `Ord` on 10⁵ values, mixed tiers and
specials included. `examples/dbsp_uda/` holds the accumulator type, `_map` and `_post`, per
Feldera's `udf.md`. The CI job pins rustc 1.99.0 and says why: 1.97 and 1.98 ICE inside dbsp.

The verify is shell because the DSL's `run` takes no toolchain and no `--features`.

## log
- 2026-10-11T15:02Z created
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-dbsp @89f0f9880ba8
- 2026-10-11T16:06Z cover docs/DESIGN.md#sp-dbsp @5900331b1262
- 2026-10-11T18:13Z cover docs/DESIGN.md#sp-dbsp @333ac4495b0d
- 2026-10-12T21:37Z handoff by claude (357feee9-5111-41e9-8f17-91f2ac458b5c)
- 2026-10-12T21:38Z handoff by claude (357feee9-5111-41e9-8f17-91f2ac458b5c)
- 2026-10-12T21:38Z handoff by claude (357feee9-5111-41e9-8f17-91f2ac458b5c)

## comments
- 2026-10-11T15:55Z [claude (85c2609c-6abc-4455-b7f3-6c1c79eda92c)] Toolchain probe, 2026-10-11, scratch crate depending on dbsp with a one-line lib: dbsp 0.356.0 and 0.364.0 (current; both declare rust-version 1.96.1) ICE in rustc 1.97.1 and 1.98.0-beta.6 at rustc_next_trait_solver/src/solve/assembly/structural_traits.rs:1005 ('could not replace AliasTerm', vtable slot of operator::dynamic::multijoin::star_join::StarJoinFuncTrait). dbsp 0.364.0 builds on 1.96.1 and on 1.99.0 (current stable; Feldera's own rust-toolchain.toml pins 1.99.0). So DESIGN.md § Crate posture's 'builds only on rustc 1.96.1' is stale: 1.97 and 1.98 ICE, 1.96.1 and 1.99 build. Owner's call whether the dbsp CI job and CLAUDE.md move the pin to 1.99.0; the 0.356.0 probe was run on 1.96.1 and would be re-run there first.
- 2026-10-12T13:55Z [claude (51dfb626-8f8b-48f4-9e4a-68528d76dabd)] Lockfile constraint found while wiring the laws feature, 2026-10-12: cargo 1.74 parses every manifest in Cargo.lock on a normal edge, optional features included, so an edition-2024 crate anywhere in dbsp 0.364's closure breaks the MSRV job (cargo +1.74.0 build --locked on the default features) even though that build never compiles it. proptest 1.12's rand 0.10 closure did exactly that, and the lock pins proptest 1.11.0. dbsp's closure is large and recent; expect this to bite, and surface it as a conflict (a separate MSRV lockfile, or the owner's ruling on the MSRV) rather than deciding.
- 2026-10-12T15:10Z [claude (745519d7-d30a-4336-9fa7-1462279793db)] Owed from "Build Binned<K>" (fl-v7t6xhz): the DBSP half of its compile-fail contract, "Binned<3>
  fails to compile as a DBSP OrdIndexedWSet weight". It could not be written before this feature
  exists. Add a `compile_fail` case to the trybuild harness in tests/ui.rs beside
  tests/ui/binned_as_differential_diff.rs, which is the differential half: a fragment that names
  `Binned<3>` as an `OrdIndexedWSet` weight (or whatever the weight bound of the pin is, `ZRingValue`
  or `DBWeight`) with the same fragment on `Exact` as the pass case, so the failure is the missing
  trait and not a missing feature. The case needs the feature on in the harness, which means the
  `ui_` test that holds it runs only under `--features dbsp` on rustc 1.99.0, as this task's verify
  does; gate it with `#[cfg(feature = "dbsp")]`.
