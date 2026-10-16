//! The heap form of `Exact` on the wire, FORMAT.md § Exact, heap form: `encode` is the tag byte
//! then the block as `Exact` holds it, four packed header limbs and the four runs of limbs, so
//! one layout serves memory, the wire and the archive; `from_bytes` reads it back, validates
//! canonical form and refuses every other spelling by name without allocating; and the `serde`
//! impls carry the same bytes. The compact encoding is untouched, which tests/wire.rs pins. Every
//! test is named `wire_heap_*`.

use std::collections::HashMap;

use float_abel::{Exact, WireError, EXACT_MAX_EXP, EXACT_MAX_LIMBS};
use num_bigint::BigInt;

use crate::exact_heap::{two_to, units};
use crate::support::{allocations, hash, rounds, Rng};

/// Limbs of header in front of the runs, one per run: the finite part, then the NaN, +∞ and −∞
/// amounts.
const HEADER: usize = 4;
/// The shortest heap encoding: the tag and the header, every run empty.
const LEAST: usize = 1 + 8 * HEADER;
/// The empty run: the zero.
const NONE: &[u64] = &[];
/// A 129-bit run, past the width in the fewest limbs with a zero limb inside.
const THREE: &[u64] = &[1, 0, 1];

fn e(x: f64) -> Exact {
    Exact::from(x)
}

/// One header limb as FORMAT.md lays it out: the sign in the top bit, the limb count in the next
/// 31, the exponent as an `i32` in the low 32.
fn entry(neg: bool, exp: i32, len: usize) -> u64 {
    (u64::from(neg) << 63) | ((len as u64) << 32) | u64::from(exp as u32)
}

/// A heap encoding built by hand from four runs of `(sign, exponent, limbs)`. Nothing here is
/// validated, so a test can build a body no operation produces.
fn heap_bytes(runs: &[(bool, i32, &[u64]); HEADER]) -> Vec<u8> {
    let mut out = vec![Exact::HEAP_TAG];
    for &(neg, exp, limbs) in runs {
        out.extend_from_slice(&entry(neg, exp, limbs.len()).to_le_bytes());
    }
    for &(_, _, limbs) in runs {
        for limb in limbs {
            out.extend_from_slice(&limb.to_le_bytes());
        }
    }
    out
}

/// A body whose finite part is `finite` and whose amounts are all zero.
fn finite_only(neg: bool, exp: i32, finite: &[u64]) -> Vec<u8> {
    heap_bytes(&[
        (neg, exp, finite),
        (false, 0, NONE),
        (false, 0, NONE),
        (false, 0, NONE),
    ])
}

/// The header of a heap encoding, unpacked: `(sign, exponent, limb count)` per run.
fn header(bytes: &[u8]) -> [(bool, i32, usize); HEADER] {
    assert_eq!(bytes[0], Exact::HEAP_TAG);
    std::array::from_fn(|i| {
        let at = 1 + 8 * i;
        let e = u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap());
        (
            e >> 63 != 0,
            e as u32 as i32,
            ((e >> 32) & 0x7FFF_FFFF) as usize,
        )
    })
}

/// The limbs behind the header of a heap encoding, every run's together.
fn limbs(bytes: &[u8]) -> Vec<u64> {
    bytes[LEAST..]
        .chunks_exact(8)
        .map(|c| u64::from_le_bytes(c.try_into().unwrap()))
        .collect()
}

/// A finite value's normalised run from the oracle: `(sign, exponent, limbs)` with the lowest set
/// bit at the exponent and no leading zero limb.
fn oracle_run(value: &BigInt) -> (bool, i32, Vec<u64>) {
    let (sign, mag) = (value.sign(), value.magnitude());
    let tz = mag.trailing_zeros().expect("a nonzero value") as i32;
    let mag = mag >> tz;
    (
        sign == num_bigint::Sign::Minus,
        tz - 1074,
        mag.to_u64_digits(),
    )
}

const fn length(expected: usize, found: usize) -> WireError {
    WireError::Length { expected, found }
}

/// postcard's framing of a byte string: a varint length, then the bytes.
fn framed(body: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut n = body.len();
    while n >= 0x80 {
        out.push((n & 0x7f) as u8 | 0x80);
        n >>= 7;
    }
    out.push(n as u8);
    out.extend_from_slice(body);
    out
}

/// `x` through every path back to itself: `from_bytes` on `encode`, postcard's byte string and
/// `serde_json`'s array of numbers, structurally and by hash.
fn round_trips(x: &Exact) {
    let bytes = x.encode();
    let back = Exact::from_bytes(&bytes).unwrap_or_else(|err| panic!("{err}: {x:?}"));
    assert_eq!(&back, x);
    assert_eq!(hash(&back), hash(x));
    assert_eq!(back.is_compact(), x.is_compact());
    assert_eq!(back.encode(), bytes);
    let pc = postcard::to_allocvec(x).unwrap();
    assert_eq!(pc, framed(&bytes));
    assert_eq!(&postcard::from_bytes::<Exact>(&pc).unwrap(), x);
    let json = serde_json::to_string(x).unwrap();
    assert_eq!(json, serde_json::to_string(&bytes[..]).unwrap());
    assert_eq!(&serde_json::from_str::<Exact>(&json).unwrap(), x);
}

#[test]
fn wire_heap_encode_is_the_tag_then_the_block_and_round_trips() {
    // 1e300 + 1e-300 spans from 2^-1049 up to 2^996, 2046 bits in 32 limbs, and no amounts.
    let x = e(1e300) + e(1e-300);
    assert!(!x.is_compact());
    let bytes = x.encode();
    assert_eq!(bytes[0], Exact::HEAP_TAG);
    assert_eq!(bytes.len(), LEAST + 8 * 32);
    let (neg, exp, run) = oracle_run(&(units(1e300) + units(1e-300)));
    assert_eq!(run.len(), 32);
    assert_eq!(
        header(&bytes),
        [(neg, exp, 32), (false, 0, 0), (false, 0, 0), (false, 0, 0)]
    );
    assert_eq!(limbs(&bytes), run);
    assert_eq!(limbs(&bytes)[0] & 1, 1);
    assert_ne!(limbs(&bytes)[31], 0);
    round_trips(&x);
    // The negation flips the sign bit of the one nonzero run and nothing else.
    let minus = -&x;
    let mut expected = bytes.clone();
    expected[8] |= 0x80;
    assert_eq!(minus.encode(), expected);
    round_trips(&minus);
    // 2.5 · NaN: no finite part, and the NaN amount 5 · 2^-1 as one limb.
    let half_nan = &e(2.5) * &e(f64::NAN);
    let bytes = half_nan.encode();
    assert_eq!(bytes.len(), LEAST + 8);
    assert_eq!(
        header(&bytes),
        [(false, 0, 0), (false, -1, 1), (false, 0, 0), (false, 0, 0)]
    );
    assert_eq!(limbs(&bytes), [5]);
    round_trips(&half_nan);
    // A count past `i32`: 2^31 of +∞ is the limb 1 at exponent 31, and a count of `i32::MIN` has
    // no compact spelling either, so it is the heap form too.
    let past = &e(f64::INFINITY) * i64::from(i32::MAX) + e(f64::INFINITY);
    assert!(!past.is_compact());
    assert_eq!(
        header(&past.encode()),
        [(false, 0, 0), (false, 0, 0), (false, 31, 1), (false, 0, 0)]
    );
    assert_eq!(limbs(&past.encode()), [1]);
    round_trips(&past);
    let min = &e(f64::NEG_INFINITY) * i64::from(i32::MIN);
    assert!(!min.is_compact());
    assert_eq!(header(&min.encode())[3], (true, 31, 1));
    round_trips(&min);
    // Every run at once: a promoted finite part beside a dyadic amount and a negative one.
    let mixed = &(&x + &half_nan) - &(&e(0.25) * &e(f64::NEG_INFINITY));
    let h = header(&mixed.encode());
    assert_eq!(h[0], (neg, exp, 32));
    assert_eq!(h[1], (false, -1, 1));
    assert_eq!(h[2], (false, 0, 0));
    assert_eq!(h[3], (true, -2, 1));
    assert_eq!(limbs(&mixed.encode()), {
        let mut all = run.clone();
        all.extend([5, 1]);
        all
    });
    round_trips(&mixed);
    // 2^127 + 1 is the narrowest finite part past the width: two limbs.
    let narrowest = two_to(127) + e(1.0);
    assert!(!narrowest.is_compact());
    assert_eq!(header(&narrowest.encode())[0], (false, 0, 2));
    assert_eq!(limbs(&narrowest.encode()), [1, 1 << 63]);
    round_trips(&narrowest);
    // The compact form's `encode` is `to_bytes`, byte for byte, and a demoted value is compact
    // again on the wire.
    for c in [e(1.5), Exact::ZERO, e(f64::NAN) * 3i64, &x - &e(1e-300)] {
        assert!(c.is_compact());
        assert_eq!(c.encode(), c.to_bytes().to_vec());
        round_trips(&c);
    }
}

#[test]
fn wire_heap_random_promoted_values_round_trip_and_distinct_values_have_distinct_bytes() {
    let mut rng = Rng(0x4EA9);
    let mut seen: HashMap<Vec<u8>, Exact> = HashMap::new();
    let mut heap = 0;
    let draws = rounds(1_000);
    for i in 0..draws {
        let a = e(rng.on_grid(-300, 300));
        let b = e(rng.on_grid(-300, 300));
        // A product of two doubles spans 106 bits and stays compact; a product of a sum does not.
        let value = match i % 4 {
            0 => &a + &b,
            1 => &(&a + &b) * &b,
            2 => &(&a * &e(f64::NAN)) + &b,
            _ => &(&a + &b) * i64::from(rng.next() as i32),
        };
        heap += usize::from(!value.is_compact());
        round_trips(&value);
        let bytes = value.encode();
        if let Some(earlier) = seen.insert(bytes.clone(), value.clone()) {
            assert_eq!(earlier, value, "two values share the bytes {bytes:02x?}");
        }
    }
    assert!(heap > draws * 3 / 4, "{heap} of {draws} promoted");
    assert!(seen.len() > draws * 19 / 20, "{} distinct", seen.len());
}

/// The refusals the decoder reaches before any run: the cap on the header alone, then the length.
fn refused_before_the_runs() -> Vec<(Vec<u8>, WireError)> {
    let cap = EXACT_MAX_LIMBS;
    let past_cap = |at: usize, len: usize| {
        let mut b = heap_bytes(&[(false, 0, NONE); HEADER]);
        b[at..at + 8].copy_from_slice(&entry(false, 0, len).to_le_bytes());
        b
    };
    let three_limbs = finite_only(false, 0, THREE);
    vec![
        // A count one past the cap in any run, with no limbs behind it at all.
        (past_cap(1, cap + 1), WireError::Limbs),
        (past_cap(25, cap + 1), WireError::Limbs),
        (past_cap(1, 1 << 30), WireError::Limbs),
        // The length the header implies.
        (
            three_limbs[..LEAST + 16].to_vec(),
            length(LEAST + 24, LEAST + 16),
        ),
        (
            [&three_limbs[..], &[0]].concat(),
            length(LEAST + 24, LEAST + 25),
        ),
        (vec![Exact::HEAP_TAG], length(LEAST, 1)),
        (vec![Exact::HEAP_TAG; 10], length(LEAST, 10)),
        (vec![Exact::HEAP_TAG; LEAST - 1], length(LEAST, LEAST - 1)),
    ]
}

/// The refusals of a run that is not normalised, each in a run that would otherwise stand.
fn refused_runs() -> Vec<(Vec<u8>, WireError)> {
    use WireError::{EvenMantissa, Exponent, Form};
    let bound = EXACT_MAX_EXP;
    vec![
        // A sign or an exponent on an empty run is a second spelling of the zero.
        (
            heap_bytes(&[
                (false, 0, THREE),
                (true, 0, NONE),
                (false, 0, NONE),
                (false, 0, NONE),
            ]),
            Form,
        ),
        (
            heap_bytes(&[
                (false, 0, THREE),
                (false, 0, NONE),
                (false, 7, NONE),
                (false, 0, NONE),
            ]),
            Form,
        ),
        // An even low limb puts the lowest set bit above the exponent.
        (finite_only(false, 0, &[2, 1, 1]), EvenMantissa),
        (
            heap_bytes(&[
                (false, 0, THREE),
                (false, 0, &[4]),
                (false, 0, NONE),
                (false, 0, NONE),
            ]),
            EvenMantissa,
        ),
        // A leading zero limb.
        (finite_only(false, 0, &[1, 1, 0]), Form),
        (
            heap_bytes(&[
                (false, 0, THREE),
                (false, 0, NONE),
                (false, 0, &[1, 0]),
                (false, 0, NONE),
            ]),
            Form,
        ),
        // The exponent bound.
        (finite_only(false, bound + 1, THREE), Exponent),
        (
            heap_bytes(&[
                (false, 0, THREE),
                (true, -bound - 1, &[5]),
                (false, 0, NONE),
                (false, 0, NONE),
            ]),
            Exponent,
        ),
    ]
}

/// A value that fits the compact form belongs in its encoding; and the checks run in the order
/// above, so the first failing one names the refusal.
fn refused_as_compact_and_in_order() -> Vec<(Vec<u8>, WireError)> {
    use WireError::{EvenMantissa, Exponent, Form};
    let bound = EXACT_MAX_EXP;
    vec![
        // The zero, 1.5, a 127-bit finite part, and `i32` counts beside a compact finite part.
        (heap_bytes(&[(false, 0, NONE); HEADER]), Form),
        (finite_only(false, -1, &[3]), Form),
        (finite_only(true, 5, &[u64::MAX, i64::MAX as u64]), Form),
        (
            heap_bytes(&[
                (false, 0, NONE),
                (false, 0, &[i32::MAX as u64]),
                (true, 30, &[1]),
                (false, 0, &[3]),
            ]),
            Form,
        ),
        // An even limb before an exponent past the bound and a sign on an empty run; a leading
        // zero limb before them; the exponent before the sign.
        (
            heap_bytes(&[
                (false, bound + 1, &[2, 0]),
                (true, 0, NONE),
                (false, 0, NONE),
                (false, 0, NONE),
            ]),
            EvenMantissa,
        ),
        (
            heap_bytes(&[
                (false, bound + 1, &[1, 0]),
                (true, 0, NONE),
                (false, 0, NONE),
                (false, 0, NONE),
            ]),
            Form,
        ),
        (
            heap_bytes(&[
                (false, bound + 1, THREE),
                (true, 0, NONE),
                (false, 0, NONE),
                (false, 0, NONE),
            ]),
            Exponent,
        ),
    ]
}

/// Each bound is itself a value: the exponent bound under a 129-bit finite part, a 128-bit
/// finite part, and amounts just past `i32` on either side.
fn accepted_bodies() -> Vec<Vec<u8>> {
    let bound = EXACT_MAX_EXP;
    vec![
        finite_only(false, bound, THREE),
        finite_only(true, -bound, THREE),
        finite_only(false, 0, &[1, 1 << 63]),
        heap_bytes(&[
            (false, 0, NONE),
            (false, 31, &[1]),
            (false, 0, NONE),
            (false, 0, NONE),
        ]),
        heap_bytes(&[
            (false, 0, NONE),
            (false, 0, NONE),
            (true, 31, &[1]),
            (false, 0, NONE),
        ]),
        heap_bytes(&[
            (false, 0, NONE),
            (false, 0, NONE),
            (false, 0, NONE),
            (false, -1, &[1]),
        ]),
    ]
}

#[test]
fn wire_heap_refuses_each_non_canonical_body_by_name_without_allocating() {
    let refused = [
        refused_before_the_runs(),
        refused_runs(),
        refused_as_compact_and_in_order(),
    ]
    .concat();
    assert_eq!(refused.len(), 23);
    for (bytes, err) in &refused {
        let (got, allocated) = allocations(|| Exact::from_bytes(bytes));
        assert_eq!(got.as_ref(), Err(err), "{bytes:02x?}");
        assert_eq!(allocated, 0, "{err} allocated on {bytes:02x?}");
        // The serde path carries the same refusal, with the decoder's words, which serde_json
        // follows with the position.
        let json = serde_json::from_str::<Exact>(&format!("{bytes:?}")).unwrap_err();
        assert!(
            json.to_string().starts_with(&err.to_string()),
            "{json} on {bytes:02x?}"
        );
        assert!(postcard::from_bytes::<Exact>(&framed(bytes)).is_err());
    }
    // Each heap refusal says why.
    for (err, word) in [
        (WireError::Limbs, "EXACT_MAX_LIMBS"),
        (WireError::Form, "compact"),
    ] {
        let text = err.to_string();
        assert!(text.contains(word), "{text:?} lacks {word:?}");
    }
    // A well-formed body costs one allocation, and a decoded word is a value the operations take
    // as it is, which leaves the heap form the way any value does.
    let accepted = accepted_bodies();
    assert_eq!(accepted.len(), 6);
    for bytes in &accepted {
        let (got, allocated) = allocations(|| Exact::from_bytes(bytes));
        let value = got.unwrap_or_else(|err| panic!("{err} on {bytes:02x?}"));
        assert_eq!(allocated, 1, "{bytes:02x?}");
        assert!(!value.is_compact());
        assert_eq!(value.encode(), *bytes);
        assert_eq!(value.checked_add(&Exact::ZERO), Ok(value.clone()));
        assert_eq!(value.checked_mul_i64(1), Ok(value.clone()));
        assert!((&value - &value).is_zero());
    }
    // The cap's worth of limbs in every run is a value the decoder takes and gives back; what the
    // operations do with a value at the cap's exact edge is their own boundary (the store's task
    // on the cap's closure), not the wire's.
    let full: Vec<u64> = std::iter::repeat(u64::MAX).take(EXACT_MAX_LIMBS).collect();
    let at_cap = heap_bytes(&[
        (false, 0, &full),
        (false, 0, &full),
        (false, 0, &full),
        (false, 0, &full),
    ]);
    let (got, allocated) = allocations(|| Exact::from_bytes(&at_cap));
    let value = got.unwrap();
    assert_eq!(allocated, 1);
    assert_eq!(value.encode(), at_cap);
    assert_eq!(value.checked_add(&Exact::ZERO), Ok(value.clone()));
    // `from_bytes` dispatches on the first byte, so a heap body of 33 bytes is never read as a
    // compact word, and a compact word is never read as a heap body.
    let compact = e(1.5).to_bytes();
    assert_eq!(Exact::from_bytes(&compact), Ok(e(1.5)));
    let mut tagged = compact;
    tagged[0] = Exact::HEAP_TAG;
    assert_ne!(Exact::from_bytes(&tagged), Ok(e(1.5)));
    assert!(Exact::from_bytes(&tagged).is_err());
}

/// A compact word's finite part and counts as the heap form's runs: its heap spelling, which the
/// decoder refuses because the value fits the compact form.
fn heap_spelling(value: &Exact) -> Vec<u8> {
    let bytes = value.to_bytes();
    let lo = u64::from_le_bytes(bytes[1..9].try_into().unwrap());
    let hi = u64::from_le_bytes(bytes[9..17].try_into().unwrap());
    let exp = i32::from_le_bytes(bytes[17..21].try_into().unwrap());
    let counts = [21, 25, 29].map(|at| i32::from_le_bytes(bytes[at..at + 4].try_into().unwrap()));
    let m = (u128::from(hi) << 64 | u128::from(lo)) as i128;
    let finite: (bool, i32, Vec<u64>) = if m == 0 {
        (false, 0, vec![])
    } else {
        let mag = m.unsigned_abs();
        let limbs = if mag >> 64 == 0 {
            vec![mag as u64]
        } else {
            vec![mag as u64, (mag >> 64) as u64]
        };
        (m < 0, exp, limbs)
    };
    let amounts: Vec<(bool, i32, Vec<u64>)> = counts
        .iter()
        .map(|&c| {
            if c == 0 {
                (false, 0, vec![])
            } else {
                let mag = u64::from(c.unsigned_abs());
                let tz = mag.trailing_zeros();
                (c < 0, tz as i32, vec![mag >> tz])
            }
        })
        .collect();
    heap_bytes(&[
        (finite.0, finite.1, &finite.2),
        (amounts[0].0, amounts[0].1, &amounts[0].2),
        (amounts[1].0, amounts[1].1, &amounts[1].2),
        (amounts[2].0, amounts[2].1, &amounts[2].2),
    ])
}

#[test]
fn wire_heap_the_decoder_and_the_operations_agree_on_which_form_holds_a_value() {
    // Whatever the operations land in the heap form, the decoder accepts as the heap form; and a
    // heap spelling of whatever they land compact, it refuses. The two decide by one rule.
    let mut rng = Rng(0x0F17);
    let (mut heap, mut compact) = (0, 0);
    let draws = rounds(2_000);
    for _ in 0..draws {
        let a = e(rng.on_grid(-80, 80));
        let b = e(rng.on_grid(-80, 80));
        let scale = i64::from(rng.next() as i32 >> (rng.next() % 32));
        let value = &(&a + &b) + &(&e(f64::NAN) * scale);
        if value.is_compact() {
            compact += 1;
            assert_eq!(
                Exact::from_bytes(&heap_spelling(&value)),
                Err(WireError::Form),
                "{value:?}"
            );
        } else {
            heap += 1;
            assert_eq!(Exact::from_bytes(&value.encode()), Ok(value.clone()));
        }
    }
    // Exponents 2^±80 put a sum past the width when they differ by more than 74: about a third.
    assert!(
        heap > draws / 20 && compact > draws / 20,
        "{heap} heap, {compact} compact of {draws}"
    );
}

/// FORMAT.md's heap vectors are `encode` byte for byte, found by their expressions as the compact
/// ones are in tests/wire.rs.
#[test]
fn wire_heap_vectors_in_format_md_are_encode_byte_for_byte() {
    let format = include_str!("../FORMAT.md");
    let vectors: Vec<(&str, Vec<u8>)> = vec![
        (
            "`Exact::from(2.5) * Exact::from(f64::NAN)`",
            (&e(2.5) * &e(f64::NAN)).encode(),
        ),
        (
            "`Exact::from(f64::INFINITY) * (i32::MAX as i64) + Exact::from(f64::INFINITY)`",
            (&e(f64::INFINITY) * i64::from(i32::MAX) + e(f64::INFINITY)).encode(),
        ),
        ("2^127 + 1", (two_to(127) + e(1.0)).encode()),
    ];
    assert_eq!(vectors.len(), 3);
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
        let mut expanded = Vec::new();
        for token in notation.split_whitespace() {
            let (byte, run) = match token.split_once('*') {
                Some((b, n)) => (b, n.parse::<usize>().unwrap()),
                None => (token, 1),
            };
            expanded.extend(std::iter::repeat(u8::from_str_radix(byte, 16).unwrap()).take(run));
        }
        assert_eq!(
            expanded, bytes,
            "FORMAT.md's vector for {expr} is not encode"
        );
    }
}
