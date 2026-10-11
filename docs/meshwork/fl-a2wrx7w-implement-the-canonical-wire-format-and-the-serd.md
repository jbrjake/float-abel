---
id: fl-a2wrx7w
title: "Implement the canonical wire format and the serde feature, with decoders that refuse non-canonical input"
category: crate/wire
labels: [reproducible-floats, rf-critical-path]
docs: [docs/DESIGN.md#wire-format-sp-wire]
needs: [fl-atkbp6q]
seq: 70
verify: run cargo test package=float-abel wire_
status: open
created: 2026-10-11T15:02Z
covers:
  - ref: docs/DESIGN.md#sp-wire
    sha: 29c92597605490927ffd107afd12b7afd03641bdb17d98526a3f7cc7ae4c02f4
---
Per DESIGN.md § Wire format: `FORMAT.md` in bitrep's style; `to_bytes` and `from_bytes` on all three
exact types; `serde` behind its feature, encoding the same bytes.

Tests `wire_*`:
- round-trip;
- a hand-built non-canonical `Exact` is refused with a typed error, never a panic. Cover an even
  mantissa, a leading zero limb, and a compact-representable value encoded as the heap tier;
- bincode 1.3 fixed-int round-trip under the feature.

## log
- 2026-10-11T15:02Z created
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-wire @29c925976054
