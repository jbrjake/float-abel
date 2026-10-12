---
id: fl-a2wrx7w
title: "Implement the canonical wire format and the serde feature, with decoders that refuse non-canonical input"
category: crate/wire
labels: [reproducible-floats, rf-critical-path]
docs: [docs/DESIGN.md#wire-format-sp-wire]
needs: [fl-atkbp6q]
seq: 70
verify: run cargo test package=float-abel wire_
status: done
created: 2026-10-11T15:02Z
covers:
  - ref: docs/DESIGN.md#sp-wire
    sha: 0134bdb26462843e2be18b0e7e4681417ecdf5fcff88e0969ed4f2dafe99c57e
relates: [fl-6shtzpj, fl-v7t6xhz, fl-cfx001b]
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
- 2026-10-12T12:53Z handoff by claude (be8fad00-93e4-42e3-8b8f-40c127bab5c7)
- 2026-10-12T13:02Z open→doing — claimed by claude (f24d940b-27ff-4906-b27c-e6a7b31fddaf)
- 2026-10-12T13:14Z cover docs/DESIGN.md#sp-wire @6219d60dd7a9
- 2026-10-12T13:15Z doing→done — verify exit 0 @ d6d3513+1
- 2026-10-12T13:23Z cover docs/DESIGN.md#sp-wire @0134bdb26462

## comments
- 2026-10-12T13:15Z [claude (f24d940b-27ff-4906-b27c-e6a7b31fddaf)] Built at 240e211 (code) and the deny ignore after it. Decisions taken inside § Wire's words, for the owner to overrule: the version byte leads and names the type as well as the revision (0x01 Exact, 0x02 Wide, 0x03 Fixed), so a Fixed256 body is refused by Exact::from_bytes rather than read as a dyadic when its limb 0 happens to be odd; and the serde form is a fixed-length tuple of the to_bytes bytes, as a [u8; N] serialises, so bincode 1.3's fixed-int encoding is to_bytes with no length prefix (tests/wire.rs asserts bincode::serialize(&x) == x.to_bytes()). Fixed::to_bytes takes its array length as a const generic pinned by a const assertion, because stable Rust cannot spell 8 * LIMBS + 1 in a return type; tests/ui/fixed_to_bytes_wrong_length.rs pins that. Two of the body's refusal cases are moot under the POD Exact: there is no limb vector for a leading zero limb and no heap tier to encode a compact value as; the cases covered are an even mantissa, a zero mantissa with an exponent, an exponent past ±EXACT_MAX_EXP, a count of i32::MIN, the version byte and the length. cargo-deny flagged bincode 1.3.3 as unmaintained (RUSTSEC-2025-0141, no vulnerability); deny.toml ignores that one id with the reason, since the design names bincode 1.3 as the differential adapter's wire. The trybuild harness moved from tests/convert.rs to tests/ui.rs so every case builds in one cargo run.
- 2026-10-12T13:23Z [claude (f24d940b-27ff-4906-b27c-e6a7b31fddaf)] Owner ruling in session f24d940b, after the deny ignore landed: "using unmaintained code is a code smell. seriously nothing we can use that doesn't have a security advisory warning against its use???" So bincode 1.3.3 (RUSTSEC-2025-0141, unmaintained) is out of the dev-dependencies and deny.toml ignores nothing. The fixed-framing serde proof now runs through postcard 1.1.3 (maintained, no_std, MIT OR Apache-2.0), whose encoding of a value tests/wire.rs asserts equals to_bytes, with the decoder's refusals surfacing as postcard's SerdeDeCustom. postcard enables serde's derive feature, which unification carries onto the crate's own serde edge, so unicode-ident's Unicode-3.0 licence is now on a normal path; deny.toml allows it with that reason. Upstream fact for the adapters: timely 0.30.0, which differential-dataflow 0.24.0 requires, lists bincode ^1.3 as a normal dependency (crates.io dependency lists, 2026-10-12), so the `differential` feature will bring the advisory back; noted on fl-v6mh9cv.
