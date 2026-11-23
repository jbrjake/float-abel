//! `Fixed::try_from(&Exact)` and `Exact::finite_part`, DESIGN.md § Operations and conversions: the
//! finite part of an `Exact` placed on a window's grid from the compact and the heap form, refused
//! by name where `Fixed::try_deposit` refuses, and refused as `NotFinite` while a special amount
//! is nonzero. Every test is named `convert_exact_to_fixed_*` so the task verify can filter on it.
//!
//! Oracles: the state the same deposits build in `Fixed` directly, which never passes through
//! `Exact`; and `Exact::try_from(&Fixed)`, the conversion this one inverts, on limb patterns that
//! enter through `from_limbs`.

use float_abel::{Exact, Fixed, Fixed256, OutOfWindow};

use crate::support::{pow2, rounds, Rng};

/// The product window an engine folds `Σ a·b` in: every product of two finite doubles is on its
/// grid and under its ceiling.
type Product = Fixed<-2148, 68>;

/// A state under the ceiling whose low limbs are each all zeros, all ones or random: `wide`
/// limbs of them, the rest the sign's fill, and the top two bits of the highest magnitude limb
/// the sign's, so the magnitude stays below `2^(64·LIMBS − 66)`.
fn state<const LIMBS: usize>(rng: &mut Rng, wide: usize) -> [u64; LIMBS] {
    let negative = rng.next() & 1 == 1;
    let mut limbs = [if negative { u64::MAX } else { 0 }; LIMBS];
    for l in limbs.iter_mut().take(wide.min(LIMBS - 1)) {
        *l = match rng.next() % 4 {
            0 => 0,
            1 => u64::MAX,
            _ => rng.next(),
        };
    }
    let top = &mut limbs[LIMBS - 2];
    *top = if negative {
        *top | 3 << 62
    } else {
        *top & (u64::MAX >> 2)
    };
    limbs
}

/// `n` states of `Fixed<LSB, LIMBS>` through `Exact` and back. Returns how many of the `Exact`s
/// were compact and how many were in the heap form.
fn round_trips<const LSB: i32, const LIMBS: usize>(rng: &mut Rng, n: usize) -> (usize, usize) {
    let (mut compact, mut heap) = (0, 0);
    for i in 0..n {
        // Alternate states of at most two limbs, most of which are compact, with full-width ones.
        let wide = if i % 2 == 0 { 1 + i % 4 / 2 } else { LIMBS };
        let fixed = Fixed::<LSB, LIMBS>::from_limbs(state(rng, wide));
        let exact = Exact::try_from(&fixed).unwrap_or_else(|e| panic!("{fixed:?}: {e}"));
        if exact.is_compact() {
            compact += 1;
        } else {
            heap += 1;
        }
        assert_eq!(
            Fixed::<LSB, LIMBS>::try_from(&exact),
            Ok(fixed),
            "{exact:?}"
        );
        assert_eq!(Fixed::<LSB, LIMBS>::try_from(&-&exact), Ok(-fixed));
    }
    (compact, heap)
}

#[test]
fn convert_exact_to_fixed_round_trips_every_state_bit_for_bit() {
    let mut rng = Rng(0xc0de_f1ed_e8ac_7001);
    let n = rounds(4_000);
    let (compact, heap) = round_trips::<-96, 4>(&mut rng, n);
    assert_eq!(compact + heap, n);
    assert!(
        compact >= n / 8 && heap >= n / 4,
        "{compact} compact, {heap} heap"
    );
    let n = rounds(1_000);
    let (compact, heap) = round_trips::<-2148, 68>(&mut rng, n);
    assert_eq!(compact + heap, n);
    assert!(
        compact >= n / 8 && heap >= n / 4,
        "{compact} compact, {heap} heap"
    );
}

#[test]
fn convert_exact_to_fixed_product_fold_lands_the_fixed_fold() {
    // The shape that asked for the conversion: Σ a·b folded in `Exact` and read into the product
    // window, against the same products folded in the window, with retractions.
    let mut rng = Rng(0x9a0d_0c75_5eed_0002);
    let mut heap = 0usize;
    let folds = rounds(300);
    for _ in 0..folds {
        let mut exact = Exact::ZERO;
        let mut fixed = Product::ZERO;
        for _ in 0..=rng.next() % 24 {
            let (a, b) = (rng.finite(), rng.finite());
            let mut term = Product::ZERO;
            term.deposit_product(a, b).unwrap();
            if rng.next() % 4 == 0 {
                exact -= Exact::product(a, b);
                fixed -= term;
            } else {
                exact.deposit_product(a, b);
                fixed += term;
            }
        }
        heap += usize::from(!exact.is_compact());
        assert_eq!(Product::try_from(&exact), Ok(fixed), "{exact:?}");
    }
    assert!(
        heap >= folds / 2,
        "{heap} of {folds} folds left the compact form"
    );
}

#[test]
fn convert_exact_to_fixed_refuses_by_name_at_each_edge_and_accepts_inside_it() {
    let fixed = |x: f64| Fixed256::try_from(x).unwrap();
    // The unit is on the grid, one bit below it is not, whatever the sign.
    let unit = Exact::from(pow2(-96));
    assert_eq!(Fixed256::try_from(&unit), Ok(fixed(pow2(-96))));
    assert_eq!(Fixed256::try_from(&-&unit), Ok(fixed(-pow2(-96))));
    let below = Exact::from(pow2(-97));
    assert_eq!(Fixed256::try_from(&below), Err(OutOfWindow::OffGrid));
    assert_eq!(Fixed256::try_from(&-&below), Err(OutOfWindow::OffGrid));

    // The ceiling is 2^95: the largest double under it passes and 2^95 is refused.
    assert_eq!(Fixed256::CEILING_EXP, 95);
    let top = pow2(95) * (1.0 - f64::EPSILON / 2.0);
    assert_eq!(Fixed256::try_from(&Exact::from(top)), Ok(fixed(top)));
    let at = Exact::from(pow2(95));
    assert_eq!(Fixed256::try_from(&at), Err(OutOfWindow::AboveCeiling));
    assert_eq!(Fixed256::try_from(&-&at), Err(OutOfWindow::AboveCeiling));

    // The same edges from the heap form: 2^94 + 2^-96 spans 191 bits and fills the window.
    let whole = Exact::from(pow2(94)) + Exact::from(pow2(-96));
    assert!(!whole.is_compact());
    assert_eq!(
        Fixed256::try_from(&whole),
        Ok(fixed(pow2(94)) + fixed(pow2(-96)))
    );
    assert_eq!(
        Fixed256::try_from(&-&whole),
        Ok(-(fixed(pow2(94)) + fixed(pow2(-96))))
    );
    let low = Exact::from(pow2(94)) + Exact::from(pow2(-97));
    assert!(!low.is_compact());
    assert_eq!(Fixed256::try_from(&low), Err(OutOfWindow::OffGrid));
    let high = Exact::from(pow2(95)) + Exact::from(pow2(-96));
    assert!(!high.is_compact());
    assert_eq!(Fixed256::try_from(&high), Err(OutOfWindow::AboveCeiling));
    assert_eq!(Fixed256::try_from(&-&high), Err(OutOfWindow::AboveCeiling));
    // Past both edges the grid is named, as `try_deposit` names it.
    let both = Exact::from(pow2(95)) + Exact::from(pow2(-97));
    assert_eq!(Fixed256::try_from(&both), Err(OutOfWindow::OffGrid));

    // The zero passes, and the product window takes the smallest and the largest product.
    assert_eq!(Fixed256::try_from(&Exact::ZERO), Ok(Fixed256::ZERO));
    for (a, b) in [(5e-324, 5e-324), (f64::MAX, -f64::MAX)] {
        let mut want = Product::ZERO;
        want.deposit_product(a, b).unwrap();
        assert_eq!(Product::try_from(&Exact::product(a, b)), Ok(want));
    }
}

#[test]
fn convert_exact_to_fixed_refuses_a_special_amount_and_finite_part_leaves_it_behind() {
    let nan = Exact::from(f64::NAN);
    let inf = Exact::from(f64::INFINITY);
    let fixed = |x: f64| Fixed256::try_from(x).unwrap();

    // 1.5 and one NaN: the conversion refuses, the finite part is 1.5, the count is read apart.
    let e = Exact::from(1.5) + &nan;
    assert_eq!(Fixed256::try_from(&e), Err(OutOfWindow::NotFinite));
    assert_eq!(e.finite_part(), Exact::from(1.5));
    assert_eq!(Fixed256::try_from(&e.finite_part()), Ok(fixed(1.5)));
    assert_eq!(e.specials(), Some((1, 0, 0)));

    // A retracted infinity is a nonzero amount too, and a bare special has a zero finite part.
    let e = Exact::from(-0.75) - &inf;
    assert_eq!(e.specials(), Some((0, -1, 0)));
    assert_eq!(Fixed256::try_from(&e), Err(OutOfWindow::NotFinite));
    assert_eq!(Fixed256::try_from(&e.finite_part()), Ok(fixed(-0.75)));
    assert_eq!(Fixed256::try_from(&nan), Err(OutOfWindow::NotFinite));
    assert_eq!(nan.finite_part(), Exact::ZERO);

    // The amounts are judged before the window: with them gone the window's own name comes out.
    let e = Exact::from(pow2(95)) + &nan;
    assert_eq!(Fixed256::try_from(&e), Err(OutOfWindow::NotFinite));
    assert_eq!(
        Fixed256::try_from(&e.finite_part()),
        Err(OutOfWindow::AboveCeiling)
    );

    // The heap form: a finite part past the width beside a count, and a dyadic amount, 2.5 NaN,
    // beside a finite part that fits the word again once the amount is gone.
    let wide = Exact::from(pow2(94)) + Exact::from(pow2(-96));
    let e = &wide + &inf;
    assert!(!e.is_compact());
    assert_eq!(Fixed256::try_from(&e), Err(OutOfWindow::NotFinite));
    assert_eq!(e.finite_part(), wide);
    assert!(!e.finite_part().is_compact());
    assert_eq!(
        Fixed256::try_from(&e.finite_part()),
        Fixed256::try_from(&wide)
    );
    let e = Exact::from(1.5) + Exact::from(2.5) * &nan;
    assert_eq!(e.specials(), None);
    assert_eq!(Fixed256::try_from(&e), Err(OutOfWindow::NotFinite));
    assert_eq!(e.finite_part(), Exact::from(1.5));
    assert!(e.finite_part().is_compact());

    // A finite value is its own finite part, in either form.
    assert_eq!(Exact::from(1.5).finite_part(), Exact::from(1.5));
    assert_eq!(wide.finite_part(), wide);
    assert!(e.finite_part().is_finite() && wide.finite_part().is_finite());
}
