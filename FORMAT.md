# The float-abel wire format (v1)

A language-neutral specification of the canonical encodings of `Exact`, `Wide` and `Fixed`. An
implementation that follows this page reads and writes the same bytes as the crate, bit for bit:
`to_bytes` and `from_bytes` emit and accept nothing else, the `serde` impls carry the same bytes,
and `tests/wire.rs` asserts the vectors below against `to_bytes`.

Every multi-byte field is little-endian. An encoding is one version byte, then the type's body,
and the body is the type's `repr(C)` in-memory layout on a little-endian machine, so an engine that
frames its own extents may cast a mapping of `Wide` or `Fixed` bodies in place. `Exact`'s bytes from
outside still go through the decoder, because not every 32-byte pattern is a value.

## The version byte

| byte   | type                | body            |
|--------|---------------------|-----------------|
| `0x01` | `Exact`             | 32 bytes        |
| `0x02` | `Wide`              | 296 bytes       |
| `0x03` | `Fixed<LSB, LIMBS>` | 8 · LIMBS bytes |

The byte names the type and the revision of its encoding. A later revision of a type's encoding
takes a fresh value, so an old body is never misread as a new one, and a body of one type is never
read as another's even when the lengths agree (`Exact` and `Fixed256` are both 33 bytes). `Fixed`'s
grid and width are the type's, not the encoding's: both sides know `(LSB, LIMBS)`, every `Fixed`
shares the byte, and a body of the wrong width is a length error.

Decoders MUST reject any other first byte, and any length other than one plus the body's, before
reading the body. The format freezes at 1.0; until then a revision may change a body, and takes a
fresh version byte when it does.

## Exact (33 bytes)

| field     | size | meaning                                                                  |
|-----------|------|--------------------------------------------------------------------------|
| `lo`      | u64  | the low limb of the mantissa                                             |
| `hi`      | u64  | the high limb: `hi:lo` is a two's-complement 128-bit integer `m`         |
| `exp`     | i32  | the finite part is `m · 2^exp`                                           |
| `nan`     | i32  | the signed count of NaN inputs in the multiset                           |
| `pos_inf` | i32  | the signed count of +∞ inputs                                            |
| `neg_inf` | i32  | the signed count of −∞ inputs                                            |

**Canonical form.** `m` is odd, or `m = 0` and `exp = 0`. `exp` lies in `−2³⁰ ..= 2³⁰`
(`EXACT_MAX_EXP`). No count is `i32::MIN`. Every odd `m` lies inside the 127-bit width, so canonical
form has no width rule.

**Decoders MUST reject** a nonzero even `m` (`EvenMantissa`), a zero `m` with a nonzero `exp`
(`ZeroExponent`), an `exp` outside the bound (`Exponent`) and a count of `i32::MIN` (`Count`),
checked in that order so that the first failing rule names the refusal. Each value then has exactly
one encoding, and structural equality of decoded words is numeric equality. A refusal is a typed
error, never a panic.

**Reading the value.** A nonzero `nan`, or both infinity counts nonzero, reads NaN; one nonzero
infinity count reads that infinity, flipped when the count is negative; otherwise `m · 2^exp` rounds
to nearest, ties to even, with subnormals and overflow to ±∞, by bitrep's `round_at` kernel on `|m|`
at `exp`. The zero reads `+0.0`.

## Wide (297 bytes)

| field         | size     | meaning                                                                                                                   |
|---------------|----------|---------------------------------------------------------------------------------------------------------------------------|
| limbs         | 34 × u64 | a 2176-bit two's-complement integer, least significant limb first, in units of 2⁻¹⁰⁷⁴: bitrep's layout, so its conformance vectors apply to these 272 bytes |
| `n`, `p`, `q` | 3 × i64  | the signed counts of NaN, +∞ and −∞ inputs                                                                                |

Every body is a value: a two's-complement integer and three integers have one bit pattern per value,
so there is no canonical-form rule, and decoders check only the version byte and the length. The
value reads as `Exact`'s does, with the finite part `limbs · 2⁻¹⁰⁷⁴`.

## Fixed (1 + 8 · LIMBS bytes)

| field | size        | meaning                                                                               |
|-------|-------------|---------------------------------------------------------------------------------------|
| limbs | LIMBS × u64 | a 64 · LIMBS-bit two's-complement integer, least significant limb first, in units of 2^LSB |

Every body is a value. `Fixed256` is `Fixed<−96, 4>`: 33 bytes, in units of 2⁻⁹⁶. The value reads as
the finite part alone, rounded as above.

## serde

Under the `serde` feature a value serialises as a tuple of `BYTES` elements of `u8`, the bytes of
`to_bytes` in order, which is how a `[u8; BYTES]` serialises. A format therefore frames it as it
frames a fixed-length array: bincode 1.3's fixed-int encoding of a value is `to_bytes` byte for
byte, with no length prefix, and serde_json carries it as an array of numbers. Deserialisation
gathers the elements and runs `from_bytes`, so every refusal above applies, as the deserialiser's
error.

## Vectors

Bytes are two hex digits each, and `xx*N` is `N` bytes of `xx`.

| value | derivation | bytes |
|-------|------------|-------|
| `Exact::from(1.5)` | 3 · 2⁻¹ | `01 03 00*7 00*8 ff ff ff ff 00*12` |
| `Exact::from(-0.1)` | −0xCCCCCCCCCCCCD · 2⁻⁵⁵, two's complement across both limbs | `01 33 33 33 33 33 33 f3 ff ff*8 c9 ff ff ff 00*12` |
| `Exact::ZERO` | the zero form | `01 00*32` |
| `Exact::from(f64::NAN) * 2 + Exact::from(f64::INFINITY) - Exact::from(f64::NEG_INFINITY)` | counts (2, 1, −1) | `01 00*20 02 00 00 00 01 00 00 00 ff ff ff ff` |
| `Exact::from(2.0)` squared thirty times | 1 · 2^(2³⁰), the exponent bound | `01 01 00*7 00*8 00 00 00 40 00*12` |
| `Wide::from(1.0)` | 2¹⁰⁷⁴ units: bit 50 of limb 16 | `02 00*128 00 00 00 00 00 00 04 00 00*136 00*24` |
| `Wide::from(-1.0)` | its two's complement: the sign fills every limb above | `02 00*128 00 00 00 00 00 00 fc ff ff*136 00*24` |
| `Wide::from(f64::NAN)` | one NaN count | `02 00*272 01 00*7 00*16` |
| `Fixed256::try_from(1.0)` | 2⁹⁶ units: bit 32 of limb 1 | `03 00*8 00 00 00 00 01 00 00 00 00*16` |
| `Fixed256::try_from(-1.0)` | its two's complement | `03 00*8 00 00 00 00 ff ff ff ff ff*16` |
| `Fixed256::try_from(2f64.powi(-44))` | 2⁵² units, the lowest double on the grid's unit column | `03 00 00 00 00 00 00 10 00 00*24` |

## Conformance

An implementation conforms iff it emits the bytes above for every value, reads them back to the
same value, and refuses every byte string this page says to refuse. The crate's checks are
`tests/wire.rs`.
