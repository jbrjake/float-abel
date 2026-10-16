//! Arbitrary bytes into every decoder, DESIGN.md § Hardening: no panic; a refusal names the first
//! rule the bytes break, re-derived here from FORMAT.md alone; an accepted word re-encodes to the
//! same bytes and decodes again to itself; and the serde path agrees with `from_bytes` byte for
//! byte. The input is tried as it comes, then forced to each type's length and version byte so the
//! body rules are reached on every run, and forced to `Exact`'s heap tag so the heap body's rules
//! are reached too.

#![no_main]
// The verdicts move bits between integer widths as FORMAT.md lays them out.
#![allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]

use core::fmt::Debug;

use float_abel::{Exact, Fixed, Fixed256, Wide, WireError, EXACT_MAX_EXP, EXACT_MAX_LIMBS};
use libfuzzer_sys::fuzz_target;

/// The narrowest `Fixed`, 17 bytes on the wire.
type Narrow = Fixed<-20, 2>;

/// Limbs of header in a heap body, one per run.
const HEADER: usize = 4;
/// The shortest heap encoding: the tag and the header, every run empty.
const LEAST: usize = 1 + 8 * HEADER;

fn u64_at(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())
}

fn i32_at(bytes: &[u8], at: usize) -> i32 {
    i32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}

/// FORMAT.md's verdict on a compact `Exact` body: the zero form, an odd mantissa, the exponent
/// bound and the counts, in the order the checks run.
fn exact_verdict(body: &[u8]) -> Result<(), WireError> {
    let lo = u64_at(body, 0);
    let hi = u64_at(body, 8);
    let exp = i32_at(body, 16);
    let counts = [20, 24, 28].map(|at| i32_at(body, at));
    if lo | hi == 0 {
        if exp != 0 {
            return Err(WireError::ZeroExponent);
        }
    } else if lo & 1 == 0 {
        return Err(WireError::EvenMantissa);
    }
    if !(-EXACT_MAX_EXP..=EXACT_MAX_EXP).contains(&exp) {
        return Err(WireError::Exponent);
    }
    if counts.contains(&i32::MIN) {
        return Err(WireError::Count);
    }
    Ok(())
}

/// FORMAT.md's verdict on a heap body, the bytes behind the tag: the header's four counts under
/// the cap, the length they imply, each run in order normalised (empty with no sign or exponent;
/// else an odd first limb, a nonzero last limb, the exponent bound), and the whole not fitting the
/// compact form, a finite part of at most 127 bits with every amount an integer inside `i32`.
fn heap_verdict(body: &[u8]) -> Result<(), WireError> {
    let found = 1 + body.len();
    if body.len() < 8 * HEADER {
        return Err(WireError::Length {
            expected: LEAST,
            found,
        });
    }
    let header: Vec<(bool, i32, usize)> = (0..HEADER)
        .map(|i| {
            let e = u64_at(body, 8 * i);
            (
                e >> 63 != 0,
                e as u32 as i32,
                ((e >> 32) & 0x7FFF_FFFF) as usize,
            )
        })
        .collect();
    if header.iter().any(|h| h.2 > EXACT_MAX_LIMBS) {
        return Err(WireError::Limbs);
    }
    let total: usize = header.iter().map(|h| h.2).sum();
    let expected = LEAST + 8 * total;
    if found != expected {
        return Err(WireError::Length { expected, found });
    }
    let limb = |i: usize| u64_at(body, 8 * (HEADER + i));
    let mut at = 0;
    let mut fits = true;
    for (k, &(neg, exp, len)) in header.iter().enumerate() {
        if len == 0 {
            if neg || exp != 0 {
                return Err(WireError::Form);
            }
            continue;
        }
        let (first, last) = (limb(at), limb(at + len - 1));
        at += len;
        if first & 1 == 0 {
            return Err(WireError::EvenMantissa);
        }
        if last == 0 {
            return Err(WireError::Form);
        }
        if !(-EXACT_MAX_EXP..=EXACT_MAX_EXP).contains(&exp) {
            return Err(WireError::Exponent);
        }
        let bits = 64 * len as i64 - i64::from(last.leading_zeros());
        fits &= if k == 0 {
            bits <= 127
        } else {
            len == 1 && exp >= 0 && i64::from(exp) + bits <= 31
        };
    }
    if fits {
        return Err(WireError::Form);
    }
    Ok(())
}

/// postcard's framing of a byte string, a varint length then the bytes, so the serde path of a
/// type that serialises as one sees what `from_bytes` sees.
fn framed(body: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(body.len() + 4);
    let mut n = body.len();
    while n >= 0x80 {
        out.push((n & 0x7f) as u8 | 0x80);
        n >>= 7;
    }
    out.push(n as u8);
    out.extend_from_slice(body);
    out
}

/// One fixed-length decoder on the input as it is, then on the input forced to its length and
/// version byte.
fn check<T: Clone + PartialEq + Debug>(
    data: &[u8],
    bytes: usize,
    version: u8,
    decode: impl Fn(&[u8]) -> Result<T, WireError>,
    encode: impl Fn(&T) -> Vec<u8>,
    serde: impl Fn(&[u8]) -> Option<T>,
) -> Vec<u8> {
    match decode(data) {
        Ok(v) => {
            assert_eq!(data.len(), bytes);
            assert_eq!(data[0], version);
            assert_eq!(encode(&v), data, "an accepted word re-encodes to its bytes");
        }
        Err(WireError::Length { expected, found }) => {
            assert_eq!((expected, found), (bytes, data.len()));
            assert_ne!(data.len(), bytes, "the length refusal names a wrong length");
        }
        Err(WireError::Version(v)) => {
            assert_eq!(
                data.len(),
                bytes,
                "the version is read only at the right length"
            );
            assert_eq!(v, data[0]);
            assert_ne!(v, version);
        }
        Err(_) => {
            assert_eq!(
                data.len(),
                bytes,
                "a body is refused only at the right length"
            );
            assert_eq!(
                data[0], version,
                "a body is refused only behind its version byte"
            );
        }
    }
    let mut buf = vec![0u8; bytes];
    let n = data.len().min(bytes);
    buf[..n].copy_from_slice(&data[..n]);
    buf[0] = version;
    let direct = decode(&buf);
    if let Ok(v) = &direct {
        assert_eq!(encode(v), buf);
        assert_eq!(
            decode(&encode(v)),
            Ok(v.clone()),
            "a decoded word decodes again"
        );
    }
    assert_eq!(
        serde(&buf),
        direct.ok(),
        "postcard and from_bytes disagree on {buf:02x?}"
    );
    buf
}

/// `Exact`'s heap decoder on the input with its first byte forced to the tag, which is the input
/// as it is when it already carries one: the verdict is FORMAT.md's, an accepted body re-encodes
/// to its bytes and is a heap value the operations take, and the serde path agrees.
fn check_heap(data: &[u8]) {
    let mut buf = data.to_vec();
    if buf.is_empty() {
        buf.push(0);
    }
    buf[0] = Exact::HEAP_TAG;
    let direct = Exact::from_bytes(&buf);
    assert_eq!(
        direct.as_ref().map(|_| ()).map_err(|e| *e),
        heap_verdict(&buf[1..]),
        "the decoder's verdict is FORMAT.md's on {buf:02x?}"
    );
    if let Ok(v) = &direct {
        assert!(!v.is_compact(), "a heap body decodes to the heap form");
        assert_eq!(
            v.encode(),
            buf,
            "an accepted heap body re-encodes to its bytes"
        );
        assert_eq!(Exact::from_bytes(&v.encode()), Ok(v.clone()));
        assert_eq!(v.checked_add(&Exact::ZERO), Ok(v.clone()));
        assert_eq!(v.checked_sub(v), Ok(Exact::ZERO));
    }
    assert_eq!(
        postcard::from_bytes::<Exact>(&framed(&buf)).ok(),
        direct.ok(),
        "postcard and from_bytes disagree on {buf:02x?}"
    );
}

fuzz_target!(|data: &[u8]| {
    // The compact rules on the input as it is, unless it carries the heap tag, which the heap
    // check below takes as it is; the compact pass then sees it behind a byte that is neither.
    let compact_data: Vec<u8> = match data.first() {
        Some(&Exact::HEAP_TAG) => {
            let mut d = data.to_vec();
            d[0] = 0;
            d
        }
        _ => data.to_vec(),
    };
    let buf = check(
        &compact_data,
        Exact::BYTES,
        Exact::WIRE_VERSION,
        Exact::from_bytes,
        |v| v.to_bytes().to_vec(),
        |b| postcard::from_bytes::<Exact>(&framed(b)).ok(),
    );
    assert_eq!(
        Exact::from_bytes(&buf).map(|_| ()),
        exact_verdict(&buf[1..]),
        "the decoder's verdict is FORMAT.md's on {buf:02x?}"
    );
    check_heap(data);
    check(
        data,
        Wide::BYTES,
        Wide::WIRE_VERSION,
        Wide::from_bytes,
        |v| v.to_bytes().to_vec(),
        |b| postcard::from_bytes::<Wide>(&framed(b)).ok(),
    );
    check(
        data,
        Fixed256::BYTES,
        Fixed256::WIRE_VERSION,
        Fixed256::from_bytes,
        |v| {
            let b: [u8; Fixed256::BYTES] = v.to_bytes();
            b.to_vec()
        },
        |b| postcard::from_bytes::<Fixed256>(b).ok(),
    );
    check(
        data,
        Narrow::BYTES,
        Narrow::WIRE_VERSION,
        Narrow::from_bytes,
        |v| {
            let b: [u8; Narrow::BYTES] = v.to_bytes();
            b.to_vec()
        },
        |b| postcard::from_bytes::<Narrow>(b).ok(),
    );
});
