---
id: fl-v6mh9cv
title: "Ship the differential feature — differential-dataflow 0.24 trait impls, the three-worker integration test and the explode-and-count example"
category: crate/adapters
labels: [reproducible-floats, rf-critical-path]
docs: [docs/DESIGN.md#differential-dataflow-sp-differential]
needs: [fl-a2wrx7w]
seq: 90
verify: run cargo test package=float-abel target=main differential_
status: done
created: 2026-10-11T15:02Z
covers:
  - ref: docs/DESIGN.md#sp-differential
    sha: 08fc72836ea13c6f93a6666e076c1960cd90057ddd07f23b369fb245d847a74a
---
The impls per DESIGN.md § differential-dataflow. In `tests/main.rs`, a `mod differential` behind the
feature: `ProcessBinary(3)`, every operator listed there, the `explode` + `count_total` idiom, an
`antijoin` with `isize`, and a retraction epoch whose netted output is empty. Plus
`examples/dd_sum.rs`.

## log
- 2026-10-11T15:02Z created
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-differential @08fc72836ea1
- 2026-10-12T13:59Z handoff by claude (51dfb626-8f8b-48f4-9e4a-68528d76dabd)
- 2026-10-12T14:00Z handoff by claude (51dfb626-8f8b-48f4-9e4a-68528d76dabd)
- 2026-10-12T14:14Z open→doing — claimed by claude (979ec15b-15d7-418f-857e-2b3284a67604)
- 2026-10-12T14:39Z doing→done — verify exit 0 @ 911bf92+1

## comments
- 2026-10-12T13:23Z [claude (f24d940b-27ff-4906-b27c-e6a7b31fddaf)] Supply-chain fact to rule on before this feature lands: differential-dataflow 0.24.0 depends on timely ^0.30, and timely 0.30.0 lists bincode ^1.3 as a normal dependency (crates.io dependency lists read 2026-10-12). bincode 1.3.3 carries RUSTSEC-2025-0141 (unmaintained; its team calls 1.3.3 complete), and deny.toml runs with all-features, so cargo-deny will fail on this feature's closure. The owner ruled unmaintained code out when the wire task used bincode as a dev-dependency (fl-a2wrx7w's comments), so this is a conflict to surface, not an ignore to add: the choices are the owner's, an ignore scoped to timely's transitive use, waiting on upstream, or not shipping the feature at that pin.
- 2026-10-12T13:25Z [claude (f24d940b-27ff-4906-b27c-e6a7b31fddaf)] The owner's words, session f24d940b, on bincode 1.3.3 as a dev-dependency: "using unmaintained code is a code smell. seriously nothing we can use that doesn't have a security advisory warning against its use???" That is the ruling the comment above paraphrases; timely 0.30's own bincode ^1.3 dependency is the conflict it leaves for this task.
- 2026-10-12T13:55Z [claude (51dfb626-8f8b-48f4-9e4a-68528d76dabd)] Lockfile constraint found while wiring the laws feature, 2026-10-12: cargo 1.74 parses every manifest in Cargo.lock that sits on a normal edge, optional features included, so the MSRV job (cargo +1.74.0 build --locked on the default features) fails with 'this version of Cargo is older than the 2024 edition' as soon as any feature's dependency closure contains an edition-2024 crate, even though that build never compiles it. proptest 1.12's rand 0.10 closure did exactly that; the lock pins proptest 1.11.0. Before this feature lands, check differential-dataflow 0.24's and timely 0.30's closure editions at the versions the lock would take; if any is 2024, the MSRV job needs a different shape (a separate lockfile for the MSRV build, or the owner's ruling on raising the MSRV), and that is a conflict to surface, not a decision.
- 2026-10-12T14:24Z [claude (979ec15b-15d7-418f-857e-2b3284a67604)] Two owner rulings, this session (979ec15b), 2026-10-12, given through one structured question after the facts were read back:
  
  1. Version. The owner asked "why are we using 0.24 instead of 0.25 dd?". Read back: the weight-trait file (src/difference.rs) is byte-identical between 0.24.0 and 0.25.1; the only signature changes are internal generics on `join_core` and `count_total_core`; the closure is the same 28 packages with the same two advisories and no edition-2024 crate at either version; a consumer on 0.25 cannot unify a 0.24 adapter. Asked "Which differential-dataflow version should the `differential` feature adapt to?", the owner chose "0.25.1 (Recommended)". So the feature pins `differential-dataflow = "0.25"` (timely 0.31), and DESIGN.md § Features and § Crate posture say 0.25.
  
  2. cargo-deny. The closure carries two RUSTSEC unmaintained advisories, not one: bincode 1.3.3 (RUSTSEC-2025-0141, through timely) and paste 1.0.15 (RUSTSEC-2024-0436, through columnation and differential-dataflow itself), at 0.24.0 and 0.25.1 alike; neither is a direct dependency. The options read back were ignore both by id, `unmaintained = "workspace"` (verified green on the closure with cargo-deny 0.20.2), hold the feature, or drop it. The owner chose "Ignore both by id (Recommended)": two `ignore` rows in deny.toml, each naming the advisory and the crate that drags it in, so a third advisory still fails the gate.
  
  The duplicate `syn` major (2.0.119 through columnar_derive 0.13.2; 3.0.x through serde_derive) is handled under deny.toml's own rule, a `skip` row naming the dragging crate; it was not part of the ruling.
- 2026-10-12T14:39Z [claude (979ec15b-15d7-418f-857e-2b3284a67604)] Built in session 979ec15b, on the two rulings in the comment above, and committed at 911bf92. `./scripts/regression.sh` exit 0 on that tree, run with the sandbox off for cargo-deny's advisory fetch: fmt, the three clippy legs, rustdoc, every test with `--include-ignored` (31 lib, 98 in tests/main.rs, the doctest), the release headroom leg, tests/noalloc, the four trybuild cases with no re-bless needed, the MSRV 1.74.0 build of the default features, and cargo-deny advisories, bans, licenses and sources ok. The commit gate at the commit: 129 tests. What is in the tree:
  
  - Cargo.toml: `differential-dataflow = "0.25"` optional on a caret (a consumer's 0.25.x must unify with it, since the impls are a diff type's traits), `differential = ["dep:differential-dataflow", "serde"]`, a dev-dependency `timely = "0.31"` because differential-dataflow does not re-export timely and the tests and the example execute dataflows, and `[[example]] dd_sum` with `required-features = ["differential"]`. Cargo.lock gained the closure's 19 packages (bincode, bytemuck, byteorder, columnar, columnar_derive, columnation, differential-dataflow, either, fnv, getopts, itertools, paste, smallvec, timely and its four subcrates, unicode-width), none edition 2024, and the MSRV 1.74 build of the default features still parses the lock.
  - src/exact/differential.rs, behind the feature: `IsZero`, `Semigroup`, `Monoid`, `Abelian`, `Multiply<Exact>`, `Multiply<i64>` and `Multiply<isize>` on `Exact`, and `Multiply<Exact>` for `i64` and `isize`, each the group operator it names. src/exact/ops.rs: `From<i8>` on every feature set, by integer arithmetic (`Compact::from_i64`), because a std trait impl that appears only with a feature would change a consumer's inference when the feature is turned on.
  - tests/differential.rs, eleven tests named `differential_*`, every dataflow on `ProcessBinary(3)` with serialized exchange: the explode + count_total SUM against an integer-arithmetic oracle; one and three workers over forward and reversed input, byte-identical through `to_bytes`; a retraction epoch that nets to nothing, with an f64 control folding the same rows that leaves residue; NaN and +∞ counted, a duplicate row scaled by multiplicity two, the NaN retracted; map, flat_map, concat, negate, consolidate, arrange_by_key and arrange_by_self; reduce with `Exact` and with `isize` out; count, count_total, count_core::<Exact>, distinct_core::<Exact>, threshold and threshold_total; join `Exact × isize` and `isize × Exact`; semijoin and antijoin with `isize`; iterate with `distinct_core::<Exact>` in the loop; and `From<i8>`. Red first: the topic failed to compile on `Exact: Semigroup` and `Exact: Abelian` before the impls existed. A deliberate break of `Multiply<isize>` that ignored the multiplier failed exactly the retraction, join, semijoin/antijoin and specials tests and no other, then was reverted.
  - examples/dd_sum.rs: `cargo run --example dd_sum -- -w 3` prints account 1 total 0.6 (an f64 fold of 0.1, 0.2, 0.3 gives 0.6000000000000001) and account 2 total 1 (an f64 fold of 1e16, 1.0, -1e16 gives 0), then epoch 1 retracts both with nothing left.
  - deny.toml: the two advisory ignores by id, each naming the dragging crate, and a bans `skip` for `syn@2` (columnar_derive 0.13 is on syn 2; serde_derive and thiserror-impl on syn 3) under deny.toml's own rule for a duplicate major. The exact text was run green on a scratch resolve of 0.25.1 before it landed.
  - DESIGN.md § Features and § Crate posture now say 0.25. § differential-dataflow, the clause this task covers, is untouched, so the cover's pin holds; it still records the probe as run at 0.24.0 with the surface unchanged at 0.25.1, which the diff of src/difference.rs between the two confirmed byte for byte.
  - lib.rs's crate docs name the feature and the idiom; README's build block has the example line; CLAUDE.md's standing decisions carry both rulings in the owner's words.
