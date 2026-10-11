---
id: fl-06sggdc
title: "Ship the dbsp feature — dbsp 0.356 impls, OrdRepr, the IsNone derive, the WSet and aggregate_linear tests, and the CREATE LINEAR AGGREGATE example"
category: crate/adapters
labels: [reproducible-floats, rf-tail]
docs: [docs/DESIGN.md#dbsp-sp-dbsp]
needs: [fl-a2wrx7w]
seq: 130
verify: cargo +1.96.1 test -p float-abel --features dbsp --test main dbsp_ 2>&1 | grep -qE 'ok. [1-9][0-9]* passed'
status: open
created: 2026-10-11T15:02Z
covers:
  - ref: docs/DESIGN.md#sp-dbsp
    sha: 89f0f9880ba8d82159363f276a61f7b8b1be84f2f0b4ea7924b052eea349b0d4
---
The impls per DESIGN.md § DBSP. In `tests/main.rs`, a `mod dbsp`, with three deliberate breaks kept
as negative cases (`IsNone`/`SupportsRoaring`, `OrdRepr`, `HasOne`/`Mul`/`Neg`). `OrdRepr` is
property-tested against `Ord` on 10⁵ values, mixed tiers and specials included. `examples/dbsp_uda/`
holds the accumulator type, `_map` and `_post`, per Feldera's `udf.md`. The CI job pins rustc 1.96.1
and says why.

The verify is shell because the DSL's `run` takes no toolchain and no `--features`.

## log
- 2026-10-11T15:02Z created
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-dbsp @89f0f9880ba8
