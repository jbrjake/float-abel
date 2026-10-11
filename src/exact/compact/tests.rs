//! The compact tier against its oracles: IEEE `a + b`, `a * b`, `a.mul_add(b, c)` and `a / n`
//! are each the correctly rounded value of the exact two-term expression; `Wide` holds the same
//! sum exactly; and `f64`'s own `partial_cmp` orders finite doubles. Every test is named
//! `compact_*` so the task verify can filter on it.

#![allow(
    clippy::float_cmp,
    clippy::cast_precision_loss,
    clippy::unreadable_literal
)]

use core::cmp::Ordering;
use std::vec::Vec;

use super::{Compact, Overflow};
use crate::exact::testing::{add, c, hash, mul, pow2, product, widest, Rng};
use crate::exact::Exact;
use crate::Wide;

const _: () = assert!(core::mem::size_of::<Exact>() == 24);

#[test]
fn compact_exact_is_24_bytes_and_round_trips_its_storage_form() {
    assert_eq!(core::mem::size_of::<Exact>(), 24);
    assert_eq!(
        core::mem::size_of::<Compact>(),
        32,
        "the working form, not the storage"
    );
    assert_eq!(Exact::ZERO.compact(), Compact::ZERO);
    assert_eq!(Exact::from_compact(Compact::ZERO), Exact::ZERO);
    let mut rng = Rng(20);
    for _ in 0..10_000 {
        let v = add(c(rng.on_grid(-20, 20)), c(rng.on_grid(-20, 20)));
        let e = Exact::from_compact(v);
        assert_eq!(e.compact(), v);
        assert_eq!(e.clone(), e);
        assert_eq!(hash(&e), hash(&Exact::from_compact(v)));
        let (lo, hi, exp) = v.parts();
        assert_eq!(Compact::from_parts(lo, hi, exp), v);
    }
    // The storage form is the value's two's complement: -1 is all ones at exponent 0.
    assert_eq!(c(-1.0).parts(), (u64::MAX, u64::MAX, 0));
    assert_eq!(c(1.0).parts(), (1, 0, 0));
    assert_eq!(c(-3.0 * pow2(-1)).parts(), (u64::MAX - 2, u64::MAX, -1));
    assert_eq!(c(pow2(70)).parts(), (1, 0, 70));
}

#[test]
fn compact_from_f64_is_exact_on_finite_doubles_and_needs_wide_on_specials() {
    let mut rng = Rng(21);
    for _ in 0..50_000 {
        let x = rng.finite();
        let v = c(x);
        assert_eq!(v.read(), (x, false), "{x:e}");
        assert_eq!(v.is_zero(), x == 0.0, "{x:e}");
        let (lo, _, exp) = v.parts();
        assert!(lo & 1 == 1 || (v.is_zero() && exp == 0), "{x:e}: canonical");
    }
    for x in [
        5e-324,
        -5e-324,
        f64::from_bits((1 << 52) - 1),
        f64::MIN_POSITIVE,
        f64::MAX,
        -f64::MAX,
        1e-300,
        1e300,
        0.1,
        1.0 + f64::EPSILON,
    ] {
        assert_eq!(c(x).read(), (x, false), "{x:e}");
    }
    assert_eq!(c(5e-324).parts(), (1, 0, -1074));
    assert_eq!(c(f64::MAX).parts(), ((1 << 53) - 1, 0, 971));
    // Both zeros are the one zero, which reads +0.0.
    assert_eq!(c(0.0), Compact::ZERO);
    assert_eq!(c(-0.0), Compact::ZERO);
    assert!(c(-0.0).is_zero());
    assert_eq!(c(-0.0).read().0.to_bits(), 0.0f64.to_bits());
    assert_eq!(Compact::ZERO.parts(), (0, 0, 0));
    // A special is the heap tier's, at construction.
    assert_eq!(Compact::from_f64(f64::NAN), Err(Overflow::Special));
    assert_eq!(Compact::from_f64(-f64::NAN), Err(Overflow::Special));
    assert_eq!(Compact::from_f64(f64::INFINITY), Err(Overflow::Special));
    assert_eq!(Compact::from_f64(f64::NEG_INFINITY), Err(Overflow::Special));
    assert_eq!(Compact::from_product(f64::NAN, 0.0), Err(Overflow::Special));
    assert_eq!(
        Compact::from_product(2.0, f64::INFINITY),
        Err(Overflow::Special)
    );
}

#[test]
fn compact_add_detects_promotion_at_the_127_bit_span() {
    // The classic fits; a span of 2000 bits does not.
    assert!(c(0.1).checked_add(c(0.2)).is_ok());
    assert_eq!(c(1e300).checked_add(c(1e-300)), Err(Overflow::NeedsWide));
    assert_eq!(c(1e-300).checked_add(c(1e300)), Err(Overflow::NeedsWide));
    // The edge: a raised operand may occupy 127 bits, not 128.
    let one_plus = add(c(1.0), c(pow2(126)));
    assert_eq!(one_plus.parts(), (1, 1 << 62, 0));
    assert_eq!(one_plus.read(), (pow2(126), true));
    assert_eq!(c(1.0).checked_add(c(pow2(127))), Err(Overflow::NeedsWide));
    assert_eq!(c(pow2(127)).checked_add(c(1.0)), Err(Overflow::NeedsWide));
    assert_eq!(c(-1.0).checked_add(c(pow2(127))), Err(Overflow::NeedsWide));
    assert!(c(5e-324).checked_add(c(pow2(-948))).is_ok());
    assert_eq!(
        c(5e-324).checked_add(c(pow2(-947))),
        Err(Overflow::NeedsWide)
    );
    // A mantissa of 53 bits raised by 74 still fits; by 75 it does not.
    assert!(c(1.0).checked_add(c(f64::MAX * pow2(-897))).is_ok());
    assert_eq!(
        c(1.0).checked_add(c(f64::MAX * pow2(-896))),
        Err(Overflow::NeedsWide)
    );
    // The sum itself may carry into bit 127, and then it is refused, on either sign.
    let w = widest();
    assert_eq!(w.checked_add(c(1.0)), Err(Overflow::NeedsWide));
    assert_eq!((-w).checked_add(c(-1.0)), Err(Overflow::NeedsWide));
    assert_eq!(c(1.0).checked_add(w), Err(Overflow::NeedsWide));
    // The check is on the aligned frame: 2^127 is refused as (2^127 - 1) + 1 at exponent 0 and
    // computed as (2^127 - 2) + 2 at exponent 1, where it is 2^126 + 2^126. The heap tier's
    // demotion brings the first route back to the one-bit form the second lands on.
    let two127_minus_2 = add(w, c(-1.0));
    assert_eq!(two127_minus_2.parts(), (u64::MAX, (1 << 62) - 1, 1));
    assert_eq!(add(two127_minus_2, c(2.0)).parts(), (1, 0, 127));
    assert_eq!(add(two127_minus_2, c(1.0)), w);
    assert_eq!(add(c(pow2(126)), c(pow2(126))).parts(), (1, 0, 127));
    assert_eq!(
        add(c(-pow2(126)), c(-pow2(126))).parts(),
        (u64::MAX, u64::MAX, 127)
    );
    // A refusal is symmetric and leaves both operands as they were, being values.
    let mut rng = Rng(22);
    let (mut refused, mut accepted) = (0, 0);
    for _ in 0..20_000 {
        let (a, b) = (c(rng.finite()), c(rng.finite()));
        let ab = a.checked_add(b);
        assert_eq!(ab, b.checked_add(a), "{a:?} + {b:?}");
        assert_eq!(ab.is_err(), (-a).checked_add(-b).is_err(), "{a:?} + {b:?}");
        if ab.is_err() {
            refused += 1;
        } else {
            accepted += 1;
        }
    }
    assert!(
        refused > 10_000 && accepted > 500,
        "{refused} refused, {accepted} accepted"
    );
}

#[test]
#[allow(clippy::erasing_op)] // scaling by a literal zero is one of the cases under test
fn compact_group_laws_hold_on_same_scale_inputs() {
    let mut rng = Rng(23);
    let by = |k: i64| Compact::from_i64(k);
    for _ in 0..20_000 {
        let (a, b, cc) = (
            c(rng.on_grid(-20, 20)),
            c(rng.on_grid(-20, 20)),
            c(rng.on_grid(-20, 20)),
        );
        assert_eq!(add(a, b), add(b, a));
        assert_eq!(add(add(a, b), cc), add(a, add(b, cc)));
        assert_eq!(add(a, Compact::ZERO), a);
        assert_eq!(add(Compact::ZERO, a), a);
        assert_eq!(add(a, -a), Compact::ZERO);
        assert_eq!(add(add(a, b), -b), a);
        assert_eq!(-(-a), a);
        // Integer scaling is repeated addition.
        assert_eq!(mul(a, by(3)), add(add(a, a), a));
        assert_eq!(mul(a, by(-1)), -a);
        assert_eq!(mul(a, by(0)), Compact::ZERO);
        assert_eq!(mul(a, by(1)), a);
        assert_eq!(mul(add(a, b), by(5)), add(mul(a, by(5)), mul(b, by(5))));
        // The ring laws, with 30-bit integer factors so that three products stay inside the
        // tier: 53 + 30 + 30 bits.
        let (p, q) = (
            by((rng.next() >> 34) as i64 - (1 << 29)),
            by((rng.next() >> 34) as i64 - (1 << 29)),
        );
        assert_eq!(mul(a, b), mul(b, a));
        assert_eq!(mul(mul(a, p), q), mul(a, mul(p, q)));
        assert_eq!(mul(a, add(p, q)), add(mul(a, p), mul(a, q)));
        assert_eq!(mul(add(a, b), p), add(mul(a, p), mul(b, p)));
    }
    assert_eq!(-Compact::ZERO, Compact::ZERO);
}

#[test]
fn compact_a_plus_b_minus_b_is_a_on_a_million_random_pairs() {
    let mut rng = Rng(24);
    for _ in 0..1_000_000 {
        let (a, b) = (c(rng.on_grid(-20, 20)), c(rng.on_grid(-20, 20)));
        assert_eq!(add(add(a, b), -b), a, "{a:?} + {b:?} - {b:?}");
    }
}

#[test]
fn compact_canonical_form_makes_structural_eq_and_hash_numeric() {
    // Two routes to one value are one bit pattern.
    assert_eq!(add(c(0.5), c(0.25)), c(0.75));
    assert_eq!(hash(&add(c(0.5), c(0.25))), hash(&c(0.75)));
    assert_eq!(add(c(3.0), c(-1.0)), add(c(1.0), c(1.0)));
    assert_eq!(add(c(1.0), c(1.0)).parts(), (1, 0, 1));
    assert_eq!(add(c(1.5), c(-1.5)), Compact::ZERO);
    assert_eq!(add(c(1.5), c(-1.5)).parts(), (0, 0, 0));
    assert_eq!(add(c(pow2(100)), c(-pow2(100))).parts(), (0, 0, 0));
    // Integers canonicalise like doubles: trailing zeros are exponent.
    assert_eq!(Compact::from_i64(8).parts(), (1, 0, 3));
    assert_eq!(
        Compact::from_i64(6),
        mul(Compact::from_i64(3), Compact::from_i64(2))
    );
    assert_eq!(
        Compact::from_i64(i64::MIN).parts(),
        (u64::MAX, u64::MAX, 63)
    );
    assert_eq!(Compact::from_i64(0), Compact::ZERO);
    assert_eq!(Compact::from_i64(12), c(12.0));
    assert_eq!(hash(&Compact::from_i64(12)), hash(&c(12.0)));
    // Every sum of random terms is odd or the zero form.
    let mut rng = Rng(25);
    for _ in 0..50_000 {
        let v = add(c(rng.on_grid(-30, 30)), c(rng.on_grid(-30, 30)));
        let (lo, _, exp) = v.parts();
        assert!(lo & 1 == 1 || (v.is_zero() && exp == 0), "{v:?}");
        let p = mul(
            c(rng.on_grid(-30, 30)),
            Compact::from_i64(rng.next() as i64 >> 20),
        );
        let (lo, _, exp) = p.parts();
        assert!(lo & 1 == 1 || (p.is_zero() && exp == 0), "{p:?}");
    }
}

#[test]
fn compact_order_is_numeric_and_total() {
    let mut rng = Rng(26);
    for _ in 0..100_000 {
        let (a, b) = (rng.finite(), rng.finite());
        assert_eq!(
            c(a).cmp(&c(b)),
            a.partial_cmp(&b).unwrap(),
            "{a:e} vs {b:e}"
        );
        assert_eq!(c(a).partial_cmp(&c(b)), Some(c(a).cmp(&c(b))));
    }
    assert!(c(-5e-324) < Compact::ZERO);
    assert!(Compact::ZERO < c(5e-324));
    assert!(c(-f64::MAX) < c(5e-324));
    assert!(c(-f64::MAX) < c(-5e-324));
    assert_eq!(Compact::ZERO.cmp(&Compact::ZERO), Ordering::Equal);
    assert_eq!(c(0.0).cmp(&c(-0.0)), Ordering::Equal);
    // Values no double holds: a 127-bit mantissa against a one-bit one at the same height, in
    // both alignment directions, and against the next value up.
    let big = add(c(1.0), c(pow2(126)));
    assert!(big > c(pow2(126)));
    assert!(c(pow2(126)) < big);
    assert!(big < add(c(3.0), c(pow2(126))));
    assert!(big < c(pow2(127)));
    assert!(-big < -c(pow2(126)));
    assert!(-c(pow2(126)) > -big);
    assert!(-big > -c(pow2(127)));
    assert!(widest() < c(pow2(127)));
    assert!(widest() > add(widest(), c(-2.0)));
    // A tie on the leading bit with a long shift either way.
    let tiny = add(c(5e-324), c(pow2(-948)));
    assert!(tiny > c(pow2(-948)));
    assert!(c(pow2(-948)) < tiny);
    // Total: a sort of mixed values is consistent with cmp, and equal values are structurally
    // equal.
    let mut xs: Vec<Compact> = (0..500)
        .map(|i| {
            if i % 3 == 0 {
                add(c(rng.on_grid(-30, 30)), c(rng.on_grid(-30, 30)))
            } else {
                c(rng.finite())
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
fn compact_reads_round_like_ieee_and_agree_with_wide() {
    let mut rng = Rng(27);
    let mut from_finite = 0;
    for i in 0..100_000 {
        let (a, b) = if i % 2 == 0 {
            (rng.finite(), rng.finite())
        } else {
            (rng.on_grid(-1000, 1000), rng.on_grid(-1000, 1000))
        };
        let Ok(sum) = c(a).checked_add(c(b)) else {
            continue;
        };
        from_finite += usize::from(i % 2 == 0);
        let ieee = a + b;
        let (value, inexact) = sum.read();
        assert_eq!(value.to_bits(), ieee.to_bits(), "{a:e} + {b:e}");
        if ieee.is_finite() {
            assert_eq!(!inexact, sum == c(ieee), "{a:e} + {b:e}");
        } else {
            assert!(inexact, "{a:e} + {b:e} overflowed a double");
        }
        let wide = Wide::from(a) + Wide::from(b);
        assert_eq!(
            sum.read().0.to_bits(),
            wide.to_f64().to_bits(),
            "{a:e} + {b:e}: Wide"
        );
        assert_eq!(
            sum.read().1,
            wide.try_to_f64().is_none(),
            "{a:e} + {b:e}: Wide"
        );
        let n = (rng.next() >> 11).max(1); // < 2^53, so `n as f64` is exact
        assert_eq!(
            sum.div_f64(n).to_bits(),
            wide.div_f64(n).to_bits(),
            "{a:e} + {b:e} / {n}"
        );
    }
    assert!(from_finite > 1_000, "{from_finite} finite pairs fit");
    // Products: the exact 106-bit product reads as IEEE's multiply, and plus a term as its fma.
    let mut fmas = 0;
    for _ in 0..50_000 {
        let (a, b) = (rng.on_grid(-480, 480), rng.on_grid(-480, 480));
        let cc = rng.on_grid(-20, 20);
        let p = product(a, b);
        assert_eq!(p.read().0.to_bits(), (a * b).to_bits(), "{a:e} * {b:e}");
        assert_eq!(p, mul(c(a), c(b)), "{a:e} * {b:e}");
        let mut w = Wide::ZERO;
        w.deposit_product(a, b).unwrap();
        assert_eq!(
            p.read().0.to_bits(),
            w.to_f64().to_bits(),
            "{a:e} * {b:e}: Wide"
        );
        if let Ok(fma) = p.checked_add(c(cc)) {
            fmas += 1;
            assert_eq!(
                fma.read().0.to_bits(),
                a.mul_add(b, cc).to_bits(),
                "{a:e} * {b:e} + {cc:e}"
            );
        }
    }
    assert!(fmas > 500, "{fmas} products fit beside the third term");
    // Division: the quotient of a single double is IEEE's.
    for _ in 0..50_000 {
        let x = rng.finite();
        let n = (rng.next() >> 11).max(1);
        assert_eq!(
            c(x).div_f64(n).to_bits(),
            (x / n as f64).to_bits(),
            "{x:e} / {n}"
        );
    }
    // The classic, and a mean of ten exact tenths.
    assert_eq!(add(c(0.1), c(0.2)).read(), (0.1 + 0.2, true));
    let tenths = (0..10).fold(Compact::ZERO, |s, _| add(s, c(0.1)));
    assert_eq!(tenths.read(), (1.0, true));
    assert_eq!(tenths.div_f64(10), 0.1);
    // Division by zero follows IEEE.
    assert!(Compact::ZERO.div_f64(0).is_nan());
    assert_eq!(c(1.0).div_f64(0), f64::INFINITY);
    assert_eq!(c(-1.0).div_f64(0), f64::NEG_INFINITY);
    assert_eq!(Compact::ZERO.div_f64(7).to_bits(), 0.0f64.to_bits());
}

#[test]
fn compact_reads_round_at_both_ends_of_the_double_range() {
    // Overflow to ∞, the tie at MAX, exact subnormals, and a state past the range that comes
    // back inside it.
    assert_eq!(add(c(f64::MAX), c(f64::MAX)).read(), (f64::INFINITY, true));
    assert_eq!(
        add(c(-f64::MAX), c(-f64::MAX)).read(),
        (f64::NEG_INFINITY, true)
    );
    assert_eq!(add(c(f64::MAX), c(pow2(970))).read().0, f64::INFINITY);
    assert_eq!(add(c(f64::MAX), c(pow2(969))).read(), (f64::MAX, true));
    let two_1024 = add(c(pow2(1023)), c(pow2(1023)));
    assert_eq!(two_1024.parts(), (1, 0, 1024));
    assert_eq!(two_1024.read(), (f64::INFINITY, true));
    assert_eq!(add(two_1024, c(-pow2(1023))).read(), (pow2(1023), false));
    assert_eq!(add(c(f64::MAX), c(f64::MAX)).div_f64(2), f64::MAX);
    assert_eq!(add(c(5e-324), c(5e-324)).read(), (f64::from_bits(2), false));
    assert_eq!(
        add(c(f64::MIN_POSITIVE), c(-5e-324)).read(),
        (f64::from_bits((1 << 52) - 1), false)
    );
    assert_eq!(c(5e-324).div_f64(2).to_bits(), 0.0f64.to_bits());
    assert_eq!(c(-5e-324).div_f64(2).to_bits(), (-0.0f64).to_bits());
    // A product below the smallest subnormal is exact here, and rounds to zero only on read.
    let under = product(pow2(-600), pow2(-600));
    assert_eq!(under.parts(), (1, 0, -1200));
    assert_eq!(under.read(), (0.0, true));
    assert_eq!((-under).read().0.to_bits(), (-0.0f64).to_bits());
}

#[test]
fn compact_mul_detects_overflow_and_negation_is_total_at_the_widest_mantissa() {
    let w = widest();
    assert_eq!((-w).parts(), (1, 1 << 63, 0));
    assert_eq!(-(-w), w);
    assert_eq!(add(w, -w), Compact::ZERO);
    // A power of two scales the exponent alone and never overflows; an odd factor widens the
    // mantissa and can.
    assert_eq!(
        mul(w, Compact::from_i64(2)).parts(),
        (u64::MAX, u64::MAX >> 1, 1)
    );
    assert_eq!(
        w.checked_mul(Compact::from_i64(3)),
        Err(Overflow::NeedsWide)
    );
    assert_eq!(
        w.checked_mul(Compact::from_i64(-3)),
        Err(Overflow::NeedsWide)
    );
    assert_eq!(
        (-w).checked_mul(Compact::from_i64(3)),
        Err(Overflow::NeedsWide)
    );
    assert_eq!(mul(w, Compact::from_i64(1)), w);
    assert_eq!(mul(w, Compact::from_i64(-1)), -w);
    assert_eq!(mul(w, Compact::ZERO), Compact::ZERO);
    let big = add(c(1.0), c(pow2(126)));
    assert_eq!(
        big.checked_mul(Compact::from_i64(3)),
        Err(Overflow::NeedsWide)
    );
    assert_eq!(mul(big, Compact::from_i64(2)).parts(), (1, 1 << 62, 1));
    assert_eq!(
        c(pow2(126)).checked_mul(Compact::from_i64(2)),
        Ok(c(pow2(127)))
    );
    assert_eq!(mul(c(3.0), c(pow2(125))).parts(), (3, 0, 125));
    // Two full mantissas multiply inside the tier; three do not.
    let full = c(f64::MAX);
    let sq = mul(full, full);
    let sqm = (1u128 << 106) - (1 << 54) + 1;
    assert_eq!(sq.parts(), (sqm as u64, (sqm >> 64) as u64, 1942));
    assert_eq!((sqm >> 64) as u64, (1 << 42) - 1);
    assert_eq!(sq.checked_mul(full), Err(Overflow::NeedsWide));
    assert_eq!(sq.read(), (f64::INFINITY, true));
    assert_eq!(
        mul(c(f64::MIN_POSITIVE), c(f64::MIN_POSITIVE)).parts(),
        (1, 0, -2044)
    );
}
