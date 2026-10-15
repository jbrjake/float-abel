//! `Exact` through its public surface: the operators by value and by reference, their checked
//! twins, the named cap, canonical equality, order, and the reads against IEEE, `Wide` and
//! `Fixed256`, on finite inputs. Every test is named `exact_cap_*` so the task verify can filter
//! on it; `tests/exact_ring.rs` holds the specials and the ring, and `tests/noalloc` the
//! no-feature build.

use core::cmp::Ordering;
use float_abel::{Exact, Fixed256, PastCap, Wide, EXACT_MAX_EXP};

use crate::support::{hash, pow2, rounds, Rng};

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
