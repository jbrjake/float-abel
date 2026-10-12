---
id: fl-06sggdc
title: "Ship the dbsp feature — dbsp 0.364 impls, OrdRepr, the IsNone derive, the WSet and aggregate_linear tests, and the CREATE LINEAR AGGREGATE example"
category: crate/adapters
labels: [reproducible-floats, rf-tail]
docs: [docs/DESIGN.md#dbsp-sp-dbsp]
needs: [fl-a2wrx7w]
seq: 130
verify: cargo +1.99.0 test -p float-abel --features dbsp --test main dbsp_ 2>&1 | grep -qE 'ok. [1-9][0-9]* passed'
status: done
created: 2026-10-11T15:02Z
covers:
  - ref: docs/DESIGN.md#sp-dbsp
    sha: 643bdf811a985a765d133d9b859d9d1436d7e5ef6995d2f10e4f23af85556693
relates: [fl-v7t6xhz, fl-ceyy1h6]
attachments: [attachments/fl-06sggdc/dbsp-probe-lib.rs]
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
- 2026-10-12T22:17Z open→doing — claimed by claude (70076334-fceb-445b-b3a5-700374edbd88)
- 2026-10-12T22:39Z cover docs/DESIGN.md#sp-dbsp @643bdf811a98
- 2026-10-12T22:40Z doing→done — verify exit 0 @ afccf59+5

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
- 2026-10-12T22:17Z [claude (70076334-fceb-445b-b3a5-700374edbd88)] Probe re-run at the pin, 2026-10-12: dbsp 0.364.0 on rustc 1.99.0, scratch crate in the session scratchpad (its src/lib.rs is attached to this task as dbsp-probe-lib.rs). The weight is a six-integer word with Exact's layout under rkyv 0.7.46's derive, converted through Exact's wire bytes, with four newtypes over it: `W` carrying every trait the design lists, and `NoNone`, `NoOrdRepr`, `NoRing` each dropping one group behind a `break_*` cargo feature. Results, all observed this session:
  
  - `cargo +1.99.0 test`: 4 passed. (1) `OrdIndexedWSet<u32, u32, W, DynWeightTyped<W>>::from_tuples` of `0.1` and `0.2` on one key consolidates to one tuple carrying `Exact(0.1) + Exact(0.2)`; merging it with a batch of `-0.2, -0.1` leaves no tuple, where the f64 control `0.1 + 0.2 - 0.2 - 0.1` leaves residue. (2) `aggregate_linear(|bits| W(Exact::from(f64::from_bits(*bits))))` over an `add_input_indexed_zset::<u32, u64>` scales by the `ZWeight`: `(0.1, +1)` and `(0.2, +3)` land `0.1 + 0.2 × 3` as the group's value with weight +1; retracting both emits the retraction `(1, total, -1)` and nothing else, so the exact-zero group is deleted, not re-emitted. (3) `Archived<W>::ord_cmp` agrees with `Ord` on a 9-value cross product with NaN, ±∞ and ±1e300 in it. (4) The word round-trips every special through the bytes.
  - The three breaks fail on the predicted bound, each as the only error: `break_none` → `NoNone: DBWeight` unsatisfied, "the trait `IsNone` is not implemented" and the same for `SupportsRoaring`; `break_ordrepr` → "`ArchivedNoOrdRepr: OrdRepr<NoOrdRepr>` is not satisfied"; `break_ring` → `Stream::neg` "trait bounds were not satisfied: doesn't satisfy `NoRing: HasOne`, `Mul` or `Neg`". At 0.364.0 `ZRingValue` bounds no public operator directly; it reaches `neg`, `minus` and `differentiate` through `Leaf<K, R>: Neg where R::Type: DBWeight + ZRingValue` (trace/layers/leaf.rs).
  - The probe crate carries `#![deny(unsafe_code)]` and builds: rkyv's derive expands `unsafe fn resolve` inside an external macro, which the lint does not report. So the adapter can derive `Archive`/`Serialize`/`Deserialize` on `Exact` without lifting the crate-wide deny.
  - `IsNone` and `SupportsRoaring` come from dbsp's own exported macros, `dbsp::never_none!(W)` and `dbsp::never_roaring_filter!(W)`, which expand to exactly what `#[derive(feldera_macros::IsNone)]` emits (feldera-macros 0.364.0 src/lib.rs: an `IsNone` impl with `Inner = Self` and an empty `SupportsRoaring` impl). The feature therefore names dbsp, rkyv and feldera-size-of as direct dependencies and not feldera-macros; DESIGN.md § Features' row changes to match.
  - dbsp 0.364.0 is `edition = "2024"`, `rust-version = "1.96.1"`, `rkyv = 0.7.45` with `features = ["std", "size_64", "validation", "uuid"]`, `size-of = { package = "feldera-size-of", version = "0.1.7" }`. The probe's closure is 440 packages against the crate's 114.
  - `iter()` exists only on `IndexedZSetReader`, i.e. `R = ZWeight`; a custom-weight batch is read through `inner().cursor()` with `downcast_checked` on the key and value and `Deref` on `DynWeightTyped<W>`, no unsafe.
- 2026-10-12T22:40Z [claude (70076334-fceb-445b-b3a5-700374edbd88)] Shipped at afccf59, 2026-10-12, on dbsp 0.364.0 and rustc 1.99.0. What landed: src/exact/dbsp.rs (HasZero, HasOne, MulByRef<i64>/<isize>, SizeOf, never_none!/never_roaring_filter!, and on the archived form Deserialize, Eq, Ord and OrdRepr through Exact::canonical, now pub(super) in src/exact/wire.rs), the rkyv Archive/Serialize derive on Exact under cfg_attr in src/exact/mod.rs, tests/dbsp.rs (seven dbsp_ tests: consolidation and cancellation on an OrdIndexedWSet, aggregate_linear scale and zero-group deletion on two workers, specials, OrdRepr against Ord on 10^5 mixed values with every ordering seen 1,000+ times, the archived word as the wire body, the non-canonical refusal by the deserialiser and both comparisons, and the ring constants), the trybuild pair tests/ui/{exact,binned}_as_dbsp_weight.rs with the blessed .stderr (Binned fails on SizeOf, IsNone, SupportsRoaring, HasZero, Serialize and the monoid), examples/dbsp_uda/{main.rs,program.sql,udf.toml}, and the gate/CI legs. Verify watched: `cargo +1.99.0 test --features dbsp --test main dbsp_` → ok. 8 passed; `cargo run --features dbsp --example dbsp_uda` prints 0.6 and 1 where f64 would print 0.6000000000000001 and 0; `cargo +stable clippy --features dbsp --all-targets -- -D warnings` clean; bare `cargo test` 164 passed; `cargo +1.74.0 build --locked` passes with the 488-package lockfile, so the lockfile fear in the second comment did not bite at this pin.
  
  Decisions taken inside the design's words, for the owner to overrule: feldera-macros is not a dependency (dbsp's exported macros emit the same impls); rkyv's Deserialize is hand-written so a non-canonical archived word panics with the invariant named rather than entering the group, because dbsp unwraps its deserialiser; the archived-root read in tests/dbsp.rs is one `unsafe` with a SAFETY line rather than a bytecheck dependency for CheckBytes; the push gate gained a dbsp leg (tests and the example on 1.99.0 when installed, skipped loudly otherwise) and CI's dbsp job lints the feature and runs the example.
  
  Owner-scoped, filed: fl-jmjbjk5 (deny.toml for the closure: eight licences, thirty duplicate majors, RUSTSEC-2026-0235 in rkyv 0.7; the push gate is red at cargo-deny until it is ruled, and the release task needs it) and fl-yzy8bwc (the dependency line, exact pin or caret; the tree ships the design's `=0.364.0`).
