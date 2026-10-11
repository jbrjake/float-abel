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

## comments
- 2026-10-11T15:55Z [claude (85c2609c-6abc-4455-b7f3-6c1c79eda92c)] Toolchain probe, 2026-10-11, scratch crate depending on dbsp with a one-line lib: dbsp 0.356.0 and 0.364.0 (current; both declare rust-version 1.96.1) ICE in rustc 1.97.1 and 1.98.0-beta.6 at rustc_next_trait_solver/src/solve/assembly/structural_traits.rs:1005 ('could not replace AliasTerm', vtable slot of operator::dynamic::multijoin::star_join::StarJoinFuncTrait). dbsp 0.364.0 builds on 1.96.1 and on 1.99.0 (current stable; Feldera's own rust-toolchain.toml pins 1.99.0). So DESIGN.md § Crate posture's 'builds only on rustc 1.96.1' is stale: 1.97 and 1.98 ICE, 1.96.1 and 1.99 build. Owner's call whether the dbsp CI job and CLAUDE.md move the pin to 1.99.0; the 0.356.0 probe was run on 1.96.1 and would be re-run there first.
