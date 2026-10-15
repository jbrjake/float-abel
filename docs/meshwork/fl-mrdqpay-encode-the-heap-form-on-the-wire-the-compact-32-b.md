---
id: fl-mrdqpay
title: "Encode the heap form on the wire — the compact 32 bytes unchanged, a tagged heap encoding with its limb count ceilinged on decode, serde and rkyv over both, the decoder refusing a heap encoding that fits the compact form, the golden hash re-blessed and the decoder fuzz target extended"
category: crate/types
labels: [reproducible-floats, rf-critical-path]
discovered-from: fl-t154dsp
needs: [fl-th7g69s]
seq: 14
verify: run cargo test package=float-abel wire_heap_
docs:
  - docs/DESIGN.md#wire-format-sp-wire
  - docs/DESIGN.md#bit-identical-across-architectures-sp-architecture
  - docs/DESIGN.md#dbsp-sp-dbsp
status: open
created: 2026-10-15T15:33Z
---
With the heap arm (fl-heap) `Exact` has two forms, and DESIGN.md § Wire format's "`Exact` as its
32 bytes" covers one. Keep the compact encoding byte for byte so every committed vector and the
golden hash's compact legs hold, and add the heap encoding in the design's shape at 622ad8a:
`(tag, exp, sign, limb count, limbs, special amounts)` normalised. Decoding validates canonical
form and refuses anything else, as a typed `WireError`, never a panic: a heap encoding whose
value fits the compact form is non-canonical and refused, as are leading zero limbs, a limb
count past `EXACT_MAX_LIMBS` (an untrusted size gets the ceiling before any allocation), and a
non-normalised special amount. Through differential the encoding rides bincode via serde; through
DBSP rkyv 0.7, which archives `Box`: `Archive` and `Serialize` on the enum, `Deserialize`, `Eq`,
`Ord` and `OrdRepr` on the archived form hand-written through one validating constructor, as the
design's DBSP row says, property-tested against `Ord` on mixed forms and specials.

Tests `wire_heap_*` in tests/wire.rs (or tests/wire_heap.rs if the file crosses 500 lines): a
heap value round-trips through `to_bytes`/`from_bytes` and through serde to the same structural
value; the compact vectors are unchanged (the committed bytes compared, not regenerated); each
refusal above is a `WireError` by name with no allocation observed; the golden stream gains heap
vectors (promoted sums, a product fold, a dyadic special amount) and `tests/data/golden.sha256` is
re-blessed on this machine with the wasm32 leg run locally (the build-environment memory has the
recipe) and the other targets left to CI; `fuzz/fuzz_targets/decode_robustness.rs` takes the heap
encoding and runs ten minutes clean. DESIGN.md § Wire format, § Bit-identical across architectures
and § DBSP describe both forms.

## log
- 2026-10-15T15:33Z created
