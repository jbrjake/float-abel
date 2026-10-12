//! The canonical wire format, FORMAT.md: `to_bytes` is the version byte then the type's `repr(C)`
//! body little-endian, `from_bytes` is its inverse and refuses everything else by name, and the
//! `serde` feature carries exactly those bytes as a fixed-length tuple, so bincode 1.3's fixed-int
//! encoding of a value is `to_bytes` byte for byte. Every test is named `wire_*`.

use std::collections::HashMap;

use bincode::Options;
use float_abel::{Exact, Fixed, Fixed256, Wide, WireError, EXACT_MAX_EXP};

use crate::support::Rng;

/// A two-limb grid: the narrowest `Fixed`, 17 bytes on the wire.
type Narrow = Fixed<-20, 2>;

/// Where limb 16 of a `Wide` starts in its encoding: behind the version byte and 16 limbs.
const LIMB_16: usize = 1 + 16 * 8;

/// An `Exact` encoding built by hand: the version byte, the mantissa's two limbs, the exponent and
/// the three counts, each little-endian. Nothing here is validated, so a test can build a word
/// that no operation produces.
fn exact_bytes(lo: u64, hi: u64, exp: i32, counts: [i32; 3]) -> [u8; Exact::BYTES] {
    let mut out = [0u8; Exact::BYTES];
    out[0] = Exact::WIRE_VERSION;
    out[1..9].copy_from_slice(&lo.to_le_bytes());
    out[9..17].copy_from_slice(&hi.to_le_bytes());
    out[17..21].copy_from_slice(&exp.to_le_bytes());
    for (i, c) in counts.iter().enumerate() {
        out[21 + 4 * i..25 + 4 * i].copy_from_slice(&c.to_le_bytes());
    }
    out
}

/// The body of a `Wide` as tests/layout.rs reads it off the accessors: the limbs least
/// significant first, then the `[n, p, q]` counts, each little-endian.
fn wide_body(w: &Wide) -> Vec<u8> {
    let mut out = Vec::with_capacity(Wide::BYTES - 1);
    for limb in w.limbs() {
        out.extend_from_slice(&limb.to_le_bytes());
    }
    let (n, p, q) = w.specials();
    for count in [n, p, q] {
        out.extend_from_slice(&count.to_le_bytes());
    }
    out
}

fn random_wide(rng: &mut Rng) -> Wide {
    let mut limbs = [0u64; Wide::LIMBS];
    for l in &mut limbs {
        *l = rng.next();
    }
    let specials = [rng.next() as i64, rng.next() as i64, rng.next() as i64];
    Wide::from_parts(limbs, specials)
}

fn random_fixed(rng: &mut Rng) -> Fixed256 {
    let mut limbs = [0u64; 4];
    for l in &mut limbs {
        *l = rng.next();
    }
    Fixed256::from_limbs(limbs)
}

/// The refusal of a buffer that is not the type's length.
const fn length(expected: usize, found: usize) -> WireError {
    WireError::Length { expected, found }
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes.iter().fold(String::new(), |mut s, b| {
        let _ = write!(s, "{b:02x}");
        s
    })
}

/// FORMAT.md's vector notation: bytes as two hex digits each, space-separated, with a run of one
/// byte written `xx*N`.
fn expand(notation: &str) -> Vec<u8> {
    let mut out = Vec::new();
    for token in notation.split_whitespace() {
        let (byte, run) = match token.split_once('*') {
            Some((b, n)) => (b, n.parse::<usize>().unwrap()),
            None => (token, 1),
        };
        let byte = u8::from_str_radix(byte, 16).unwrap();
        out.extend(std::iter::repeat(byte).take(run));
    }
    out
}

#[test]
fn wire_exact_is_the_version_byte_then_its_layout_and_round_trips() {
    // 1.5 = 3 · 2^-1: the canonical mantissa is odd and the exponent carries the scale.
    let x = Exact::from(1.5);
    let bytes = x.to_bytes();
    assert_eq!(bytes.len(), 33);
    assert_eq!(bytes, exact_bytes(3, 0, -1, [0; 3]));
    assert_eq!(Exact::from_bytes(&bytes), Ok(x));
    // -0.1 = -0xCCCCCCCCCCCCD · 2^-55: a negative mantissa is two's complement across both limbs.
    let neg = Exact::from(-0.1);
    assert_eq!(
        neg.to_bytes(),
        exact_bytes(0xFFF3_3333_3333_3333, u64::MAX, -55, [0; 3])
    );
    // The counts sit behind the exponent, signed.
    let mut s = Exact::from(f64::NAN) + Exact::from(f64::NAN);
    s += Exact::from(f64::INFINITY);
    s -= Exact::from(f64::NEG_INFINITY);
    assert_eq!(s.to_bytes(), exact_bytes(0, 0, 0, [2, 1, -1]));
    assert_eq!(Exact::ZERO.to_bytes(), exact_bytes(0, 0, 0, [0; 3]));
    // Both exponent bounds, reached by repeated squaring, are values and round-trip.
    let top = (0..30).fold(Exact::from(2.0), |x, _| x * x);
    let bottom = (0..30).fold(Exact::from(0.5), |x, _| x * x);
    assert_eq!(top.to_bytes(), exact_bytes(1, 0, EXACT_MAX_EXP, [0; 3]));
    assert_eq!(bottom.to_bytes(), exact_bytes(1, 0, -EXACT_MAX_EXP, [0; 3]));
    let values = [
        x,
        neg,
        s,
        Exact::ZERO,
        top,
        bottom,
        Exact::from(5e-324),
        Exact::from(f64::MAX),
        Exact::product(f64::MAX, -f64::MAX),
        Exact::product(5e-324, 5e-324),
        Exact::from(-1.0),
    ];
    assert_eq!(values.len(), 11);
    for v in values {
        let back = Exact::from_bytes(&v.to_bytes()).unwrap();
        assert_eq!(back, v);
        assert_eq!(back.to_f64().to_bits(), v.to_f64().to_bits());
        assert_eq!(back.specials(), v.specials());
    }
}

#[test]
fn wire_exact_round_trips_random_multisets_and_distinct_values_have_distinct_bytes() {
    let mut rng = Rng(0x77);
    let mut seen: HashMap<[u8; Exact::BYTES], Exact> = HashMap::new();
    for _ in 0..2000 {
        let mut s = Exact::ZERO;
        for _ in 0..8 {
            s.deposit(rng.on_grid(-10, 10));
        }
        match rng.next() % 8 {
            0 => s += Exact::from(f64::NAN),
            1 => s -= Exact::from(f64::INFINITY),
            2 => s += Exact::from(f64::NEG_INFINITY) * 3i64,
            _ => {}
        }
        let bytes = s.to_bytes();
        assert_eq!(Exact::from_bytes(&bytes), Ok(s));
        if let Some(earlier) = seen.insert(bytes, s) {
            assert_eq!(earlier, s, "two values share the bytes {}", hex(&bytes));
        }
    }
    // Eight random same-scale doubles rarely sum to the same value twice.
    assert!(seen.len() > 1900, "{} distinct encodings", seen.len());
}

#[test]
fn wire_wide_is_the_version_byte_then_the_limbs_then_the_counters_and_round_trips() {
    let mut w = Wide::ZERO;
    for x in [
        1.5,
        -2.0e300,
        5e-324,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::INFINITY,
    ] {
        w.deposit(x);
    }
    let bytes = w.to_bytes();
    assert_eq!(bytes.len(), 297);
    assert_eq!(bytes[0], Wide::WIRE_VERSION);
    assert_eq!(&bytes[1..], &wide_body(&w)[..]);
    assert_eq!(Wide::from_bytes(&bytes), Ok(w));
    // 1.0 is 2^1074 units: bit 50 of limb 16, and nothing else.
    let one = Wide::from(1.0).to_bytes();
    assert_eq!(one[LIMB_16..LIMB_16 + 8], (1u64 << 50).to_le_bytes());
    assert!(one[1..LIMB_16].iter().all(|&b| b == 0));
    assert!(one[LIMB_16 + 8..].iter().all(|&b| b == 0));
    // Every bit pattern is a value, and distinct patterns have distinct bytes.
    let mut rng = Rng(0x5eed);
    let mut seen: HashMap<Vec<u8>, Wide> = HashMap::new();
    for _ in 0..256 {
        let w = random_wide(&mut rng);
        let bytes = w.to_bytes();
        assert_eq!(Wide::from_bytes(&bytes), Ok(w));
        assert!(seen.insert(bytes.to_vec(), w).is_none());
    }
    assert_eq!(seen.len(), 256);
}

#[test]
fn wire_fixed_is_the_version_byte_then_the_limbs_and_round_trips_at_every_width() {
    let one = Fixed256::try_from(1.0).unwrap();
    let bytes: [u8; Fixed256::BYTES] = one.to_bytes();
    assert_eq!(bytes.len(), 33);
    // 1.0 is 2^96 units: bit 32 of limb 1.
    let mut expected = [0u8; 33];
    expected[0] = Fixed256::WIRE_VERSION;
    expected[9..17].copy_from_slice(&(1u64 << 32).to_le_bytes());
    assert_eq!(bytes, expected);
    assert_eq!(Fixed256::from_bytes(&bytes), Ok(one));
    // -1.0 is the two's complement across the limbs: the sign fills every limb above.
    let minus: [u8; Fixed256::BYTES] = (-one).to_bytes();
    let mut expected = [0xffu8; 33];
    expected[0] = Fixed256::WIRE_VERSION;
    expected[1..13].fill(0);
    assert_eq!(minus, expected);
    assert_eq!(Fixed256::from_bytes(&minus), Ok(-one));
    // A two-limb grid is 17 bytes, and its own type reads them.
    let mut narrow = Narrow::ZERO;
    narrow.try_deposit(-3.25).unwrap();
    let bytes: [u8; Narrow::BYTES] = narrow.to_bytes();
    assert_eq!(bytes.len(), 17);
    assert_eq!(bytes[0], Narrow::WIRE_VERSION);
    assert_eq!(Narrow::from_bytes(&bytes), Ok(narrow));
    assert_eq!(Fixed256::from_bytes(&bytes), Err(length(33, 17)));
    // Every bit pattern is a value, and distinct patterns have distinct bytes.
    let mut rng = Rng(0xf1);
    let mut seen: HashMap<[u8; Fixed256::BYTES], Fixed256> = HashMap::new();
    for _ in 0..256 {
        let f = random_fixed(&mut rng);
        let bytes: [u8; Fixed256::BYTES] = f.to_bytes();
        assert_eq!(Fixed256::from_bytes(&bytes), Ok(f));
        assert!(seen.insert(bytes, f).is_none());
    }
    assert_eq!(seen.len(), 256);
}

#[test]
fn wire_exact_refuses_each_non_canonical_form_by_name_and_never_panics() {
    use WireError::{Count, EvenMantissa, Exponent, Version, ZeroExponent};
    let refused = [
        (exact_bytes(2, 0, 0, [0; 3]), EvenMantissa),
        (exact_bytes(6, 0, -3, [0; 3]), EvenMantissa),
        // i128::MIN: the one 128-bit magnitude, which is even.
        (exact_bytes(0, 1 << 63, 0, [0; 3]), EvenMantissa),
        (exact_bytes(0, 0, 5, [0; 3]), ZeroExponent),
        (exact_bytes(0, 0, -1, [0; 3]), ZeroExponent),
        (exact_bytes(1, 0, EXACT_MAX_EXP + 1, [0; 3]), Exponent),
        (exact_bytes(1, 0, -EXACT_MAX_EXP - 1, [0; 3]), Exponent),
        (exact_bytes(1, 0, i32::MIN, [0; 3]), Exponent),
        (exact_bytes(1, 0, i32::MAX, [0; 3]), Exponent),
        (exact_bytes(1, 0, 0, [i32::MIN, 0, 0]), Count),
        (exact_bytes(1, 0, 0, [0, i32::MIN, 0]), Count),
        (exact_bytes(1, 0, 0, [0, 0, i32::MIN]), Count),
        // The checks run mantissa, exponent, counts: the first failing one names the error.
        (exact_bytes(2, 0, i32::MAX, [i32::MIN; 3]), EvenMantissa),
        (exact_bytes(1, 0, i32::MAX, [i32::MIN; 3]), Exponent),
    ];
    assert_eq!(refused.len(), 14);
    for (bytes, err) in refused {
        assert_eq!(Exact::from_bytes(&bytes), Err(err), "{}", hex(&bytes));
    }
    // Each bound itself is a value, and so is every odd mantissa.
    let accepted = [
        exact_bytes(1, 0, EXACT_MAX_EXP, [0; 3]),
        exact_bytes(1, 0, -EXACT_MAX_EXP, [i32::MIN + 1, i32::MAX, -1]),
        exact_bytes(u64::MAX, u64::MAX, 0, [0; 3]),
        // i128::MIN + 1 and i128::MAX: the widest magnitudes, both odd.
        exact_bytes(1, 1 << 63, 0, [0; 3]),
        exact_bytes(u64::MAX, i64::MAX as u64, 0, [0; 3]),
    ];
    assert_eq!(accepted.len(), 5);
    for bytes in accepted {
        let v = Exact::from_bytes(&bytes).unwrap_or_else(|e| panic!("{e} on {}", hex(&bytes)));
        assert_eq!(v.to_bytes(), bytes);
        // A decoded word is a value the operations accept as it is.
        assert_eq!(v.checked_add(&Exact::ZERO), Ok(v));
        assert_eq!(v.checked_mul_i64(1), Ok(v));
    }
    assert_eq!(
        Exact::from_bytes(&exact_bytes(u64::MAX, u64::MAX, 0, [0; 3])),
        Ok(Exact::from(-1.0))
    );
    // Any other first byte, including another type's, is refused before the body is read.
    for v in [0x00, Wide::WIRE_VERSION, Fixed256::WIRE_VERSION, 0x11, 0xff] {
        let mut b = exact_bytes(3, 0, -1, [0; 3]);
        b[0] = v;
        assert_eq!(Exact::from_bytes(&b), Err(Version(v)));
    }
    // Any other length is refused before the first byte is read.
    let good = Exact::from(1.5).to_bytes();
    for n in [0, 1, 32] {
        assert_eq!(Exact::from_bytes(&good[..n]), Err(length(33, n)));
    }
    let mut long = good.to_vec();
    long.push(0);
    assert_eq!(Exact::from_bytes(&long), Err(length(33, 34)));
    // A Fixed256 is also 33 bytes; its version byte keeps its limbs from reading as a dyadic.
    let fixed: [u8; Fixed256::BYTES] = Fixed256::try_from(1.0).unwrap().to_bytes();
    assert_eq!(
        Exact::from_bytes(&fixed),
        Err(Version(Fixed256::WIRE_VERSION))
    );
    assert_eq!(
        Fixed256::from_bytes(&good),
        Err(Version(Exact::WIRE_VERSION))
    );
    // Each refusal says why.
    for (err, word) in [
        (EvenMantissa, "even"),
        (ZeroExponent, "zero"),
        (Exponent, "exponent"),
        (Count, "count"),
        (Version(7), "7"),
        (length(33, 2), "33"),
    ] {
        let text = err.to_string();
        assert!(text.contains(word), "{text:?} lacks {word:?}");
    }
    let boxed: Box<dyn std::error::Error> = Box::new(EvenMantissa);
    assert!(boxed.to_string().contains("even"));
}

#[test]
fn wire_wide_and_fixed_refuse_only_the_version_byte_and_the_length() {
    let mut rng = Rng(9);
    for _ in 0..64 {
        let w = random_wide(&mut rng);
        let bytes = w.to_bytes();
        let mut bad = bytes;
        bad[0] ^= 0x80;
        assert_eq!(Wide::from_bytes(&bad), Err(WireError::Version(bad[0])));
        assert_eq!(
            Wide::from_bytes(&bytes[..Wide::BYTES - 1]),
            Err(length(297, 296))
        );
        let long = [&bytes[..], &[0]].concat();
        assert_eq!(Wide::from_bytes(&long), Err(length(297, 298)));
        assert_eq!(Wide::from_bytes(&bytes), Ok(w));

        let f = random_fixed(&mut rng);
        let bytes: [u8; Fixed256::BYTES] = f.to_bytes();
        let mut bad = bytes;
        bad[0] = 0;
        assert_eq!(Fixed256::from_bytes(&bad), Err(WireError::Version(0)));
        assert_eq!(Fixed256::from_bytes(&bytes[1..]), Err(length(33, 32)));
        assert_eq!(Fixed256::from_bytes(&bytes), Ok(f));
    }
    assert_eq!(Wide::from_bytes(&[]), Err(length(297, 0)));
}

#[test]
fn wire_serde_bincode_fixint_is_to_bytes_byte_for_byte_and_round_trips() {
    let fixint = bincode::options().with_fixint_encoding();
    let x = Exact::from(1.5) + Exact::from(f64::NAN) - Exact::from(-0.1);
    let encoded = fixint.serialize(&x).unwrap();
    assert_eq!(encoded, x.to_bytes().to_vec());
    assert_eq!(fixint.deserialize::<Exact>(&encoded).unwrap(), x);
    // The legacy entry points, which the differential adapter rides, are fixed-int too.
    assert_eq!(bincode::serialize(&x).unwrap(), encoded);
    assert_eq!(bincode::deserialize::<Exact>(&encoded).unwrap(), x);

    let mut w = Wide::from(1e300);
    w.deposit(-5e-324);
    w.deposit(f64::NEG_INFINITY);
    let encoded = fixint.serialize(&w).unwrap();
    assert_eq!(encoded.len(), 297);
    assert_eq!(encoded, w.to_bytes().to_vec());
    assert_eq!(fixint.deserialize::<Wide>(&encoded).unwrap(), w);

    let f = Fixed256::try_from(-7.5).unwrap();
    let encoded = fixint.serialize(&f).unwrap();
    let direct: [u8; Fixed256::BYTES] = f.to_bytes();
    assert_eq!(encoded, direct.to_vec());
    assert_eq!(fixint.deserialize::<Fixed256>(&encoded).unwrap(), f);
    let mut narrow = Narrow::ZERO;
    narrow.try_deposit(2.0).unwrap();
    let encoded = fixint.serialize(&narrow).unwrap();
    let direct: [u8; Narrow::BYTES] = narrow.to_bytes();
    assert_eq!(encoded.len(), 17);
    assert_eq!(encoded, direct.to_vec());
    assert_eq!(fixint.deserialize::<Narrow>(&encoded).unwrap(), narrow);

    // A non-canonical word is refused through serde with the decoder's reason, never read.
    let even = exact_bytes(2, 0, 0, [0; 3]);
    let err = fixint.deserialize::<Exact>(&even).unwrap_err();
    assert!(err.to_string().contains("even"), "{err}");
    let zero_exp = exact_bytes(0, 0, 9, [0; 3]);
    assert!(fixint.deserialize::<Exact>(&zero_exp).is_err());
    // A short buffer, and another type's encoding of the same length, are refused too.
    assert!(fixint.deserialize::<Exact>(&even[..20]).is_err());
    let fixed = bincode::serialize(&Fixed256::try_from(1.0).unwrap()).unwrap();
    assert_eq!(fixed.len(), 33);
    let err = fixint.deserialize::<Exact>(&fixed).unwrap_err();
    assert!(err.to_string().contains("version"), "{err}");
    let mut wrong = fixint.serialize(&narrow).unwrap();
    wrong[0] = 0x7f;
    let err = fixint.deserialize::<Narrow>(&wrong).unwrap_err();
    assert!(err.to_string().contains("version"), "{err}");
    assert!(fixint.deserialize::<Narrow>(&wrong[..16]).is_err());
}

#[test]
fn wire_serde_json_carries_the_bytes_as_numbers_and_refuses_a_bad_or_short_array() {
    let x = Exact::from(1.5);
    let json = serde_json::to_string(&x).unwrap();
    let numbers: Vec<String> = x.to_bytes().iter().map(u8::to_string).collect();
    assert_eq!(json, format!("[{}]", numbers.join(",")));
    assert_eq!(serde_json::from_str::<Exact>(&json).unwrap(), x);

    let w = Wide::from(-2.5);
    let json = serde_json::to_string(&w).unwrap();
    assert_eq!(serde_json::from_str::<Wide>(&json).unwrap(), w);
    let f = Fixed256::try_from(0.75).unwrap();
    let json = serde_json::to_string(&f).unwrap();
    let direct: [u8; Fixed256::BYTES] = f.to_bytes();
    assert_eq!(json, serde_json::to_string(&direct[..]).unwrap());
    assert_eq!(serde_json::from_str::<Fixed256>(&json).unwrap(), f);

    assert!(serde_json::from_str::<Exact>("[1,2]").is_err());
    assert!(serde_json::from_str::<Fixed256>("[3,0]").is_err());
    let mut long = x.to_bytes().to_vec();
    long.push(0);
    assert!(serde_json::from_str::<Exact>(&format!("{long:?}")).is_err());
    let even = exact_bytes(2, 0, 0, [0; 3]);
    let err = serde_json::from_str::<Exact>(&format!("{even:?}")).unwrap_err();
    assert!(err.to_string().contains("even"), "{err}");
}

/// FORMAT.md's vectors are `to_bytes` byte for byte. Each row is found by its expression and its
/// notation expanded, so the document cannot drift from the code.
#[test]
fn wire_vectors_in_format_md_are_to_bytes_byte_for_byte() {
    let format = include_str!("../FORMAT.md");
    let two_nan_one_pos_minus_one_neg = {
        let mut s = Exact::from(f64::NAN) * 2i64;
        s += Exact::from(f64::INFINITY);
        s -= Exact::from(f64::NEG_INFINITY);
        s
    };
    let vectors: Vec<(&str, Vec<u8>)> = vec![
        ("`Exact::from(1.5)`", Exact::from(1.5).to_bytes().to_vec()),
        ("`Exact::from(-0.1)`", Exact::from(-0.1).to_bytes().to_vec()),
        ("`Exact::ZERO`", Exact::ZERO.to_bytes().to_vec()),
        (
            "`Exact::from(f64::NAN) * 2 + Exact::from(f64::INFINITY) - Exact::from(f64::NEG_INFINITY)`",
            two_nan_one_pos_minus_one_neg.to_bytes().to_vec(),
        ),
        (
            "`Exact::from(2.0)` squared thirty times",
            (0..30)
                .fold(Exact::from(2.0), |x, _| x * x)
                .to_bytes()
                .to_vec(),
        ),
        ("`Wide::from(1.0)`", Wide::from(1.0).to_bytes().to_vec()),
        ("`Wide::from(-1.0)`", Wide::from(-1.0).to_bytes().to_vec()),
        (
            "`Wide::from(f64::NAN)`",
            Wide::from(f64::NAN).to_bytes().to_vec(),
        ),
        ("`Fixed256::try_from(1.0)`", {
            let b: [u8; 33] = Fixed256::try_from(1.0).unwrap().to_bytes();
            b.to_vec()
        }),
        ("`Fixed256::try_from(-1.0)`", {
            let b: [u8; 33] = Fixed256::try_from(-1.0).unwrap().to_bytes();
            b.to_vec()
        }),
        ("`Fixed256::try_from(2f64.powi(-44))`", {
            let b: [u8; 33] = Fixed256::try_from(2f64.powi(-44)).unwrap().to_bytes();
            b.to_vec()
        }),
    ];
    assert_eq!(vectors.len(), 11);
    for (expr, bytes) in vectors {
        let row = format
            .lines()
            .find(|l| l.starts_with("| ") && l.contains(expr))
            .unwrap_or_else(|| panic!("FORMAT.md has no vector row for {expr}"));
        let notation = row
            .trim_end_matches('|')
            .trim_end()
            .rsplit('`')
            .nth(1)
            .unwrap_or_else(|| panic!("no notation on {row:?}"));
        assert_eq!(
            hex(&expand(notation)),
            hex(&bytes),
            "FORMAT.md's vector for {expr} is not to_bytes"
        );
    }
}
