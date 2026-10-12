---
id: fl-v6mh9cv
title: "Ship the differential feature — differential-dataflow 0.24 trait impls, the three-worker integration test and the explode-and-count example"
category: crate/adapters
labels: [reproducible-floats, rf-critical-path]
docs: [docs/DESIGN.md#differential-dataflow-sp-differential]
needs: [fl-a2wrx7w]
seq: 90
verify: run cargo test package=float-abel target=main differential_
status: open
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

## comments
- 2026-10-12T13:23Z [claude (f24d940b-27ff-4906-b27c-e6a7b31fddaf)] Supply-chain fact to rule on before this feature lands: differential-dataflow 0.24.0 depends on timely ^0.30, and timely 0.30.0 lists bincode ^1.3 as a normal dependency (crates.io dependency lists read 2026-10-12). bincode 1.3.3 carries RUSTSEC-2025-0141 (unmaintained; its team calls 1.3.3 complete), and deny.toml runs with all-features, so cargo-deny will fail on this feature's closure. The owner ruled unmaintained code out when the wire task used bincode as a dev-dependency (fl-a2wrx7w's comments), so this is a conflict to surface, not an ignore to add: the choices are the owner's, an ignore scoped to timely's transitive use, waiting on upstream, or not shipping the feature at that pin.
- 2026-10-12T13:25Z [claude (f24d940b-27ff-4906-b27c-e6a7b31fddaf)] The owner's words, session f24d940b, on bincode 1.3.3 as a dev-dependency: "using unmaintained code is a code smell. seriously nothing we can use that doesn't have a security advisory warning against its use???" That is the ruling the comment above paraphrases; timely 0.30's own bincode ^1.3 dependency is the conflict it leaves for this task.
