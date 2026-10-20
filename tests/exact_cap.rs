//! `Exact` through its public surface: the operators by value and by reference, their checked
//! twins, the named cap, canonical equality, order, and the reads against IEEE, `Wide` and
//! `Fixed256`, on finite inputs, and the closure of the group at the limb cap's exact edge against
//! a `num-bigint` oracle (`exact_cap_closed_*`). Every test is named `exact_cap_*` so the task
//! verify can filter on it; `tests/exact_ring.rs` holds the specials and the ring, and
//! `tests/noalloc` the no-feature build.

use core::cmp::Ordering;
use float_abel::{Exact, Fixed256, PastCap, Wide, EXACT_MAX_EXP, EXACT_MAX_LIMBS};
use num_bigint::{BigInt, Sign};

use crate::exact_heap::two_to;
use crate::support::{hash, pow2, rounds, Rng};
use crate::wire_heap::{at_cap_body, header, limbs};

const _: () = assert!(core::mem::size_of::<Exact>() == 32);

fn e(x: f64) -> Exact {
    Exact::from(x)
}

/// `2^(2^30)`, the value at the exponent bound, by squaring `2` thirty times.
fn at_bound() -> Exact {
    (0..30).fold(e(2.0), |x, _| &x * &x)
}

#[test]
fn exact_cap_size_zero_and_constants() {
    assert_eq!(core::mem::size_of::<Exact>(), 32);
    assert_eq!(EXACT_MAX_EXP, 1 << 30);
    assert_eq!(Exact::default(), Exact::ZERO);
    assert!(Exact::ZERO.is_zero());
    assert!(Exact::ZERO.is_finite());
    assert_eq!(Exact::ZERO.to_f64().to_bits(), 0.0f64.to_bits());
    assert_eq!(e(0.0), Exact::ZERO);
    assert_eq!(e(-0.0), Exact::ZERO);
    assert_eq!(Exact::product(3.0, 0.5), e(1.5));
}

#[test]
#[should_panic(expected = "exponent")]
fn exact_cap_an_operator_past_the_exponent_bound_panics_by_name() {
    let top = at_bound();
    let _ = &top * &top;
}

#[test]
#[should_panic(expected = "exponent")]
fn exact_cap_a_scalar_multiply_past_the_exponent_bound_panics_by_name() {
    let _ = at_bound() * 2i64;
}

#[test]
fn exact_cap_the_checked_methods_return_the_cap_and_leave_the_state() {
    let top = at_bound();
    let bottom = (0..30).fold(e(0.5), |x, _| &x * &x);
    assert_eq!(top.to_f64(), f64::INFINITY);
    assert_eq!(bottom.to_f64().to_bits(), 0.0f64.to_bits());
    assert_eq!(top.try_to_f64(), None);
    assert_eq!(top.checked_mul(&e(2.0)), Err(PastCap::Exponent));
    assert_eq!(e(2.0).checked_mul(&top), Err(PastCap::Exponent));
    assert_eq!(top.checked_mul(&top), Err(PastCap::Exponent));
    assert_eq!(top.checked_mul_i64(2), Err(PastCap::Exponent));
    assert_eq!(top.checked_mul_i64(-2), Err(PastCap::Exponent));
    assert_eq!(top.checked_add(&top), Err(PastCap::Exponent));
    assert_eq!(top.checked_sub(&-&top), Err(PastCap::Exponent));
    assert_eq!(bottom.checked_mul(&e(0.5)), Err(PastCap::Exponent));
    // At the bottom a doubling moves the canonical exponent toward zero, so it stays inside.
    assert_eq!(bottom.checked_add(&bottom).map(|v| v.is_finite()), Ok(true));
    // Nothing moved, and the same shapes one step inside the bound hold.
    assert_eq!(top, at_bound());
    assert_eq!(top.checked_mul(&e(0.5)).map(|v| v.is_finite()), Ok(true));
    assert_eq!(
        top.checked_mul_i64(3).map(|v| v.to_f64()),
        Ok(f64::INFINITY)
    );
    assert_eq!(top.checked_mul(&bottom), Ok(e(1.0)));
    assert_eq!(&top * &bottom, e(1.0));
    assert_eq!(top.checked_sub(&top), Ok(Exact::ZERO));
    // A deposit that fits changes the state; one that does not leaves it.
    let mut s = e(1.5);
    assert_eq!(s.checked_deposit(2.5), Ok(()));
    assert_eq!(s.checked_deposit_product(2.0, 3.0), Ok(()));
    assert_eq!(s, e(10.0));
    // The cap names itself, and is an error type under std.
    assert!(PastCap::Exponent.to_string().contains("exponent"));
    assert!(PastCap::Width.to_string().contains("width"));
    assert!(PastCap::Count.to_string().contains("count"));
    let _: &dyn std::error::Error = &PastCap::Exponent;
}

#[test]
#[allow(clippy::erasing_op)] // scaling by a literal zero is one of the cases under test
#[allow(clippy::op_ref)] // the reference operators are the surface under test here
fn exact_cap_group_and_ring_laws_hold_through_the_operators() {
    let mut rng = Rng(30);
    for _ in 0..rounds(20_000) {
        let (a, b, cc) = (
            e(rng.on_grid(-20, 20)),
            e(rng.on_grid(-20, 20)),
            e(rng.on_grid(-20, 20)),
        );
        assert_eq!(&a + &b, &b + &a);
        assert_eq!(&(&a + &b) + &cc, &a + &(&b + &cc));
        assert_eq!(&a + &Exact::ZERO, a);
        assert_eq!(&a + &-&a, Exact::ZERO);
        assert_eq!(&(&a + &b) - &b, a);
        assert_eq!(-(-&a), a);
        assert_eq!(&a - &b, &a + &-&b);
        assert_eq!(a.clone() + b.clone(), &a + &b);
        assert_eq!(a.clone() - b.clone(), &a - &b);
        assert_eq!(-a.clone(), -&a);
        let mut aa = a.clone();
        aa += &b;
        aa -= &b;
        assert_eq!(aa, a);
        aa += b.clone();
        aa -= b.clone();
        assert_eq!(aa, a);
        // Integer scaling is repeated addition.
        assert_eq!(&a * 3i64, &(&a + &a) + &a);
        assert_eq!(&a * -1i64, -&a);
        assert_eq!(&a * 0i64, Exact::ZERO);
        assert_eq!(a.clone() * 2isize, &a + &a);
        assert_eq!(&a * 2isize, &a + &a);
        assert_eq!(&(&a + &b) * 5i64, &(&a * 5i64) + &(&b * 5i64));
        // The ring laws, with 30-bit integer factors so that three products stay inside the
        // tier: 53 + 30 + 30 bits.
        let (p, q) = (
            e(((rng.next() >> 34) as i64 - (1 << 29)) as f64),
            e(((rng.next() >> 34) as i64 - (1 << 29)) as f64),
        );
        assert_eq!(&a * &b, &b * &a);
        assert_eq!(a.clone() * b.clone(), &a * &b);
        assert_eq!(&(&a * &p) * &q, &a * &(&p * &q));
        assert_eq!(&a * &(&p + &q), &(&a * &p) + &(&a * &q));
        assert_eq!(&(&a + &b) * &p, &(&a * &p) + &(&b * &p));
        // Sum by value and by reference are the fold.
        let xs = [a.clone(), b.clone(), cc.clone()];
        let by_value: Exact = xs.iter().cloned().sum();
        let by_ref: Exact = xs.iter().sum();
        assert_eq!(by_value, &(&a + &b) + &cc);
        assert_eq!(by_ref, by_value);
    }
}

#[test]
fn exact_cap_a_plus_b_minus_b_is_a_on_random_pairs() {
    let mut rng = Rng(31);
    for _ in 0..rounds(200_000) {
        let (a, b) = (e(rng.on_grid(-20, 20)), e(rng.on_grid(-20, 20)));
        assert_eq!((&a + &b) - &b, a, "{a:?} + {b:?} - {b:?}");
    }
}

#[test]
fn exact_cap_canonical_form_makes_eq_hash_and_order_numeric() {
    assert_eq!(e(0.5) + e(0.25), e(0.75));
    assert_eq!(hash(&(e(0.5) + e(0.25))), hash(&e(0.75)));
    assert_eq!(e(3.0) - e(1.0), e(1.0) + e(1.0));
    assert_eq!(hash(&(e(3.0) - e(1.0))), hash(&(e(1.0) + e(1.0))));
    assert_eq!(e(1.5) - e(1.5), Exact::ZERO);
    assert_eq!(hash(&(e(1.5) - e(1.5))), hash(&Exact::ZERO));
    assert_eq!(e(12.0), e(3.0) * e(4.0));
    assert_eq!(e(12.0), e(1.5) * 8i64);
    let mut rng = Rng(32);
    for _ in 0..rounds(50_000) {
        let (a, b) = (rng.finite(), rng.finite());
        assert_eq!(
            e(a).cmp(&e(b)),
            a.partial_cmp(&b).unwrap(),
            "{a:e} vs {b:e}"
        );
        assert_eq!(e(a).partial_cmp(&e(b)), Some(e(a).cmp(&e(b))));
    }
    assert!(e(-5e-324) < Exact::ZERO);
    assert!(Exact::ZERO < e(5e-324));
    assert!(e(-f64::MAX) < e(5e-324));
    assert!(e(1.0) + e(pow2(126)) > e(pow2(126)));
    assert!(e(1.0) + e(pow2(126)) < e(pow2(127)));
    let mut xs: Vec<Exact> = (0..300)
        .map(|i| {
            if i % 3 == 0 {
                e(rng.on_grid(-30, 30)) + e(rng.on_grid(-30, 30))
            } else {
                e(rng.finite())
            }
        })
        .collect();
    xs.sort();
    for pair in xs.windows(2) {
        assert_ne!(pair[0].cmp(&pair[1]), Ordering::Greater);
        if pair[0].cmp(&pair[1]) == Ordering::Equal {
            assert_eq!(pair[0], pair[1]);
        }
    }
}

#[test]
fn exact_cap_reads_round_like_ieee_and_agree_with_wide_and_fixed() {
    let mut rng = Rng(33);
    for _ in 0..rounds(50_000) {
        let (a, b) = (rng.on_grid(-1000, 1000), rng.on_grid(-1000, 1000));
        let Ok(sum) = e(a).checked_add(&e(b)) else {
            continue;
        };
        let wide = Wide::from(a) + Wide::from(b);
        assert_eq!(sum.to_f64().to_bits(), (a + b).to_bits(), "{a:e} + {b:e}");
        assert_eq!(sum.try_to_f64(), wide.try_to_f64(), "{a:e} + {b:e}");
        let n = (rng.next() >> 11).max(1);
        assert_eq!(
            sum.div_f64(n).to_bits(),
            wide.div_f64(n).to_bits(),
            "{a:e} + {b:e} / {n}"
        );
    }
    for _ in 0..rounds(20_000) {
        // Leading bits within [-44, 30): on Fixed256's grid, and a pair spans at most 127 bits.
        let (a, b) = (rng.on_grid(-44, 30), rng.on_grid(-44, 30));
        let fixed = Fixed256::try_from(a).unwrap() + Fixed256::try_from(b).unwrap();
        let exact = e(a) + e(b);
        assert_eq!(
            exact.to_f64().to_bits(),
            fixed.to_f64().to_bits(),
            "{a:e} + {b:e}"
        );
        assert_eq!(exact.try_to_f64(), fixed.try_to_f64());
        let n = (rng.next() >> 11).max(1);
        assert_eq!(exact.div_f64(n).to_bits(), fixed.div_f64(n).to_bits());
    }
    for _ in 0..rounds(20_000) {
        let (a, b) = (rng.on_grid(-480, 480), rng.on_grid(-480, 480));
        let mut p = Exact::ZERO;
        p.deposit_product(a, b);
        assert_eq!(p.to_f64().to_bits(), (a * b).to_bits(), "{a:e} * {b:e}");
        assert_eq!(p, Exact::product(a, b));
        assert_eq!(p, e(a) * e(b));
        let mut w = Wide::ZERO;
        w.deposit_product(a, b).unwrap();
        assert_eq!(p.try_to_f64(), w.try_to_f64(), "{a:e} * {b:e}");
    }
    // The classic, a mean of ten exact tenths, and division by zero.
    assert_eq!((e(0.1) + e(0.2)).to_f64(), 0.1 + 0.2);
    assert_eq!((e(0.1) + e(0.2)).try_to_f64(), None);
    let mut tenths = Exact::ZERO;
    for _ in 0..10 {
        tenths.deposit(0.1);
    }
    assert_eq!(tenths.to_f64(), 1.0);
    assert_eq!(tenths.try_to_f64(), None);
    assert_eq!(tenths.div_f64(10), 0.1);
    assert!(Exact::ZERO.div_f64(0).is_nan());
    assert_eq!(e(1.0).div_f64(0), f64::INFINITY);
    assert_eq!(e(-1.0).div_f64(0), f64::NEG_INFINITY);
    // A cancellation across 300 orders of magnitude inside the width keeps the small term.
    let small = pow2(-100);
    assert_eq!((e(1.0) + e(small) - e(1.0)).try_to_f64(), Some(small));
    assert_eq!((e(f64::MAX) + e(f64::MAX)).to_f64(), f64::INFINITY);
    assert_eq!((e(f64::MAX) + e(f64::MAX)).div_f64(2), f64::MAX);
}

#[test]
fn exact_cap_debug_names_the_value_and_the_tier() {
    let s = format!("{:?}", e(1.5));
    assert!(s.starts_with("Exact(1.5e0; nan 0, +inf 0, -inf 0;"), "{s}");
    assert!(s.contains("0x0:0x3"), "{s}");
    assert!(s.ends_with("2^-1)"), "{s}");
    let n = format!("{:?}", e(-1.5));
    assert!(
        n.starts_with("Exact(-1.5e0; nan 0, +inf 0, -inf 0; 0xffffffffffffffff:0xfffffffffffffffd"),
        "{n}"
    );
    let x = format!("{:?}", e(0.1) + e(0.2));
    assert!(x.starts_with("Exact(≈3.0000000000000004e-1;"), "{x}");
    let z = format!("{:?}", Exact::ZERO);
    assert_eq!(z, "Exact(0e0; nan 0, +inf 0, -inf 0; 0x0:0x0 · 2^0)");
}

/// A sum lands compact iff its own span fits, whatever the raised operand needed on the way: the
/// add agrees with the `Wide` route on every pair, either way round, and a sum past the width is
/// the heap form, not a refusal. `2^127 + (−1)` is 127 one-bits, inside the width, though `2^127`
/// raised to the unit needs 128; `2^127 + 1` is 128 bits and promotes by both routes.
#[test]
fn exact_cap_a_sum_that_spans_exactly_the_width_fits() {
    let via_wide = |a: &Exact, b: &Exact| {
        let w = Wide::try_from(a).unwrap() + Wide::try_from(b).unwrap();
        Exact::try_from(&w).unwrap()
    };
    // Both routes agree, and the answer is whether the sum landed compact.
    let agree = |a: &Exact, b: &Exact| {
        let want = via_wide(a, b);
        assert_eq!(a.checked_add(b), Ok(want.clone()), "{a:?} + {b:?}");
        assert_eq!(b.checked_add(a), Ok(want.clone()), "{b:?} + {a:?}");
        assert_eq!(a.checked_sub(&-b), Ok(want.clone()), "{a:?} - -{b:?}");
        want.is_compact()
    };
    let top = e(pow2(127));
    let all_ones = top.checked_add(&e(-1.0)).expect("2^127 - 1 is 127 bits");
    assert!(all_ones.is_compact());
    assert_eq!(all_ones.try_to_f64(), None);
    assert_eq!(&all_ones + &e(1.0), top);
    assert_eq!((-&top).checked_add(&e(1.0)), Ok(-&all_ones));
    assert!(!(&top + &e(1.0)).is_compact());
    assert!(!(&-&top + &e(-1.0)).is_compact());
    // At one exponent two odd mantissas sum to an even one, so a carry into bit 127 fits: the
    // widest odd value plus one is 2^127, plus itself is its double, and plus 2^k + 1 fits for
    // every k ≥ 1 while plus 2 (even, so raised) and plus 2^0 + 1 (128 odd bits) promote.
    assert_eq!(&all_ones + &all_ones, &all_ones * 2i64);
    assert_eq!(&-&all_ones + &e(-1.0), -&top);
    let (mut checked, mut compact, mut promoted) = (0, 0, 0);
    for k in 0..=126 {
        compact += usize::from(agree(&all_ones, &(e(pow2(k)) + e(1.0))));
        compact += usize::from(agree(&-&all_ones, &-(e(pow2(k)) + e(1.0))));
        checked += 2;
    }
    assert_eq!(compact, 2 * 126);
    assert!(!(&all_ones + &e(2.0)).is_compact());
    // Across the boundary: ±2^k ± 1, and ±(2^k + 2^j) ∓ (2^j + 1), whose raised operand needs
    // one bit more than the width at the unit while the sum cancels back under it.
    for k in 120..=130 {
        let p = e(pow2(k));
        for (a, b) in [
            (p.clone(), e(1.0)),
            (p.clone(), e(-1.0)),
            (-&p, e(1.0)),
            (-&p, e(-1.0)),
        ] {
            promoted += usize::from(!agree(&a, &b));
            checked += 1;
        }
        for j in [1, 32, 64, 100, 126] {
            if k - j > 126 {
                continue;
            }
            let a = p.checked_add(&e(pow2(j))).unwrap();
            assert!(a.is_compact());
            let b = -(e(pow2(j)) + e(1.0));
            promoted += usize::from(!agree(&a, &b));
            promoted += usize::from(!agree(&-&a, &-&b));
            checked += 2;
        }
    }
    // The 127 carries both ways, eleven k by four sign pairs, and the three (k, j) with
    // k − j > 126 skipped of fifty-five; the k past 127 promote.
    assert_eq!(checked, 2 * 127 + 44 + 2 * 52);
    assert!(promoted > 0, "{promoted}");
    assert!(promoted < 44 + 2 * 52, "{promoted}");
}

/// Bits a run may span under the limb cap, as `src/exact/big.rs` counts them.
const CAP_BITS: i32 = 64 * EXACT_MAX_LIMBS as i32;

/// The finite part as the oracle's integer, read off the encoding of either form. Every value
/// these tests build has a nonnegative exponent.
fn finite_of(v: &Exact) -> BigInt {
    let (neg, exp, mag): (bool, i32, Vec<u64>) = if v.is_compact() {
        let bytes = v.to_bytes();
        let lo = u64::from_le_bytes(bytes[1..9].try_into().unwrap());
        let hi = u64::from_le_bytes(bytes[9..17].try_into().unwrap());
        let exp = i32::from_le_bytes(bytes[17..21].try_into().unwrap());
        let m = (u128::from(hi) << 64 | u128::from(lo)) as i128;
        let mag = m.unsigned_abs();
        (m < 0, exp, vec![mag as u64, (mag >> 64) as u64])
    } else {
        let bytes = v.encode();
        let (neg, exp, len) = header(&bytes)[0];
        (neg, exp, limbs(&bytes)[..len].to_vec())
    };
    assert!(exp >= 0, "{v:?}");
    let le: Vec<u8> = mag.iter().flat_map(|l| l.to_le_bytes()).collect();
    let sign = if neg { Sign::Minus } else { Sign::Plus };
    BigInt::from_bytes_le(sign, &le) << exp
}

/// Whether the oracle's integer is a value under the cap: its set bits span at most `CAP_BITS`.
fn fits(x: &BigInt) -> bool {
    let mag = x.magnitude();
    match mag.trailing_zeros() {
        None => true,
        Some(tz) => mag.bits() - tz <= CAP_BITS as u64,
    }
}

/// The finite part's limb count on the wire: the cap is reached when it is `EXACT_MAX_LIMBS`.
fn finite_limbs(v: &Exact) -> usize {
    assert!(!v.is_compact(), "{v:?}");
    header(&v.encode())[0].2
}

/// Integers at the cap's edge, each built through the operations beside its oracle: values of
/// exactly the cap's width, one bit under it, powers of two above and at it, and small ones.
fn edge_shapes() -> Vec<(&'static str, Exact, BigInt)> {
    let c = CAP_BITS;
    let one = BigInt::from(1);
    let build = |name: &'static str, v: Result<Exact, PastCap>, oracle: BigInt| {
        let v = v.unwrap_or_else(|err| panic!("{name}: {err}"));
        assert_eq!(finite_of(&v), oracle, "{name}");
        (name, v, oracle)
    };
    vec![
        build(
            "2^(C-1) + 3",
            two_to(c - 1).checked_add(&e(3.0)),
            (&one << (c - 1)) + 3,
        ),
        build("2^C - 1", two_to(c).checked_sub(&e(1.0)), (&one << c) - 1),
        build(
            "2^(C-1) - 1",
            two_to(c - 1).checked_sub(&e(1.0)),
            (&one << (c - 1)) - 1,
        ),
        build(
            "2^(C-2) + 1",
            two_to(c - 2).checked_add(&e(1.0)),
            (&one << (c - 2)) + 1,
        ),
        build(
            "2^(C-2) + 2",
            two_to(c - 2).checked_add(&e(2.0)),
            (&one << (c - 2)) + 2,
        ),
        build("2^C", Ok(two_to(c)), &one << c),
        build("2^(C-1)", Ok(two_to(c - 1)), &one << (c - 1)),
        build("2^(C+64)", Ok(two_to(c + 64)), &one << (c + 64)),
        build("1", Ok(e(1.0)), one.clone()),
        build("3", Ok(e(3.0)), BigInt::from(3)),
        build(
            "2^64 + 1",
            e(pow2(64)).checked_add(&e(1.0)),
            (&one << 64) + 1,
        ),
    ]
}

/// The decoded body at the cap's exact edge, every run `EXACT_MAX_LIMBS` limbs of ones, is a
/// value the operations take: it retracts, scales by one, and refuses one bit past by name.
#[test]
fn exact_cap_closed_a_decoded_value_at_the_cap_retracts_and_scales_by_one() {
    let v = Exact::from_bytes(&at_cap_body()).unwrap();
    for run in header(&v.encode()) {
        assert_eq!(run, (false, 0, EXACT_MAX_LIMBS));
    }
    assert_eq!(v.checked_mul_i64(1), Ok(v.clone()));
    assert_eq!(v.checked_mul_i64(-1), Ok(-&v));
    assert_eq!(v.checked_mul(&e(1.0)), Ok(v.clone()));
    assert_eq!(v.checked_mul(&e(-1.0)), Ok(-&v));
    assert_eq!(v.checked_add(&Exact::ZERO), Ok(v.clone()));
    assert_eq!(v.checked_sub(&v), Ok(Exact::ZERO));
    assert_eq!(v.checked_add(&-&v), Ok(Exact::ZERO));
    assert!((&v - &v).is_zero());
    assert!((&-&v + &v).is_zero());
    assert_eq!(&v * 1i64, v);
    // Doubling moves the exponent, so the double of a value at the cap is at the cap too, and
    // taking the value back from its double crosses the edge both ways.
    let twice = v.checked_add(&v).unwrap();
    assert_eq!(twice, &v * 2i64);
    assert_eq!(finite_limbs(&twice), EXACT_MAX_LIMBS);
    assert_eq!(header(&twice.encode())[0].1, 1);
    assert_eq!(twice.checked_sub(&v), Ok(v.clone()));
    assert_eq!(&twice - &v, v);
    // The finite part is 2^C − 1: plus one it is 2^C, one limb, and back it is itself; plus two
    // it would be 2^C + 1, one bit past the cap, refused by name with the state as it was.
    let up = v.checked_add(&e(1.0)).unwrap();
    assert_eq!(finite_of(&up), BigInt::from(1) << CAP_BITS);
    assert_eq!(finite_limbs(&up), 1);
    assert_eq!(up.checked_sub(&e(1.0)), Ok(v.clone()));
    assert_eq!(v.checked_add(&e(2.0)), Err(PastCap::Limbs));
    assert_eq!(v.checked_sub(&e(-2.0)), Err(PastCap::Limbs));
    assert_eq!(v.checked_mul_i64(3), Err(PastCap::Limbs));
    assert_eq!(v.checked_mul(&e(3.0)), Err(PastCap::Limbs));
    assert_eq!(v.checked_mul(&v), Err(PastCap::Limbs));
    assert_eq!(v.encode(), at_cap_body());
    assert!(PastCap::Limbs.to_string().contains("EXACT_MAX_LIMBS"));
}

/// A sum the operations land with exactly the cap's bits is retractable: `v − v`, `v + (−v)`,
/// `v × 1`, `v × (−1)` and `(v + w) − w` hold, and one bit past is refused by name.
#[test]
fn exact_cap_closed_a_sum_that_lands_at_the_cap_retracts() {
    let cap = CAP_BITS;
    // 2^(C−2) + 1 and 2^(C−2) + 2 span C − 1 bits each; their sum carries into bit C − 1 and is
    // odd, so it spans exactly the cap.
    let a = two_to(cap - 2) + e(1.0);
    let b = two_to(cap - 2) + e(2.0);
    let v = &a + &b;
    assert_eq!(finite_of(&v), (BigInt::from(1) << (cap - 1)) + 3);
    assert_eq!(finite_limbs(&v), EXACT_MAX_LIMBS);
    assert_eq!(header(&v.encode())[0].1, 0);
    assert!((&v - &v).is_zero());
    assert_eq!(v.checked_sub(&v), Ok(Exact::ZERO));
    assert_eq!(v.checked_add(&-&v), Ok(Exact::ZERO));
    assert_eq!(v.checked_mul_i64(1), Ok(v.clone()));
    assert_eq!(v.checked_mul_i64(-1), Ok(-&v));
    assert_eq!(&v * &e(1.0), v);
    assert_eq!(&(&v + &a) - &a, v);
    assert_eq!(&(&v + &b) - &b, v);
    assert_eq!(&(&v - &a) + &a, v);
    assert_eq!(&(&v + &v) - &v, v);
    let mut acc = v.clone();
    acc += &v;
    acc -= &v;
    assert_eq!(acc, v);
    // 2^(C−1) is one limb; v + 2^(C−1) = 2^C + 3 spans C + 1 bits.
    let one_limb = two_to(cap - 1);
    assert_eq!(v.checked_add(&one_limb), Err(PastCap::Limbs));
    assert_eq!(one_limb.checked_add(&v), Err(PastCap::Limbs));
    assert_eq!(v.checked_sub(&-&one_limb), Err(PastCap::Limbs));
    assert_eq!(v, &a + &b);
}

#[test]
#[should_panic(expected = "EXACT_MAX_LIMBS")]
fn exact_cap_closed_an_operator_one_bit_past_the_cap_panics_by_name() {
    let c = CAP_BITS;
    let v = &(two_to(c - 2) + e(1.0)) + &(two_to(c - 2) + e(2.0));
    let _ = &v + &two_to(c - 1);
}

/// A product spans the sum of its factors' bits or one less: at the edge the one that fits is
/// held and the one that does not is refused, by the result and not by the factors.
#[test]
fn exact_cap_closed_a_product_at_the_cap_is_refused_only_past_it() {
    let c = CAP_BITS;
    // (2^(C−2) + 1) × 3 = 3·2^(C−2) + 3 spans C bits; (2^(C−1) − 1) × 3 = 3·2^(C−1) − 3 spans
    // C + 1. Both pairs sum their factors' bits to C + 1.
    let fits_at_cap = two_to(c - 2) + e(1.0);
    let product = fits_at_cap.checked_mul(&e(3.0)).unwrap();
    assert_eq!(finite_of(&product), (BigInt::from(3) << (c - 2)) + 3);
    assert_eq!(finite_limbs(&product), EXACT_MAX_LIMBS);
    assert_eq!(fits_at_cap.checked_mul_i64(3), Ok(product.clone()));
    assert_eq!(e(3.0).checked_mul(&fits_at_cap), Ok(product));
    let all_ones = two_to(c - 1) - e(1.0);
    assert_eq!(all_ones.checked_mul(&e(3.0)), Err(PastCap::Limbs));
    assert_eq!(all_ones.checked_mul_i64(3), Err(PastCap::Limbs));
    assert_eq!(all_ones.checked_mul_i64(-3), Err(PastCap::Limbs));
    // An even factor moves the exponent and never the width.
    assert_eq!(all_ones.checked_mul_i64(2), Ok(&all_ones + &all_ones));
    assert_eq!(all_ones.checked_mul_i64(4), Ok(&all_ones * 4i64));
    assert_eq!(all_ones, two_to(c - 1) - e(1.0));
}

/// The shapes the cross keeps under miri, where every limb of a cap-wide operand is interpreted
/// and a product of two is quadratic in them: the two widest, the half-span one whose doubled sum
/// lands exactly at the cap, a power past the cap and the two small ones, so held and refused
/// sums and products all still occur.
#[cfg(miri)]
const MIRI_CROSS: [&str; 6] = [
    "2^(C-1) + 3",
    "2^C - 1",
    "2^(C-2) + 1",
    "2^(C+64)",
    "1",
    "2^64 + 1",
];

/// Every pairwise sum, difference and product of the edge shapes is held exactly when its result
/// spans at most the cap and refused exactly when it spans more: the operations decide by the
/// result, not by the operands' spans, and every value they land retracts. Under miri the cross
/// runs over `MIRI_CROSS` and the identities over every shape.
#[test]
fn exact_cap_closed_sums_and_products_at_the_edge_agree_with_the_oracle() {
    let shapes = edge_shapes();
    assert_eq!(shapes.len(), 11);
    let (mut held, mut refused) = (0, 0);
    for (name, v, _) in &shapes {
        assert!((v - v).is_zero(), "{name}");
        assert_eq!(v.checked_add(&-v), Ok(Exact::ZERO), "{name}");
        assert_eq!(v.checked_mul_i64(1), Ok(v.clone()), "{name}");
        assert_eq!(v.checked_mul_i64(-1), Ok(-v), "{name}");
        assert_eq!(&(v + v) - v, *v, "{name}");
    }
    #[cfg(miri)]
    let shapes: Vec<_> = shapes
        .into_iter()
        .filter(|(name, ..)| MIRI_CROSS.contains(name))
        .collect();
    let n = shapes.len();
    assert_eq!(n, if cfg!(miri) { 6 } else { 11 });
    for (an, a, oa) in &shapes {
        for (bn, b, ob) in &shapes {
            for (sign, bb, obb) in [(1, b.clone(), ob.clone()), (-1, -b, -ob)] {
                let oracle = oa + &obb;
                let got = a.checked_add(&bb);
                assert_eq!(got.is_ok(), fits(&oracle), "{an} + {sign}·({bn}): {got:?}");
                assert_eq!(a.checked_sub(&-&bb), got, "{an} - -{sign}·({bn})");
                match got {
                    Ok(sum) => {
                        held += 1;
                        assert_eq!(finite_of(&sum), oracle, "{an} + {sign}·({bn})");
                        assert_eq!(a + &bb, sum, "{an} + {sign}·({bn})");
                        assert_eq!(&sum - &bb, *a, "{an} + {sign}·({bn}) - the same");
                    }
                    Err(err) => {
                        refused += 1;
                        assert_eq!(err, PastCap::Limbs, "{an} + {sign}·({bn})");
                    }
                }
            }
            let oracle = oa * ob;
            let got = a.checked_mul(b);
            assert_eq!(got.is_ok(), fits(&oracle), "{an} × {bn}: {got:?}");
            match got {
                Ok(product) => {
                    held += 1;
                    assert_eq!(finite_of(&product), oracle, "{an} × {bn}");
                    assert_eq!(a * b, product, "{an} × {bn}");
                }
                Err(err) => {
                    refused += 1;
                    assert_eq!(err, PastCap::Limbs, "{an} × {bn}");
                }
            }
        }
    }
    assert_eq!(held + refused, n * n * 3);
    assert!(held > 0 && refused > 0, "{held} held, {refused} refused");
}
