//! `Fixed<LSB, LIMBS>`: exact-or-refused deposit, quantize, the group laws, canonical zero, order,
//! and the reads. Every test is named `fixed_*` so the task verify can filter on it.
//!
//! Oracles: IEEE `a + b`, `a * b`, `a.mul_add(b, c)` and `a / n` are each the correctly rounded
//! value of the exact two-term expression, so they check `to_f64`, `deposit_product` and `div_f64`
//! bit for bit on random inputs. Multi-term rounding is proved against committed vectors later.

use core::cmp::Ordering;
use float_abel::{Fixed, Fixed256, OutOfWindow};

use crate::support::{hash, Rng};

fn dep(x: f64) -> Fixed256 {
    Fixed256::try_from(x).unwrap_or_else(|e| panic!("{x:e} refused: {e}"))
}

const _: () = assert!(core::mem::size_of::<Fixed256>() == 32);

#[test]
fn fixed_size_and_window_constants() {
    assert_eq!(core::mem::size_of::<Fixed256>(), 32);
    assert_eq!(core::mem::size_of::<Fixed<-10, 2>>(), 16);
    assert_eq!(Fixed256::LSB, -96);
    assert_eq!(Fixed256::LIMBS, 4);
    // |x| < 2^95: 256 bits less a sign bit and 64 of headroom, over a 2^-96 unit.
    assert_eq!(Fixed256::CEILING_EXP, 95);
}

#[test]
fn fixed_accepts_the_grid_and_refuses_off_grid_by_name() {
    let mut f = Fixed256::default();
    assert_eq!(f.try_deposit(2f64.powi(-44)), Ok(()));
    assert_eq!(f.try_deposit(2f64.powi(-96)), Ok(()));
    assert_eq!(f.try_deposit(-2f64.powi(-96)), Ok(()));
    assert_eq!(f.try_deposit(1.0 + f64::EPSILON), Ok(()));

    let below = 2f64.powi(-45) * (1.0 + f64::EPSILON);
    assert_eq!(f.try_deposit(below), Err(OutOfWindow::OffGrid));
    assert_eq!(f.try_deposit(2f64.powi(-97)), Err(OutOfWindow::OffGrid));
    assert_eq!(f.try_deposit(f64::MIN_POSITIVE), Err(OutOfWindow::OffGrid));
    assert_eq!(f.try_deposit(5e-324), Err(OutOfWindow::OffGrid));

    assert_eq!(f.try_deposit(2f64.powi(95)), Err(OutOfWindow::AboveCeiling));
    assert_eq!(
        f.try_deposit(-2f64.powi(95)),
        Err(OutOfWindow::AboveCeiling)
    );
    assert_eq!(f.try_deposit(f64::MAX), Err(OutOfWindow::AboveCeiling));
    // The largest double below 2^95 is (2^53 - 1) · 2^42: 191 bits over the unit, which fits.
    let just_under = 2f64.powi(95) * (1.0 - f64::EPSILON / 2.0);
    assert!(just_under < 2f64.powi(95));
    assert_eq!(f.try_deposit(just_under), Ok(()));

    assert_eq!(f.try_deposit(f64::NAN), Err(OutOfWindow::NotFinite));
    assert_eq!(f.try_deposit(f64::INFINITY), Err(OutOfWindow::NotFinite));
    assert_eq!(
        f.try_deposit(f64::NEG_INFINITY),
        Err(OutOfWindow::NotFinite)
    );

    // A refusal leaves the state untouched.
    let before = f;
    assert!(f.try_deposit(f64::NAN).is_err());
    assert!(f.try_deposit(2f64.powi(-97)).is_err());
    assert!(f.try_deposit(f64::MAX).is_err());
    assert_eq!(f, before);
}

#[test]
fn fixed_zero_is_canonical_and_signless() {
    let zero = Fixed256::default();
    assert!(zero.is_zero());
    assert_eq!(zero, Fixed256::ZERO);
    assert_eq!(dep(0.0), zero);
    assert_eq!(dep(-0.0), zero);
    assert_eq!(dep(-0.0).to_f64().to_bits(), 0.0f64.to_bits());
    assert_eq!((dep(1.5) - dep(1.5)).to_f64().to_bits(), 0.0f64.to_bits());
    assert_eq!(-zero, zero);
    assert_eq!(zero.cmp(&zero), Ordering::Equal);
}

#[test]
fn fixed_single_deposits_read_back_bit_for_bit() {
    let mut rng = Rng(1);
    for _ in 0..20_000 {
        let x = rng.on_grid(-44, 94);
        assert_eq!(dep(x).to_f64().to_bits(), x.to_bits(), "{x:e}");
        assert_eq!(dep(x).try_to_f64(), Some(x));
    }
}

#[test]
fn fixed_pair_sums_round_like_ieee_addition() {
    let mut rng = Rng(2);
    for _ in 0..50_000 {
        let (a, b) = (rng.on_grid(-44, 90), rng.on_grid(-44, 90));
        let sum = dep(a) + dep(b);
        assert_eq!(sum.to_f64().to_bits(), (a + b).to_bits(), "{a:e} + {b:e}");
        let exact = sum.try_to_f64();
        assert_eq!(exact.is_some(), sum == dep(a + b), "{a:e} + {b:e}");
    }
    // The classic.
    assert_eq!((dep(0.1) + dep(0.2)).to_f64(), 0.1 + 0.2);
    assert_eq!((dep(0.1) + dep(0.2)).try_to_f64(), None);
}

#[test]
fn fixed_products_round_like_ieee_multiply_and_fma() {
    let mut rng = Rng(3);
    for _ in 0..50_000 {
        // The product's lowest bit is at 2^(ea + eb - 104), on the grid when ea + eb ≥ 8.
        let (a, b, c) = (rng.on_grid(0, 40), rng.on_grid(8, 40), rng.on_grid(-40, 80));
        let mut p = Fixed256::default();
        p.deposit_product(a, b)
            .unwrap_or_else(|e| panic!("{a:e} * {b:e} refused: {e}"));
        assert_eq!(p.to_f64().to_bits(), (a * b).to_bits(), "{a:e} * {b:e}");
        let fma = p + dep(c);
        assert_eq!(
            fma.to_f64().to_bits(),
            a.mul_add(b, c).to_bits(),
            "{a:e} * {b:e} + {c:e}"
        );
    }
    // Off-grid and over-the-ceiling products are refused by name, and leave the state alone.
    let mut p = dep(1.0);
    assert_eq!(
        p.deposit_product(2f64.powi(-50), 2f64.powi(-50)),
        Err(OutOfWindow::OffGrid)
    );
    assert_eq!(
        p.deposit_product(2f64.powi(50), 2f64.powi(50)),
        Err(OutOfWindow::AboveCeiling)
    );
    assert_eq!(
        p.deposit_product(f64::NAN, 1.0),
        Err(OutOfWindow::NotFinite)
    );
    assert_eq!(
        p.deposit_product(1.0, f64::INFINITY),
        Err(OutOfWindow::NotFinite)
    );
    assert_eq!(p, dep(1.0));
    // A zero factor makes the exact product zero, whatever the other factor.
    assert_eq!(p.deposit_product(0.0, f64::MAX), Ok(()));
    assert_eq!(p.deposit_product(-0.0, 5e-324), Ok(()));
    assert_eq!(p, dep(1.0));
}

#[test]
fn fixed_div_f64_rounds_like_ieee_division() {
    let mut rng = Rng(4);
    for _ in 0..50_000 {
        let x = rng.on_grid(-44, 90);
        let n = (rng.next() >> 11).max(1); // < 2^53, so `n as f64` is exact
        assert_eq!(
            dep(x).div_f64(n).to_bits(),
            (x / n as f64).to_bits(),
            "{x:e} / {n}"
        );
    }
    assert_eq!(dep(1.0).div_f64(3), 1.0 / 3.0);
    assert_eq!(dep(-1.0).div_f64(3), -1.0 / 3.0);
    assert_eq!(dep(2.0).div_f64(1), 2.0);
    // 7 / (2^64 - 1) exceeds 7 · 2^-64 by a relative 2^-64, far below half an ulp.
    assert_eq!(dep(7.0).div_f64(u64::MAX), 7.0 * 2f64.powi(-64));
    // Division by zero follows IEEE: 0/0 is NaN, x/0 is ±∞.
    assert!(Fixed256::ZERO.div_f64(0).is_nan());
    assert_eq!(dep(1.0).div_f64(0), f64::INFINITY);
    assert_eq!(dep(-1.0).div_f64(0), f64::NEG_INFINITY);
    assert_eq!(Fixed256::ZERO.div_f64(7).to_bits(), 0.0f64.to_bits());
}

#[test]
fn fixed_sum_then_divide_is_the_exact_mean() {
    // 10 copies of 0.1 sum to exactly 1 on the grid; the mean of the exact sum is exactly 0.1.
    let mut f = Fixed256::default();
    for _ in 0..10 {
        f.try_deposit(0.1).unwrap();
    }
    assert_eq!(f.try_to_f64(), None, "ten exact tenths are not a double");
    assert_eq!(f.to_f64(), 1.0);
    assert_eq!(f.div_f64(10), 0.1);
    // The f64 fold lands elsewhere.
    let folded: f64 = (0..10).map(|_| 0.1f64).sum();
    assert_ne!(folded, 1.0);
}

#[test]
fn fixed_group_laws_hold_bit_for_bit() {
    let mut rng = Rng(5);
    for _ in 0..2_000 {
        let (a, b, c) = (
            dep(rng.on_grid(-44, 92)),
            dep(rng.on_grid(-44, 92)),
            dep(rng.on_grid(-44, 92)),
        );
        assert_eq!(a + b, b + a);
        assert_eq!((a + b) + c, a + (b + c));
        assert_eq!(a + Fixed256::ZERO, a);
        assert_eq!(a + (-a), Fixed256::ZERO);
        assert_eq!(a + b - b, a);
        assert_eq!(-(-a), a);
        assert_eq!(a - b, a + (-b));
        let mut aa = a;
        aa += b;
        aa -= b;
        assert_eq!(aa, a);
    }
}

#[test]
fn fixed_folds_are_order_independent() {
    let mut rng = Rng(6);
    let xs: Vec<f64> = (0..1_000).map(|_| rng.on_grid(-44, 80)).collect();
    let forward: Fixed256 = xs.iter().map(|&x| dep(x)).sum();
    let backward: Fixed256 = xs.iter().rev().map(|&x| dep(x)).sum();
    assert_eq!(forward, backward);
    // Random partitions, merged in random order.
    for round in 0..50 {
        let mut parts = vec![Fixed256::ZERO; 1 + round % 7];
        for &x in &xs {
            let i = (rng.next() % parts.len() as u64) as usize;
            parts[i] += dep(x);
        }
        let mut merged = Fixed256::ZERO;
        while !parts.is_empty() {
            let i = (rng.next() % parts.len() as u64) as usize;
            merged += parts.swap_remove(i);
        }
        assert_eq!(merged, forward);
    }
    // The f64 fold is not order independent on the same data, which is the point.
    let f: f64 = xs.iter().sum();
    let r: f64 = xs.iter().rev().sum();
    assert_ne!(
        f.to_bits(),
        r.to_bits(),
        "a 1000-term f64 fold happened to commute; reseed"
    );
}

#[test]
#[allow(clippy::erasing_op)] // scaling by a literal zero is one of the cases under test
fn fixed_agrees_with_an_i128_oracle_on_a_small_window() {
    // Fixed<-10, 2>: 128 bits, unit 2^-10, ceiling 2^(128 - 65 - 10) = 2^53. Every state is an
    // i128 count of units, so i128 arithmetic is an exact oracle for the group and the order.
    type F = Fixed<-10, 2>;
    let mut rng = Rng(7);
    let mut acc = F::default();
    let mut oracle: i128 = 0;
    for _ in 0..10_000 {
        let units = i128::from(rng.next() >> 14) * if rng.next() & 1 == 0 { 1 } else { -1 };
        let x = units as f64 * 2f64.powi(-10);
        let mut term = F::default();
        term.try_deposit(x).unwrap();
        acc += term;
        oracle += units;
        let expect = {
            let lo = oracle as u64;
            let hi = (oracle >> 64) as u64;
            [lo, hi]
        };
        assert_eq!(acc.limbs(), &expect);
        assert_eq!(acc.to_f64(), oracle as f64 * 2f64.powi(-10));
        assert_eq!(acc.cmp(&term), oracle.cmp(&units));
        assert_eq!(acc.is_zero(), oracle == 0);
    }
    assert_eq!(acc * 3i64, {
        let mut t = acc;
        t += acc;
        t += acc;
        t
    });
    assert_eq!(acc * -1i64, -acc);
    assert_eq!(acc * 0i64, F::ZERO);
    assert_eq!(acc * 2isize, acc + acc);
}

#[test]
fn fixed_order_is_numeric_and_total() {
    let mut rng = Rng(8);
    for _ in 0..20_000 {
        let (a, b) = (rng.on_grid(-44, 92), rng.on_grid(-44, 92));
        assert_eq!(
            dep(a).cmp(&dep(b)),
            a.partial_cmp(&b).unwrap(),
            "{a:e} vs {b:e}"
        );
        assert_eq!(dep(a).partial_cmp(&dep(b)), Some(dep(a).cmp(&dep(b))));
    }
    assert!(dep(-1.0) < Fixed256::ZERO);
    assert!(Fixed256::ZERO < dep(2f64.powi(-96)));
    assert!(dep(-2f64.powi(94)) < dep(2f64.powi(-96)));
    // Structural Eq is numeric equality: two routes to one value are one state.
    assert_eq!(dep(0.5) + dep(0.25), dep(0.75));
    assert_eq!(dep(3.0) - dep(1.0), dep(1.0) + dep(1.0));
    assert_eq!(hash(&(dep(0.5) + dep(0.25))), hash(&dep(0.75)));
}

#[test]
fn fixed_quantize_is_round_to_nearest_even_on_the_grid_and_odd() {
    let q = Fixed256::quantize;
    // On-grid values pass through untouched.
    assert_eq!(q(1.0).to_bits(), 1.0f64.to_bits());
    assert_eq!(q(2f64.powi(-96)).to_bits(), 2f64.powi(-96).to_bits());
    assert_eq!(q(f64::MAX).to_bits(), f64::MAX.to_bits());
    // 2^-45 · (1 + ε) is 2^51 + 1/2 units: a tie, rounded to the even 2^51 units.
    assert_eq!(q(2f64.powi(-45) * (1.0 + f64::EPSILON)), 2f64.powi(-45));
    // 2^-45 · (1 + 2ε) is 2^51 + 1 units, exact.
    let x = 2f64.powi(-45) * (1.0 + 2.0 * f64::EPSILON);
    assert_eq!(q(x).to_bits(), x.to_bits());
    // 2^-45 · (1 + 3ε) is 2^51 + 3/2 units: a tie, rounded to the even 2^51 + 2.
    assert_eq!(
        q(2f64.powi(-45) * (1.0 + 3.0 * f64::EPSILON)),
        x + 2f64.powi(-96)
    );
    // Below half a unit rounds to zero, keeping its sign; exactly half a unit ties to zero.
    assert_eq!(q(2f64.powi(-98)).to_bits(), 0.0f64.to_bits());
    assert_eq!(q(-2f64.powi(-98)).to_bits(), (-0.0f64).to_bits());
    assert_eq!(q(2f64.powi(-97)).to_bits(), 0.0f64.to_bits());
    assert_eq!(q(2f64.powi(-97) * 1.5), 2f64.powi(-96));
    assert_eq!(q(5e-324).to_bits(), 0.0f64.to_bits());
    // Specials pass through: quantize declares a floor, not a window.
    assert!(q(f64::NAN).is_nan());
    assert_eq!(q(f64::INFINITY), f64::INFINITY);
    assert_eq!(q(f64::NEG_INFINITY), f64::NEG_INFINITY);
    // Odd under negation, bit for bit, and always on the grid.
    let mut rng = Rng(9);
    for _ in 0..50_000 {
        let x = f64::from_bits(rng.next() & !(0x7FFu64 << 52) | ((rng.next() % 0x7FF) << 52));
        assert_eq!(q(-x).to_bits(), (-q(x)).to_bits(), "{x:e}");
        let mut f = Fixed256::default();
        match f.try_deposit(q(x)) {
            Ok(()) | Err(OutOfWindow::AboveCeiling) => {}
            Err(e) => panic!("quantize({x:e}) = {:e} refused: {e}", q(x)),
        }
        // Idempotent.
        assert_eq!(q(q(x)).to_bits(), q(x).to_bits());
        // Never off by more than half a unit, as a distance in units.
        if x.is_finite() && x.abs() < 2f64.powi(60) {
            assert!((q(x) - x).abs() <= 2f64.powi(-97), "{x:e} -> {:e}", q(x));
        }
    }
}

#[test]
fn fixed_subnormal_and_overflow_reads_round_correctly() {
    // A window whose unit is 2^-1076: below the smallest double, so reads can land subnormal.
    type Deep = Fixed<-1076, 20>;
    type Big = Fixed<0, 20>;
    let mut f = Deep::default();
    // 2^-1075 is half the smallest subnormal: a tie, rounded to the even zero.
    f.deposit_product(2f64.powi(-540), 2f64.powi(-535)).unwrap();
    assert_eq!(f.to_f64().to_bits(), 0.0f64.to_bits());
    assert_eq!(f.try_to_f64(), None);
    // Three quarters of it rounds up to the smallest subnormal.
    f.deposit_product(2f64.powi(-540), 2f64.powi(-536)).unwrap();
    assert_eq!(f.to_f64().to_bits(), f64::from_bits(1).to_bits());
    // Exactly 3 · 2^-1074 reads exactly.
    let mut g = Deep::default();
    g.try_deposit(3.0 * 5e-324).unwrap();
    assert_eq!(g.to_f64().to_bits(), f64::from_bits(3).to_bits());
    assert_eq!(g.try_to_f64(), Some(f64::from_bits(3)));
    // A negative value that rounds to zero reads as -0.0, as IEEE rounds it; only the zero state
    // itself is signless.
    let mut n = Deep::default();
    n.deposit_product(-2f64.powi(-540), 2f64.powi(-535))
        .unwrap();
    assert_eq!(n.to_f64().to_bits(), (-0.0f64).to_bits());
    assert_eq!((n - n).to_f64().to_bits(), 0.0f64.to_bits());

    // A window wide enough to overflow a double.
    let mut big = Big::default();
    big.try_deposit(2f64.powi(1023)).unwrap();
    big.try_deposit(2f64.powi(1023)).unwrap();
    assert_eq!(big.to_f64(), f64::INFINITY);
    assert_eq!(big.try_to_f64(), None);
    assert_eq!((-big).to_f64(), f64::NEG_INFINITY);
    let mut max = Big::default();
    max.try_deposit(f64::MAX).unwrap();
    assert_eq!(max.to_f64(), f64::MAX);
    // MAX + half an ulp ties to even; MAX's mantissa is odd, so it rounds away, to ∞.
    let mut tie = max;
    tie.try_deposit(2f64.powi(970)).unwrap();
    assert_eq!(tie.to_f64(), f64::INFINITY);
    // MAX + less than half an ulp stays MAX.
    let mut under = max;
    under.try_deposit(2f64.powi(969)).unwrap();
    assert_eq!(under.to_f64(), f64::MAX);
    // A division can overflow too, and 0 / n is an exact zero.
    assert_eq!(big.div_f64(1), f64::INFINITY);
    assert_eq!(big.div_f64(2), 2f64.powi(1023));
    // 2^1024 / 3 is finite; 2^1023 / 3 is correctly rounded by IEEE and doubling it is exact.
    assert_eq!(big.div_f64(3), (2f64.powi(1023) / 3.0) * 2.0);
}

#[test]
#[should_panic(expected = "headroom")]
fn fixed_exhausting_the_headroom_is_a_named_panic_in_every_build() {
    // 2^95 - ulp is the largest deposit; 2^64 of them would just fit. One past the signed limit
    // is a named panic in every build, release included: the gate runs this under --release.
    let top = 2f64.powi(95) * (1.0 - f64::EPSILON / 2.0);
    let mut f = dep(top);
    for _ in 0..64 {
        f += f; // 2^k copies of the top value: 2^64 copies overflow the sign bit
    }
    let _ = f + f;
}

#[test]
#[should_panic(expected = "scalar multiply")]
fn fixed_overflowing_a_scalar_multiply_is_a_named_panic_in_every_build() {
    // 2^63 copies of the top value sit just under 2^254 units, inside the 255 magnitude bits;
    // three times that crosses 2^255 into the sign bit, and the multiply names itself.
    let top = 2f64.powi(95) * (1.0 - f64::EPSILON / 2.0);
    let mut f = dep(top);
    for _ in 0..63 {
        f += f;
    }
    let _ = f * 3i64;
}

#[test]
fn fixed_out_of_window_is_an_error_with_a_message() {
    let e: Box<dyn std::error::Error> = Box::new(OutOfWindow::OffGrid);
    assert!(e.to_string().contains("grid"), "{e}");
    assert!(OutOfWindow::AboveCeiling.to_string().contains("ceiling"));
    assert!(OutOfWindow::NotFinite.to_string().contains("finite"));
    assert_eq!(format!("{:?}", OutOfWindow::NotFinite), "NotFinite");
}

#[test]
fn fixed_debug_names_the_window_and_the_value() {
    let s = format!("{:?}", dep(1.5));
    assert!(s.contains("Fixed<-96, 4>"), "{s}");
    assert!(s.contains("1.5"), "{s}");
}
