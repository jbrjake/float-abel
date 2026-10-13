//! `Exact` with specials in the multiset and the ring on top: the group laws on the counts,
//! `Mul<i64>` on them, the monoid-algebra product with integer amounts, the documented special
//! products, and where the 127-bit width stops a sum that `Wide` still holds. Every test is named
//! `exact_ring_*` so the task verify can filter on it.

use float_abel::{Exact, PastCap, Wide};

use crate::support::{hash, rounds, Rng};

const NAN: f64 = f64::NAN;
const INF: f64 = f64::INFINITY;
const NINF: f64 = f64::NEG_INFINITY;

fn e(x: f64) -> Exact {
    Exact::from(x)
}

/// One time in sixteen a special, else a same-scale finite value.
fn draw(rng: &mut Rng) -> Exact {
    match rng.next() % 16 {
        0 => e(NAN),
        1 => e(INF),
        2 => e(NINF),
        _ => e(rng.on_grid(-20, 20)),
    }
}

/// A ring input: a special, or a 10-bit integer, so that three products and every special amount
/// stay inside the width and `i32`.
fn small(rng: &mut Rng) -> Exact {
    match rng.next() % 8 {
        0 => e(NAN),
        1 => e(INF),
        2 => e(NINF),
        _ => e(((rng.next() % 2048) as i64 - 1024) as f64),
    }
}

fn is_copy<T: Copy>() {}

#[test]
fn exact_ring_is_a_32_byte_copy_word_that_counts_specials() {
    is_copy::<Exact>();
    assert_eq!(core::mem::size_of::<Exact>(), 32);
    assert_eq!(core::mem::align_of::<Exact>(), 8);
    assert!(e(NAN).to_f64().is_nan());
    assert_eq!(e(NAN).specials(), (1, 0, 0));
    assert_eq!(e(INF).specials(), (0, 1, 0));
    assert_eq!(e(NINF).specials(), (0, 0, 1));
    assert_eq!(e(f64::from_bits(0x7FF0_0000_0000_0001)), e(NAN));
    for s in [e(NAN), e(INF), e(NINF)] {
        assert!(!s.is_finite());
        assert!(!s.is_zero());
    }
    // A NaN inserted then retracted leaves the finite sum, and on its own leaves the zero.
    let mut s = e(1.5);
    s += e(NAN);
    assert!(s.to_f64().is_nan());
    s -= e(NAN);
    assert_eq!(s, e(1.5));
    assert_eq!(s.to_f64(), 1.5);
    assert!((e(NAN) - e(NAN)).is_zero());
    // Projection: NaN first, then both infinities, then one by the sign of its count.
    assert!((e(INF) + e(NINF)).to_f64().is_nan());
    assert_eq!((e(INF) + e(INF) + e(-1e300)).to_f64(), INF);
    assert_eq!((-e(INF)).specials(), (0, -1, 0));
    assert_eq!((-e(INF)).to_f64(), NINF);
    assert_eq!((e(INF) * -3i64).to_f64(), NINF);
    assert_eq!(e(INF).try_to_f64(), Some(INF));
    assert_eq!((e(NINF) + e(0.1)).div_f64(3), NINF);
    assert!(e(NAN).div_f64(0).is_nan());
    // Order: the finite part first, then (n, p, q).
    assert!(e(NAN) < e(5e-324));
    assert!(e(INF) < e(NAN));
    assert!(e(NINF) < e(INF));
    assert!(Exact::ZERO < e(NINF));
    assert!(-e(INF) < Exact::ZERO);
    assert_eq!(hash(&(e(INF) + e(1.0) - e(1.0))), hash(&e(INF)));
    let d = format!("{:?}", e(0.1) + e(NAN) - e(INF));
    assert!(
        d.contains("NaN") && d.contains("nan 1, +inf -1, -inf 0"),
        "{d}"
    );
}

#[test]
#[allow(clippy::erasing_op)] // scaling by a literal zero is one of the cases under test
fn exact_ring_group_laws_hold_with_specials_in_the_multiset() {
    let mut rng = Rng(40);
    let mut specials = 0;
    let draws = rounds(20_000);
    for _ in 0..draws {
        let (a, b, c) = (draw(&mut rng), draw(&mut rng), draw(&mut rng));
        specials += usize::from(!a.is_finite());
        assert_eq!(a + b, b + a);
        assert_eq!((a + b) + c, a + (b + c));
        assert_eq!(a + Exact::ZERO, a);
        assert_eq!(a + (-a), Exact::ZERO);
        assert_eq!(a + b - b, a);
        assert_eq!(-(-a), a);
        assert_eq!(a * 3i64, a + a + a);
        assert_eq!(a * -1i64, -a);
        assert_eq!(a * 0i64, Exact::ZERO);
        assert_eq!((a + b) * 5i64, a * 5i64 + b * 5i64);
    }
    assert!(
        specials > draws / 25,
        "{specials} specials in {draws} draws"
    );
    // Folds are order independent, specials included.
    let xs: Vec<Exact> = (0..2_000).map(|_| draw(&mut rng)).collect();
    let forward: Exact = xs.iter().sum();
    let backward: Exact = xs.iter().rev().copied().sum();
    assert_eq!(forward, backward);
    assert!(!forward.is_finite());
    let mut back = forward;
    for x in &xs {
        back -= x;
    }
    assert!(back.is_zero());
}

#[test]
fn exact_ring_mul_is_a_commutative_ring_with_integer_special_amounts() {
    let mut rng = Rng(41);
    let one = e(1.0);
    let mut special_products = 0;
    let draws = rounds(100_000);
    for _ in 0..draws {
        let (a, b, c) = (small(&mut rng), small(&mut rng), small(&mut rng));
        special_products += usize::from(!(a * b).is_finite());
        assert_eq!(a * b, b * a);
        assert_eq!((a * b) * c, a * (b * c));
        assert_eq!(a * (b + c), a * b + a * c);
        assert_eq!((a + b) * c, a * c + b * c);
        assert_eq!(a * one, a);
        assert_eq!(a * Exact::ZERO, Exact::ZERO);
        assert_eq!(a * (-b), -(a * b));
    }
    assert!(
        special_products > draws / 10,
        "{special_products} special products in {draws}"
    );
}

#[test]
fn exact_ring_special_products_follow_the_algebra_and_a_fraction_is_the_count_cap() {
    // An exact zero annihilates, by bilinearity, not IEEE.
    assert_eq!(Exact::ZERO * e(INF), Exact::ZERO);
    assert_eq!(Exact::ZERO * e(NAN), Exact::ZERO);
    assert_eq!(Exact::product(0.0, INF), Exact::ZERO);
    assert_eq!(Exact::product(NAN, -0.0), Exact::ZERO);
    // The sign rule on two infinities, and NaN absorbing.
    assert_eq!((e(INF) * e(NINF)).specials(), (0, 0, 1));
    assert_eq!((e(INF) * e(NINF)).to_f64(), NINF);
    assert_eq!((e(NINF) * e(NINF)).specials(), (0, 1, 0));
    assert_eq!((e(NINF) * e(NINF)).to_f64(), INF);
    assert_eq!((e(INF) * e(INF)).specials(), (0, 1, 0));
    assert_eq!((e(NAN) * e(INF)).specials(), (1, 0, 0));
    assert_eq!((e(NAN) * e(2.0)).specials(), (2, 0, 0));
    assert_eq!((e(NAN) * e(NAN)).specials(), (1, 0, 0));
    // An integer weight scales a count; a negative one flips an infinity through the projection.
    assert_eq!((e(-1.0) * e(INF)).specials(), (0, -1, 0));
    assert_eq!((e(-1.0) * e(INF)).to_f64(), NINF);
    assert_ne!(e(-1.0) * e(INF), e(NINF));
    assert_eq!((e(3.0) * e(NINF)).specials(), (0, 0, 3));
    assert_eq!(e(2.0) * e(NAN), e(NAN) + e(NAN));
    // A fraction cannot be a count: that is the cap, on either side and in a row product too.
    assert_eq!(e(2.5).checked_mul(&e(NAN)), Err(PastCap::Count));
    assert_eq!(e(NAN).checked_mul(&e(2.5)), Err(PastCap::Count));
    assert_eq!(e(0.5).checked_mul(&e(INF)), Err(PastCap::Count));
    assert_eq!(e(1e300).checked_mul(&e(INF)), Err(PastCap::Count));
    // A row product counts once per row, as Wide does, whatever the finite factor's size.
    assert_eq!(Exact::product(2.5, NAN).specials(), (1, 0, 0));
    assert_eq!(Exact::product(INF, -2.5).specials(), (0, -1, 0));
    assert_eq!(Exact::product(-2.5, NINF).specials(), (0, 0, -1));
    assert_eq!(Exact::product(5e-324, NINF).specials(), (0, 0, 1));
    let mut w = Wide::ZERO;
    w.deposit_product(INF, -3.0).unwrap();
    let (n, p, q) = w.specials();
    assert_eq!(
        Exact::product(INF, -3.0).specials(),
        (n as i32, p as i32, q as i32)
    );
    // Counts are bounded: past i32, or at i32::MIN, is the cap, and the state is untouched.
    let many = e(NAN) * i64::from(i32::MAX);
    assert_eq!(many.checked_add(&e(NAN)), Err(PastCap::Count));
    assert_eq!(many.checked_mul_i64(2), Err(PastCap::Count));
    assert_eq!(
        e(NAN).checked_mul_i64(i64::from(i32::MIN)),
        Err(PastCap::Count)
    );
    assert_eq!(
        e(NAN)
            .checked_mul_i64(i64::from(i32::MIN) + 1)
            .map(|v| v.specials()),
        Ok((i32::MIN + 1, 0, 0))
    );
    assert_eq!(many.specials(), (i32::MAX, 0, 0));
}

#[test]
#[should_panic(expected = "count")]
fn exact_ring_a_fractional_special_amount_panics_by_name_on_the_operator() {
    let _ = e(2.5) * e(NAN);
}

#[test]
#[cfg_attr(miri, ignore = "the refusal rate over 10 000 pairs is the assertion")]
fn exact_ring_the_width_stops_a_wide_cancellation_that_wide_holds() {
    // 1e30 and 1e-10 are 186 bits apart with full mantissas, past the 127-bit width: Exact says
    // so by name, and Wide holds the history exactly.
    let history = [1e30, 1e-10, -1e30];
    let mut x = Exact::ZERO;
    assert_eq!(x.checked_deposit(history[0]), Ok(()));
    assert_eq!(x.checked_deposit(history[1]), Err(PastCap::Width));
    assert_eq!(x, e(1e30));
    let w: Wide = history.iter().map(|&v| Wide::from(v)).sum();
    assert_eq!(w.try_to_f64(), Some(1e-10));
    // Magnitudes over 2^±300 are the adversary. DESIGN.md § Exactness bounds measured that 77% of
    // pairwise adds leave the width: a pair fits iff its leading bits are within 74 of each other.
    // Wide holds every pair.
    let mut rng = Rng(42);
    let (mut refused, mut wide) = (0, Wide::ZERO);
    for _ in 0..10_000 {
        let (a, b) = (rng.on_grid(-300, 300), rng.on_grid(-300, 300));
        wide.deposit(a);
        wide.deposit(b);
        match e(a).checked_add(&e(b)) {
            Ok(_) => {}
            Err(cap) => {
                assert_eq!(cap, PastCap::Width);
                refused += 1;
            }
        }
    }
    assert!(
        (7_300..=8_100).contains(&refused),
        "{refused} of 10000 pairs refused"
    );
    assert!(wide.is_finite());
    // Inside the width, the same cancellation is exact.
    assert_eq!((e(1e30) + e(1e10) - e(1e30)).try_to_f64(), Some(1e10));
}
