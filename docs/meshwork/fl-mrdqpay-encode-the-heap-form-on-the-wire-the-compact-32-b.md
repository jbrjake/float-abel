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
handoff: |
  Next session encodes the heap form on the wire here. The heap arm is in
  at 09c92b5 (search
  `Findings` on fl-th7g69s for the shape and the day-one limits this task
  replaces). What stands
  today, all in src/exact/wire.rs and src/wire/serde_impls.rs: `to_bytes`
  is the compact form's and
  asserts by name on a heap value; `Exact::encode` (pub(crate), behind
  `alloc` + `serde`) writes a
  heap value as `HEAP_TAG` 0x04, then for the finite part and the three
  amounts a sign byte, an
  `i32` exponent and a `u32` limb count (`RUN_HEADER` = 9 bytes each),
  then the limbs little-endian;
  `Exact::decode` dispatches on the first byte to `from_bytes` or
  `decode_heap`, which checks every
  limb count against `EXACT_MAX_LIMBS` before any allocation, then the
  body length, then each run
  (sign byte 0 or 1, an empty run with no sign or exponent, an odd low
  limb, a nonzero top limb,
  the exponent bound), then lands through `Exact::land` and refuses a
  value that came back compact
  as `WireError::Form`; `WireError::Limbs` and `WireError::Form` are the
  two new variants, with
  their Display in src/wire.rs. The serde visitor's `visit_seq` gathers a
  heap body whole when the
  first element is the tag. tests/exact_heap_ring.rs's
  `exact_heap_ring_serde_*` test pins that
  body by hand (`heap_body`) and the refusals by their messages; move or
  rewrite it when the
  encoding is yours. The design's shape at 622ad8a was `(tag, exp, sign,
  limb count, limbs,
  special amounts)` normalised, and DESIGN.md § Wire format and § Open
  questions say the heap
  form's encoding is this task's.
  
  The pieces to read before changing: `Parts::finite`/`amount` and
  `Exact::land` in
  src/exact/heap.rs (a decoder builds `big::Owned` runs and lands them, so
  canonical form is
  checked by the same code every operation uses); `Owned::from_magnitude`
  in src/exact/big.rs if
  the encoding carries a two's-complement body instead of sign-magnitude.
  Keep `to_bytes` for the
  compact form, 33 bytes and the committed vectors unchanged
  (tests/wire.rs asserts them). For a
  heap `to_bytes` the signature has to change, since there is no fixed
  length: `Vec<u8>` behind
  `alloc`, or a writer taking `&mut Vec<u8>`; whichever it is,
  `benches/wire.rs`'s `AsTuple` and
  `AsFields` newtypes call `to_bytes` and expect 33 bytes.
  
  rkyv: `Exact` still derives `rkyv::Archive` and `rkyv::Serialize` behind
  `dbsp`, so a heap word
  archives its six integers (the pointer among them) and
  `ArchivedExact::value` refuses it by name
  through `Exact::canonical`; tests/dbsp.rs's `mixed` filters to compact
  values for the archive
  tests and
  `dbsp_weighted_batch_holds_a_promoted_weight_and_demotes_it_on_merge`
  shows a heap
  weight in an in-memory batch. A hand-written `Archive` needs an `unsafe
  fn resolve`, which the
  crate denies outside src/simd/ and src/exact/heap.rs, so it goes in a
  submodule of heap.rs or the
  ruling on `unsafe` modules is read back first. `size_of_children`
  already counts the block.
  
  Golden: tests/golden/main.rs records a heap value as tag 0x24 and its
  reads with no bytes
  (`Transcript::exact`), `Stats::exact_promoted` replaces the refusal
  count, BYTES is 65,385,705
  and the hash 56de810b… was blessed on aarch64 macOS at 09c92b5 with
  the other six targets left
  to CI, so this task re-blesses once more with the heap bytes and the
  wasm32 leg run locally
  (the build-environment memory has the recipe). Fuzz:
  fuzz/fuzz_targets/decode_robustness.rs's
  `check` only needs `Clone` now; it feeds `from_bytes` and postcard with
  the version byte forced
  to 0x01, so a heap body never reaches it, and the target gains the heap
  encoding here.
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
- 2026-10-15T17:47Z handoff by claude (fa2db8b2-1866-4c7c-86e2-74f6ae52f278)
