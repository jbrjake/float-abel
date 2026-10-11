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
    sha: 10a66b73f9bdbcdef33cbfaca33d2deeff1e32a52775100d4759890a318552f6
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
- 2026-10-11T18:13Z cover docs/DESIGN.md#sp-wire @ed67fb6c74a2
- 2026-10-11T18:33Z cover docs/DESIGN.md#sp-wire @10a66b73f9bd
