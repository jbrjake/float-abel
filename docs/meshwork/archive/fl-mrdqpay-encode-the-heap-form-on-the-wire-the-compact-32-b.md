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
status: done
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
- 2026-10-15T17:47Z handoff by claude (fa2db8b2-1866-4c7c-86e2-74f6ae52f278)
- 2026-10-16T13:29Z open→doing — claimed by claude (3f2084cf-748a-445c-adcf-4fe16d8173b8)
- 2026-10-16T14:04Z doing→done — verify exit 0 @ 2ca2ddd+2

## comments
- 2026-10-16T14:04Z [claude (3f2084cf-748a-445c-adcf-4fe16d8173b8)] Findings: built at 2ca2ddd, the push gate green on this machine
  (scripts/regression.sh: clippy on every configuration, the fuzz package's lint and oracle tests,
  rustdoc, the suite with the ignored tests, ui, the release every_build tests, noalloc, golden, the
  MSRV build, the dbsp tests and example on 1.99.0, both cargo-deny configs), `cargo +stable clippy
  --features dbsp --all-targets -- -D warnings` green, `cargo +nightly miri test --test main wire_`
  green (14 tests), and `cargo fuzz run decode_robustness -- -max_total_time=600` clean over
  82.3 million inputs.
  
  The encoding: `Exact::HEAP_TAG` (0x04), then the block as the word holds it, four header limbs
  (bit 63 the sign, bits 32–62 the limb count, bits 0–31 the exponent as an `i32`) for the finite
  part and the NaN, +∞ and −∞ amounts, then the four runs of limbs; 33 + 8·limbs bytes. One layout
  serves memory, the wire and the rkyv archive, so one validator, `heap::check_runs`, gates every
  block from outside: the wire decoder runs it over the bytes, after the header's counts are checked
  against the cap and the length they imply, and before it allocates, so every refusal allocates
  nothing (tests/wire_heap.rs counts allocations through a thread-local counting allocator in
  tests/support.rs); the archive runs it over its block. The day-one body (9-byte run headers) is
  gone with its test. FORMAT.md has the section and three vectors.
  
  API: `Exact::encode(&self) -> Vec<u8>` behind `alloc`, either form; `Exact::from_bytes` reads both
  forms, dispatching on the first byte, and without `alloc` refuses the tag as `Version(0x04)`;
  `to_bytes` unchanged, the compact form's array, panicking by name on a heap value; `HEAP_TAG` a
  `pub const` on every feature set. Refusals in order: `Length` under 33 bytes, `Limbs` on the
  header, `Length` against the header, then per run `Form` (sign or exponent on an empty run),
  `EvenMantissa`, `Form` (zero last limb), `Exponent`, and last `Form` for a value that fits the
  compact form (finite part ≤ 127 bits and every amount an integer inside `i32`, the operations' own
  rule, which tests/wire_heap.rs cross-checks against the operations on 2,000 draws).
  
  rkyv (DBSP): the derive is gone. `Archive` and `Serialize<S: Serializer>` are hand-written:
  `ArchivedExact` is a `repr(C)` 32-byte word; a compact value archives as the word (the wire body
  without its version byte, unchanged); a heap value archives with `lo` = the block's used length
  doubled, `hi` = the block's offset from the `hi` field (rkyv's relative-pointer convention, a
  multiple of 8), trailing fields zero, and the block written 8-aligned before the root. The word's
  structural checks (trailing fields zero, length within the header and `MAX_BLOCK`, offset aligned)
  run before the block is dereferenced; `Parts::canonical` then validates it; `Ord`, `Eq` and
  `OrdRepr` read the block in place without a copy, and `Deserialize` copies it into a word. The two
  raw-memory pieces, `resolve` and `ArchivedExact::block`, are in src/exact/heap.rs (the module's
  seventh and eighth `unsafe` blocks, each with its SAFETY line); src/exact/dbsp.rs is safe code.
  tests/dbsp.rs's `mixed` now keeps heap results (a fifth of 10⁵ values), the `OrdRepr` property
  test covers both forms and across them, a heap weight's archive layout and round trip are pinned,
  and a corrupted block or word is refused by name before the block is read where the word alone
  says so.
  
  Golden: tests/golden/main.rs records a heap value's `encode` bytes under tag 0x24, adds a fold of
  row products through `Exact` (9,592 of the 10⁴ multisets promote) and counts dyadic amounts
  (1,887); BYTES 74,365,506, hash 0cc95362…, blessed on aarch64 macOS and matched by the wasm32
  leg run locally under wasmtime 49.0.2; the other five targets are CI's.
  
  Fuzz: fuzz/fuzz_targets/decode_robustness.rs forces the heap tag on every input and checks the
  decoder against an independent `heap_verdict` re-derived from FORMAT.md, re-encodes accepted
  bodies, and compares the postcard path, which now frames `Exact` and `Wide` as postcard does
  (the previous target passed the raw buffer, which postcard would read as a varint length).
  
  Found and filed: fl-dchdzd4, the limb cap is not closed under retraction at its edge (a value of
  exactly `EXACT_MAX_LIMBS` full limbs decodes but `v − v` and `v × 1` refuse on the operands'
  spans plus a carry bit); tests/wire_heap.rs pins only the decoder's claim for that body.
  
  Not done: the deliberate-break pass on the decoder's rules (disabling a check in src to watch a
  test fail) was refused by the harness's classifier; the evidence that each assertion can fail is
  the three fixture failures seen while building (a 65-bit body expected accepted and refused as
  `Form`, the at-cap body's `checked_mul_i64(1)` refusal, and the miri threshold). The `Arbitrary`
  impl still draws compact values only; a heap-drawing strategy would let sum_vs_oracle reach the
  heap form and is a follow-up if wanted.
