# The float-abel wire format (v1)

A language-neutral specification of the canonical encodings of `Exact`, `Wide` and `Fixed`. An
implementation that follows this page reads and writes the same bytes as the crate, bit for bit:
`to_bytes` and `from_bytes` emit and accept nothing else, the `serde` impls carry the same bytes,
and `tests/wire.rs` asserts the vectors below against `to_bytes`.

Every multi-byte field is little-endian. An encoding is one version byte, then the type's body,
and the body is the type's `repr(C)` in-memory layout on a little-endian machine, so an engine that
frames its own extents may cast a mapping of `Wide` or `Fixed` bodies in place. `Exact`'s bytes from
outside still go through the decoder, because not every 32-byte pattern is a value, and its heap
form's body is the block the word points at, so it has no fixed length.

## The version byte

| byte   | type                | body                                    |
|--------|---------------------|-----------------------------------------|
| `0x01` | `Exact`, compact    | 32 bytes                                |
| `0x02` | `Wide`              | 296 bytes                               |
| `0x03` | `Fixed<LSB, LIMBS>` | 8 · LIMBS bytes                         |
| `0x04` | `Exact`, heap       | 32 bytes of header, then 8 bytes a limb |

The byte names the type, its form and the revision of its encoding. A later revision of a type's
encoding takes a fresh value, so an old body is never misread as a new one, and a body of one type
is never read as another's even when the lengths agree (`Exact` and `Fixed256` are both 33 bytes).
`Fixed`'s grid and width are the type's, not the encoding's: both sides know `(LSB, LIMBS)`, every
`Fixed` shares the byte, and a body of the wrong width is a length error.

Decoders MUST reject any other first byte, and any length other than one plus the body's, before
reading the body; a heap `Exact`'s length follows from its header, as its section says. The format
freezes at 1.0; until then a revision may change a body, and takes a fresh version byte when it
does.

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

## Exact, heap form (33 + 8 · limbs bytes)

The form a value takes when it does not fit the compact one: a finite part of more than 127 bits,
or a special amount that is not an integer inside `i32` (a count past `i32`, or a dyadic amount
such as `2.5 · NaN`). The body is the block the word holds in memory: four header limbs, one per
run, then the runs' limbs. The runs are the finite part, then the NaN, +∞ and −∞ amounts, each a
normalised sign-magnitude dyadic `(−1)^sign · mag · 2^exp`.

| field  | size       | meaning                                                                                                   |
|--------|------------|-----------------------------------------------------------------------------------------------------------|
| header | 4 × u64    | one entry per run: bit 63 the sign, bits 32–62 the run's limb count, bits 0–31 its exponent as an `i32` |
| limbs  | Σ len × u64 | each run's magnitude, least significant limb first, the four runs back to back in header order         |

**Canonical form.** Every run is normalised: the zero is the empty run (count 0, sign 0, exponent
0); otherwise the first limb is odd, so the lowest set bit sits at the exponent, the last limb is
nonzero, so there is no leading zero limb, and the exponent lies in `−2³⁰ ..= 2³⁰`. Every count is
at most `EXACT_MAX_LIMBS` (537 at the default cap). And the value does not fit the compact form:
the finite part spans more than 127 bits, or some amount is not an integer inside
`−(2³¹ − 1) ..= 2³¹ − 1`, which for a normalised run means more than one limb, a negative exponent,
or the exponent plus the limb's bit length past 31. A value the compact form holds has the compact
encoding and no other.

**Decoders MUST reject**, in this order so that the first failing rule names the refusal: fewer
than 33 bytes (`Length`, expecting 33); a header count past `EXACT_MAX_LIMBS` (`Limbs`), checked on
the header alone before the length and before anything is allocated, because an untrusted size gets
a ceiling; a length other than `33 + 8 · Σ count` (`Length`); then for each run in order, a sign or
an exponent on an empty run (`Form`), an even first limb (`EvenMantissa`), a zero last limb
(`Form`) and an exponent outside the bound (`Exponent`); and last a value that fits the compact form
(`Form`). The crate validates the body over the bytes and allocates once, for the accepted block.
A refusal is a typed error, never a panic.

**Reading the value.** As the compact form reads, with the amounts in place of the counts: a
nonzero NaN amount, or both infinity amounts nonzero, reads NaN; one nonzero infinity amount reads
that infinity, flipped when the amount is negative; otherwise the finite part rounds to nearest,
ties to even.

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

Under the `serde` feature an `Exact` or a `Wide` serialises as one byte string, the bytes of
`to_bytes` in order, so a format frames it with its own length: bincode 1 a `u64` little-endian
(41 bytes for an `Exact`, 305 for a `Wide`), postcard a varint (34 and 299), and serde_json an array
of numbers. A `Fixed` serialises as a tuple of `BYTES` elements of `u8`, which is how a
`[u8; BYTES]` serialises, so a format that writes a tuple's elements back to back, postcard or
bincode 1's fixed-int encoding, emits its `to_bytes` byte for byte, and serde_json carries it as an
array of numbers. `tests/wire.rs` asserts the framing with postcard and the arrays with serde_json.
Deserialisation takes the byte string, or gathers the array's elements, and runs `from_bytes`, so
every refusal above applies, as the deserialiser's error. An `Exact` in its heap form serialises
the same way, as the byte string `encode` gives, its tag then its block, under the `alloc`
feature; `to_bytes` is the compact form's fixed array, `encode` either form's bytes, and
`from_bytes` reads both. Without `alloc` no value is in the heap form and `from_bytes` refuses the
tag as a `Version`.

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
| `Exact::from(2.5) * Exact::from(f64::NAN)` | heap form: no finite part, the NaN amount 5 · 2⁻¹ as one limb at exponent −1 | `04 00*8 ff ff ff ff 01 00 00 00 00*8 00*8 05 00*7` |
| `Exact::from(f64::INFINITY) * (i32::MAX as i64) + Exact::from(f64::INFINITY)` | heap form: a +∞ amount of 2³¹, past `i32`, as the limb 1 at exponent 31 | `04 00*8 00*8 1f 00 00 00 01 00 00 00 00*8 01 00*7` |
| 2^127 + 1 (`Exact::from(1.0)` added to 2¹²⁷) | heap form: the narrowest finite part past the width, two limbs at exponent 0 | `04 00 00 00 00 02 00 00 00 00*24 01 00*7 00*7 80` |

## Conformance

An implementation conforms iff it emits the bytes above for every value, reads them back to the
same value, and refuses every byte string this page says to refuse. The crate's checks are
`tests/wire.rs`.
