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
