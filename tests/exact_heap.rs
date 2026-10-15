//! The heap arm of `Exact`, DESIGN.md § Exact: a sum past the 127-bit width promotes to one
//! allocation and demotes after every operation, so every value is held exactly and the canonical
//! form is the smallest form that holds it. The two forms compare, hash and order as one type, the
//! five histories of docs/OVERFLOW.md complete in every order, a mixed-magnitude fold matches a
//! `num-bigint` oracle, and the one cap left is the limb cap, by name. Every test is named
//! `exact_heap_*` so the task verify can filter on it; `tests/exact_heap_ring.rs` holds the
//! products, the dyadic special amounts, the laws kit and the exchange.

// The tests name their operands as the algebra does.
#![allow(clippy::many_single_char_names)]

use core::cmp::Ordering;

use float_abel::{Exact, PastCap, Wide, EXACT_MAX_LIMBS};
use num_bigint::BigInt;

use crate::support::{hash, pow2, rounds, Rng};

const _: () = assert!(core::mem::size_of::<Exact>() == 32 && core::mem::align_of::<Exact>() == 8);
// Any `Wide` fits under the limb cap.
const _: () = assert!(EXACT_MAX_LIMBS >= 34);

fn e(x: f64) -> Exact {
    Exact::from(x)
}

/// `2^k` as an `Exact` for any `k` inside the exponent bound: a product of the powers of two a
/// double holds.
pub(crate) fn two_to(k: i32) -> Exact {
    let mut v = e(1.0);
    let mut left = k;
    while left > 1023 {
        v = v * e(pow2(1023));
        left -= 1023;
    }
    while left < -1022 {
        v = v * e(pow2(-1022));
        left += 1022;
    }
    v * e(pow2(left))
}

/// A finite double in units of `2^-1074`, the oracle's integer: every finite double is a whole
/// number of them, read off its bits.
pub(crate) fn units(x: f64) -> BigInt {
    let bits = x.to_bits();
    let negative = bits >> 63 != 0;
    let biased = (bits >> 52) & 0x7FF;
    let frac = bits & ((1 << 52) - 1);
    assert_ne!(biased, 0x7FF, "{x} is not finite");
    let (mant, shift) = if biased == 0 {
        (frac, 0)
    } else {
        (frac | (1 << 52), biased - 1)
    };
    let m = BigInt::from(mant) << shift;
    if negative {
        -m
    } else {
        m
    }
}

/// A `Wide`'s finite part in the oracle's units: two's complement across its 34 limbs.
pub(crate) fn wide_units(w: &Wide) -> BigInt {
    let mut bytes = Vec::with_capacity(34 * 8);
    for limb in w.limbs() {
        bytes.extend_from_slice(&limb.to_le_bytes());
    }
    BigInt::from_signed_bytes_le(&bytes)
}

/// `x`'s finite part in the oracle's units, through `Wide`, which holds every sum of doubles.
fn exact_units(x: &Exact) -> BigInt {
    wide_units(&Wide::try_from(x).expect("a sum of doubles lies inside Wide's window"))
}

#[test]
fn exact_heap_is_a_32_byte_word_on_both_forms() {
    fn is_clone<T: Clone + Send + Sync + 'static>() {}
    is_clone::<Exact>();
    assert_eq!(core::mem::size_of::<Exact>(), 32);
    assert_eq!(core::mem::align_of::<Exact>(), 8);
    // The cap: 16 products' worth of bits, and any `Wide` fits under it.
    assert_eq!(EXACT_MAX_LIMBS * 64, 16 * 2148);
    assert!(Exact::ZERO.is_compact());
    assert!(e(0.1).is_compact());
    assert!(e(f64::NAN).is_compact());
}

#[test]
fn exact_heap_promotes_past_the_width_and_demotes_after_every_operation() {
    let classic = e(0.1) + e(0.2);
    assert!(classic.is_compact());
    let wide = e(1e300) + e(1e-300);
    assert!(!wide.is_compact(), "{wide:?}");
    assert_eq!(wide.try_to_f64(), None);
    assert_eq!(wide.to_f64(), 1e300);
    assert!(wide.is_finite());
    assert!(!wide.is_zero());
    let back = &wide - &e(1e-300);
    assert!(back.is_compact());
    assert_eq!(back, e(1e300));
    assert_eq!(hash(&back), hash(&e(1e300)));
    assert_eq!(back.cmp(&e(1e300)), Ordering::Equal);
    let small = &wide - &e(1e300);
    assert!(small.is_compact());
    assert_eq!(small, e(1e-300));
    assert_eq!(small.try_to_f64(), Some(1e-300));
    // A promoted sum and its negation cancel to the compact zero, and `is_zero` sees it.
    let cancelled = &wide + &-&wide;
    assert!(cancelled.is_compact());
    assert!(cancelled.is_zero());
    assert_eq!(cancelled, Exact::ZERO);
    assert_eq!(&wide - &wide, Exact::ZERO);
    // The operands of a promotion are untouched: they are values.
    let (a, b) = (e(1e300), e(1e-300));
    assert_eq!(a.checked_add(&b).map(|v| v.is_compact()), Ok(false));
    assert_eq!(a, e(1e300));
    assert_eq!(b, e(1e-300));
    // The in-place operators promote and demote the slot itself.
    let mut slot = e(1e300);
    slot += &b;
    assert!(!slot.is_compact());
    slot -= &b;
    assert!(slot.is_compact());
    assert_eq!(slot, a);
    slot.deposit(1e-300);
    assert!(!slot.is_compact());
    slot.deposit(-1e300);
    assert_eq!(slot, b);
}

#[test]
fn exact_heap_a_value_built_through_the_heap_form_equals_the_compact_one() {
    let mut rng = Rng(50);
    for _ in 0..rounds(2_000) {
        let a = e(rng.on_grid(-20, 20));
        let big = e(rng.on_grid(200, 300));
        let raised = &a + &big;
        assert!(!raised.is_compact(), "{a:?} + {big:?}");
        let via_heap = &raised - &big;
        assert!(via_heap.is_compact());
        assert_eq!(via_heap, a);
        assert_eq!(hash(&via_heap), hash(&a));
        assert_eq!(via_heap.cmp(&a), Ordering::Equal);
        let mut acc = a.clone();
        acc += &big;
        acc -= &big;
        assert_eq!(acc, a);
        // The same multiset in two orders lands one structure.
        assert_eq!(&(&big + &a) - &big, via_heap);
        assert_eq!(&a + &big, &big + &a);
    }
}

/// The five histories of docs/OVERFLOW.md, each ending at the multiset `{small}`. On one slot
/// through `+=` they are the three updates in every arrival order, the epoch change of A formed
/// first, and D's two cancelling epochs.
#[test]
fn exact_heap_the_five_histories_read_the_small_value_in_every_order() {
    const ORDERS: [[usize; 3]; 6] = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];
    for (big, small) in [(1e12, 1e-12), (pow2(1000), pow2(-1000)), (1e300, 1e-300)] {
        let updates = [big, -big, small];
        let mut promoted = 0;
        for order in ORDERS {
            let mut slot = Exact::ZERO;
            for i in order {
                slot += e(updates[i]);
                promoted += usize::from(!slot.is_compact());
            }
            assert!(slot.is_compact(), "{big:e}, {small:e}, {order:?}");
            assert_eq!(slot, e(small), "{big:e}, {small:e}, {order:?}");
            assert_eq!(slot.try_to_f64(), Some(small));
        }
        assert!(promoted > 0, "{big:e}, {small:e}");
        // A: the epoch's change, `−big + small`, consolidated before it reaches the total.
        let change = e(-big) + e(small);
        assert!(!change.is_compact());
        assert_eq!(e(big) + &change, e(small));
        // D: the retraction lands first and the slot is the zero word again.
        let mut d = e(big);
        d += e(-big);
        assert!(d.is_zero());
        d += e(small);
        assert_eq!(d, e(small));
        // The checked twin never refuses on the way.
        let mut checked = Exact::ZERO;
        for x in updates {
            assert_eq!(checked.checked_deposit(x), Ok(()), "{x:e}");
        }
        assert_eq!(checked, e(small));
    }
}

#[test]
fn exact_heap_a_fold_over_the_mixed_band_with_retractions_matches_the_bigint_oracle() {
    let mut rng = Rng(51);
    let xs: Vec<f64> = (0..rounds(2_000)).map(|_| rng.on_grid(-300, 300)).collect();
    let mut acc = Exact::ZERO;
    let mut oracle = BigInt::from(0);
    let (mut heap_steps, mut compact_steps) = (0, 0);
    for &x in &xs {
        acc += e(x);
        oracle += units(x);
        assert_eq!(exact_units(&acc), oracle, "{x:e}");
        if acc.is_compact() {
            compact_steps += 1;
        } else {
            heap_steps += 1;
        }
    }
    assert!(
        heap_steps > compact_steps,
        "{heap_steps} heap, {compact_steps} compact"
    );
    assert!(compact_steps >= 1);
    // Retract every third in reverse, then the rest in a shuffled order: the oracle follows at
    // every step and the slot ends as the zero word.
    let (thirds, rest): (Vec<usize>, Vec<usize>) = (0..xs.len()).partition(|i| i % 3 == 0);
    for &i in thirds.iter().rev() {
        acc -= e(xs[i]);
        oracle -= units(xs[i]);
        assert_eq!(exact_units(&acc), oracle, "{i}");
    }
    let mut rest = rest;
    for i in (1..rest.len()).rev() {
        rest.swap(i, (rng.next() % (i as u64 + 1)) as usize);
    }
    for &i in &rest {
        acc -= e(xs[i]);
        oracle -= units(xs[i]);
        assert_eq!(exact_units(&acc), oracle, "{i}");
    }
    assert!(acc.is_zero());
    assert!(acc.is_compact());
    assert_eq!(acc, Exact::ZERO);
    assert_eq!(oracle, BigInt::from(0));
    // The whole multiset in one shot, through `Sum`, by value and by reference.
    let by_value: Exact = xs.iter().map(|&x| e(x)).sum();
    let by_ref: Exact = xs.iter().map(|&x| e(x)).collect::<Vec<_>>().iter().sum();
    let total: BigInt = xs.iter().map(|&x| units(x)).sum();
    assert_eq!(exact_units(&by_value), total);
    assert_eq!(by_value, by_ref);
}

#[test]
fn exact_heap_order_is_numeric_and_total_across_forms() {
    let (big, tiny) = (e(1e300), e(1e-300));
    let up = &big + &tiny;
    let down = &big - &tiny;
    assert!(!up.is_compact() && !down.is_compact());
    assert!(down < big);
    assert!(big < up);
    assert!(up > down);
    assert!(-&up < -&big);
    assert!(-&big < -&down);
    assert!(-&down < Exact::ZERO);
    assert!(Exact::ZERO < down);
    assert_eq!(up.cmp(&up.clone()), Ordering::Equal);
    assert_eq!(up.partial_cmp(&down), Some(Ordering::Greater));
    assert_ne!(up, down);
    assert_ne!(up, big);
    // A sorted mix of forms, finite and not, is monotone; ties are structural equality with one
    // hash, and the order agrees with the sign of the difference.
    let mut rng = Rng(52);
    let mut xs: Vec<Exact> = (0..300)
        .map(|i| match i % 4 {
            0 => e(rng.on_grid(-300, 300)) + e(rng.on_grid(-300, 300)),
            1 => e(rng.finite()),
            2 => e(rng.on_grid(-300, 300)) + e(rng.on_grid(-300, 300)) + e(f64::NAN),
            _ => e(rng.on_grid(-10, 10)),
        })
        .collect();
    xs.extend([up.clone(), down.clone(), big.clone(), up.clone(), -&up]);
    xs.sort();
    for pair in xs.windows(2) {
        assert_ne!(pair[0].cmp(&pair[1]), Ordering::Greater);
        if pair[0] == pair[1] {
            assert_eq!(hash(&pair[0]), hash(&pair[1]));
            assert_eq!(pair[0].cmp(&pair[1]), Ordering::Equal);
        }
        let d = &pair[1] - &pair[0];
        assert!(
            d.is_zero() || d > Exact::ZERO,
            "{:?} vs {:?}",
            pair[0],
            pair[1]
        );
    }
    let ups = xs.iter().filter(|x| **x == up).count();
    assert_eq!(ups, 2);
}

/// A value one squaring under the cap: `2^53 − 1` squared nine times spans 53 · 2⁹ = 27,136 bits
/// of the cap's 34,368.
fn near_cap() -> Exact {
    (0..9).fold(e(9_007_199_254_740_991.0), |x, _| &x * &x)
}

#[test]
#[should_panic(expected = "limb")]
fn exact_heap_the_limb_cap_panics_by_name_one_step_past() {
    let x = near_cap();
    let _ = &x * &x;
}

#[test]
#[should_panic(expected = "limb")]
fn exact_heap_a_sum_past_the_limb_cap_panics_by_name() {
    let _ = two_to(34000) + e(1e-300);
}

#[test]
fn exact_heap_the_checked_api_returns_the_limb_cap_and_leaves_the_state() {
    let x = near_cap();
    assert!(!x.is_compact());
    let before = x.clone();
    assert_eq!(x.checked_mul(&x), Err(PastCap::Limbs));
    assert_eq!(x, before);
    // Sums: 2^34000 with 1e-300 spans past the cap; with 2^-340 it spans 34,341 bits, inside it,
    // and a scalar of 63 bits then carries that past the cap where a scalar of 2 bits does not.
    let top = two_to(34000);
    let mut slot = top.clone();
    assert_eq!(slot.checked_deposit(1e-300), Err(PastCap::Limbs));
    assert_eq!(slot, top);
    assert_eq!(top.checked_add(&e(1e-300)), Err(PastCap::Limbs));
    assert_eq!(e(1e-300).checked_add(&top), Err(PastCap::Limbs));
    assert_eq!(top.checked_sub(&e(1e-300)), Err(PastCap::Limbs));
    let fits = top
        .checked_add(&two_to(-340))
        .expect("34,341 bits fit the cap");
    assert!(!fits.is_compact());
    assert_eq!(fits.checked_mul_i64(i64::MAX), Err(PastCap::Limbs));
    assert_eq!(fits.checked_mul_i64(3).map(|v| v.is_compact()), Ok(false));
    assert_eq!(&fits - &two_to(-340), top);
    let (up, down) = (two_to(32768), two_to(-32768));
    assert_eq!(up.checked_add(&down), Err(PastCap::Limbs));
    assert_eq!(up.checked_sub(&down), Err(PastCap::Limbs));
    let half = two_to(16384) + two_to(-16384);
    assert!(!half.is_compact());
    assert_eq!(&half - &two_to(-16384), two_to(16384));
    // The exponent bound still stands beside the limb cap.
    let at_bound = (0..30).fold(e(2.0), |x, _| &x * &x);
    assert_eq!(at_bound.checked_mul(&e(2.0)), Err(PastCap::Exponent));
    assert!(PastCap::Limbs.to_string().contains("limb"));
    let _: &dyn std::error::Error = &PastCap::Limbs;
}

#[test]
fn exact_heap_reads_round_from_the_heap_form_as_wide_does() {
    let mut rng = Rng(53);
    let mut promoted = 0;
    for _ in 0..rounds(5_000) {
        let (a, b, c) = (
            rng.on_grid(-300, 300),
            rng.on_grid(-300, 300),
            rng.on_grid(-300, 300),
        );
        let x = e(a) + e(b) + e(c);
        promoted += usize::from(!x.is_compact());
        let w = Wide::from(a) + Wide::from(b) + Wide::from(c);
        assert_eq!(
            x.to_f64().to_bits(),
            w.to_f64().to_bits(),
            "{a:e} + {b:e} + {c:e}"
        );
        assert_eq!(x.try_to_f64(), w.try_to_f64());
        assert_eq!(x.to_f32().to_bits(), w.to_f32().to_bits());
        let n = (rng.next() >> 11).max(1);
        assert_eq!(x.div_f64(n).to_bits(), w.div_f64(n).to_bits(), "/ {n}");
        assert_eq!(x.div_f64(0).to_bits(), w.div_f64(0).to_bits());
        // The conversions agree both ways, and a `Wide` past the width comes in as a heap value.
        assert_eq!(Exact::try_from(&w), Ok(x.clone()));
        assert_eq!(Wide::try_from(&x), Ok(w));
    }
    assert!(promoted > rounds(5_000) / 2, "{promoted}");
    assert_eq!((e(1e300) + e(1e-300) - e(1e300)).try_to_f64(), Some(1e-300));
    let s = e(f64::MAX) + e(f64::MAX) + e(5e-324);
    assert!(!s.is_compact());
    assert_eq!(s.to_f64(), f64::INFINITY);
    assert_eq!(s.to_f32(), f32::INFINITY);
    assert_eq!(s.div_f64(2).to_bits(), f64::MAX.to_bits());
    assert_eq!(s.try_to_f64(), None);
    // A `Fixed256` state wider than 127 bits comes in as a heap value too.
    let mut f = float_abel::Fixed256::ZERO;
    f.try_deposit(pow2(90)).unwrap();
    f.try_deposit(pow2(-44)).unwrap();
    let x = Exact::try_from(&f).unwrap();
    assert!(!x.is_compact());
    assert_eq!(x, e(pow2(90)) + e(pow2(-44)));
    assert_eq!(x.try_to_f64(), None);
}

#[test]
fn exact_heap_counts_ride_through_promotion_and_a_count_past_i32_promotes() {
    let nan = e(f64::NAN);
    let mut s = &nan + &e(1e300);
    s += e(1e-300);
    assert!(!s.is_compact());
    assert!(s.to_f64().is_nan());
    assert!(!s.is_finite());
    assert_eq!(s.specials(), Some((1, 0, 0)));
    s -= &nan;
    assert!(s.is_finite());
    assert_eq!(s.specials(), Some((0, 0, 0)));
    assert_eq!(s.to_f64(), 1e300);
    s -= e(1e-300);
    assert!(s.is_compact());
    assert_eq!(s, e(1e300));
    // A count past `i32` promotes rather than capping, and comes back down.
    let many = &nan * i64::from(i32::MAX);
    assert!(many.is_compact());
    assert_eq!(many.specials(), Some((i64::from(i32::MAX), 0, 0)));
    let more = &many + &nan;
    assert!(!more.is_compact());
    assert_eq!(more.specials(), Some((1 << 31, 0, 0)));
    assert!(more.to_f64().is_nan());
    assert_eq!(&more - &nan, many);
    assert!((&more - &nan).is_compact());
    assert_eq!(
        more.checked_mul_i64(2).map(|v| v.specials()),
        Ok(Some((1 << 32, 0, 0)))
    );
    assert_eq!(many.checked_add(&nan), Ok(more.clone()));
    // `i32::MIN` is not a compact count, because negation must stay total: it is a heap amount.
    let inf = e(f64::INFINITY);
    let min = &inf * i64::from(i32::MIN);
    assert!(!min.is_compact());
    assert_eq!(min.specials(), Some((0, i64::from(i32::MIN), 0)));
    assert_eq!(min.to_f64(), f64::NEG_INFINITY);
    let neg_min = -&min;
    assert!(!neg_min.is_compact());
    assert_eq!(neg_min.specials(), Some((0, 1 << 31, 0)));
    assert_eq!(neg_min.to_f64(), f64::INFINITY);
    assert_eq!(&neg_min - &inf, &inf * i64::from(i32::MAX));
    assert!((&neg_min - &inf).is_compact());
    assert_eq!(
        nan.checked_mul_i64(i64::MIN).map(|v| v.specials()),
        Ok(Some((i64::MIN, 0, 0)))
    );
    // `Wide`'s `i64` counts come in and go out.
    let w = Wide::from_parts([0; 34], [1 << 40, 0, -(1 << 40)]);
    let x = Exact::try_from(&w).unwrap();
    assert!(!x.is_compact());
    assert_eq!(x.specials(), Some((1 << 40, 0, -(1 << 40))));
    assert!(x.to_f64().is_nan());
    assert_eq!(Wide::try_from(&x), Ok(w));
    assert_eq!(&x - &x, Exact::ZERO);
}

#[test]
#[allow(clippy::erasing_op)] // scaling by a literal zero is one of the cases under test
fn exact_heap_neg_and_integer_scaling_act_on_the_heap_form() {
    let x = e(1e300) + e(1e-300);
    let n = -&x;
    assert!(!n.is_compact());
    assert_eq!(&n + &x, Exact::ZERO);
    assert_eq!(-(-&x), x);
    assert_eq!(n.to_f64(), -1e300);
    let mut m = x.clone();
    m = -m;
    assert_eq!(m, n);
    assert_eq!(hash(&m), hash(&n));
    assert_eq!(&x * 3i64, &(&x + &x) + &x);
    assert_eq!(&x * -1i64, n);
    assert_eq!(&x * 0i64, Exact::ZERO);
    assert!((&x * 0i64).is_compact());
    assert_eq!(x.clone() * 2isize, &x + &x);
    assert_eq!(&(&x * 6i64) * 7i64, &x * 42i64);
    assert_eq!(&(&x * i64::MAX) * i64::MIN, &(&x * i64::MIN) * i64::MAX);
    // Scaling a heap value can land it compact: 2^127 + 1 is 128 bits, and halving it is not
    // possible by an integer, so take `(2^126 + 1) · 2 = 2^127 + 2`, 127 bits.
    let y = two_to(126) + e(1.0);
    assert!(y.is_compact());
    let z = &y + &two_to(126);
    assert!(!z.is_compact(), "{z:?}");
    assert_eq!(&z - &two_to(126), y);
    assert_eq!(&(&y * 2i64) - &y, y);
    assert!((&y * 2i64).is_compact());
}

/// Under miri, the leak checker fails this test if any block outlives its word.
#[test]
fn exact_heap_promote_clone_drop_and_demote_leak_nothing() {
    let mut rng = Rng(54);
    for _ in 0..rounds(300) {
        let a = e(rng.on_grid(-300, 300));
        let b = e(rng.on_grid(-300, 300));
        let s = &a + &b;
        let t = s.clone();
        let u = t.clone();
        drop(t);
        assert_eq!(&u - &b, a);
        let mut w = s.clone();
        w += &a;
        w -= &a;
        assert_eq!(w, s);
        w = -w;
        w.clone_from(&s);
        assert_eq!(w, s);
        let mut z = Exact::ZERO;
        z.clone_from(&s);
        assert_eq!(z, s);
        let old = core::mem::replace(&mut z, Exact::ZERO);
        assert_eq!(old, s);
        let mut slot = s.clone();
        slot *= 1i64;
        assert_eq!(slot, s);
        let v: Vec<Exact> = (0..8).map(|_| s.clone()).collect();
        let total: Exact = v.iter().sum();
        assert_eq!(total, &s * 8i64);
    }
}

#[test]
fn exact_heap_debug_names_the_form_and_the_value() {
    let x = e(1e300) + e(1e-300);
    let d = format!("{x:?}");
    assert!(
        d.starts_with("Exact(≈1e300; nan 0, +inf 0, -inf 0; heap"),
        "{d}"
    );
    assert!(d.contains("limbs"), "{d}");
    assert!(d.ends_with("2^-1049)") || d.contains("· 2^"), "{d}");
    // A special projection is exact, so it carries no `≈`; the dyadic amount does.
    let n = format!("{:?}", &(e(2.5) * e(f64::NAN)) + &x);
    assert!(
        n.starts_with("Exact(NaN; nan ≈2.5e0, +inf 0, -inf 0; heap"),
        "{n}"
    );
    // The compact form's rendering is unchanged.
    assert_eq!(
        format!("{:?}", Exact::ZERO),
        "Exact(0e0; nan 0, +inf 0, -inf 0; 0x0:0x0 · 2^0)"
    );
}
