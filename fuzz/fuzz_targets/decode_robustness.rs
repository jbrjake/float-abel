//! Arbitrary bytes into every decoder, DESIGN.md § Hardening: no panic; a refusal names the first
//! rule the bytes break, re-derived here from FORMAT.md alone; an accepted word re-encodes to the
//! same bytes and decodes again to itself; and the serde path agrees with `from_bytes` byte for
//! byte. The input is tried as it comes, then forced to each type's length and version byte so the
//! body rules are reached on every run.

#![no_main]

use core::fmt::Debug;

use float_abel::{Exact, Fixed, Fixed256, Wide, WireError, EXACT_MAX_EXP};
use libfuzzer_sys::fuzz_target;

/// The narrowest `Fixed`, 17 bytes on the wire.
type Narrow = Fixed<-20, 2>;

/// FORMAT.md's verdict on an `Exact` body: the zero form, an odd mantissa, the exponent bound and
/// the counts, in the order the checks run.
fn exact_verdict(body: &[u8]) -> Result<(), WireError> {
    let lo = u64::from_le_bytes(body[0..8].try_into().unwrap());
    let hi = u64::from_le_bytes(body[8..16].try_into().unwrap());
    let exp = i32::from_le_bytes(body[16..20].try_into().unwrap());
    let counts = [20, 24, 28].map(|at| i32::from_le_bytes(body[at..at + 4].try_into().unwrap()));
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

/// One decoder on the input as it is, then on the input forced to its length and version byte.
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

fuzz_target!(|data: &[u8]| {
    let buf = check(
        data,
        Exact::BYTES,
        Exact::WIRE_VERSION,
        Exact::from_bytes,
        |v| v.to_bytes().to_vec(),
        |b| postcard::from_bytes::<Exact>(b).ok(),
    );
    assert_eq!(
        Exact::from_bytes(&buf).map(|_| ()),
        exact_verdict(&buf[1..]),
        "the decoder's verdict is FORMAT.md's on {buf:02x?}"
    );
    check(
        data,
        Wide::BYTES,
        Wide::WIRE_VERSION,
        Wide::from_bytes,
        |v| v.to_bytes().to_vec(),
        |b| postcard::from_bytes::<Wide>(b).ok(),
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
