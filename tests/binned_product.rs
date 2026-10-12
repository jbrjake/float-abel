//! `Binned<K>`'s product deposit: the 106-bit product sliced onto the bins with the product's
//! sign, equal to the exact product truncated below the window and below the grid, bit-identical
//! under any order or partition, inside the bound with its floor, and refused by name at the
//! ceiling with the state untouched. The oracle places each set bit of the mantissa product in a
//! `Wide` limb by hand from the factors' bits, so it shares nothing with the crate's limb
//! arithmetic; `Wide::deposit_product` is the second oracle wherever the product is on the grid.

use float_abel::{Binned, Exact, OutOfWindow, Wide};

use crate::binned::{assert_same_bits, shuffle};
use crate::support::{pow2, Rng};

const INF: f64 = f64::INFINITY;
const NINF: f64 = f64::NEG_INFINITY;
const NAN: f64 = f64::NAN;
/// A full 52-bit fraction: every bit set.
const FULL: u64 = (1 << 52) - 1;

/// A finite double as `(negative, mant, exp)` meaning `mant · 2^exp`, with the mantissa as the
/// bits hold it, the hidden bit restored and nothing reduced to odd.
fn dyadic(x: f64) -> (bool, u64, i32) {
    let bits = x.to_bits();
    let negative = bits >> 63 != 0;
    let biased = ((bits >> 52) & 0x7FF) as i32;
    let frac = bits & FULL;
    assert_ne!(biased, 0x7FF, "{x} is finite");
    if biased == 0 {
        (negative, frac, -1074)
    } else {
        (negative, frac | (1 << 52), biased - 1075)
    }
}

/// A normal double with its leading bit at `2^e` and the fraction `frac`, either sign.
fn normal(negative: bool, e: i32, frac: u64) -> f64 {
    let bits = ((e + 1023) as u64) << 52 | frac;
    f64::from_bits(if negative { bits | 1 << 63 } else { bits })
}

/// The exact product `a · b` with every bit below `Wide` limb `lo` dropped, which with `lo = 0`
/// is every bit below the unit: each set bit of the mantissa product placed in a limb by hand,
/// then the sign. The caller keeps the product under the ceiling, so every kept bit has a limb.
fn product_truncated(a: f64, b: f64, lo: usize) -> Wide {
    let (na, ma, ea) = dyadic(a);
    let (nb, mb, eb) = dyadic(b);
    let mant = u128::from(ma) * u128::from(mb);
    let mut limbs = [0u64; 34];
    for i in 0..128i32 {
        if (mant >> i) & 1 == 1 {
            // The bit weighs 2^(ea + eb + i); its position above the unit is 1074 more.
            let p = ea + eb + i + 1074;
            if p >= 64 * lo as i32 {
                limbs[(p / 64) as usize] |= 1 << (p % 64);
            }
        }
    }
    let w = Wide::from_parts(limbs, [0; 3]);
    if na == nb {
        w
    } else {
        -w
    }
}

fn wide_product(a: f64, b: f64) -> Result<Wide, OutOfWindow> {
    let mut w = Wide::ZERO;
    w.deposit_product(a, b)?;
    Ok(w)
}

fn one<const K: usize>(a: f64, b: f64) -> Binned<K> {
    let mut bins = Binned::<K>::ZERO;
    bins.deposit_product(a, b)
        .expect("the product is under the ceiling");
    bins
}

fn fold<const K: usize>(pairs: &[(f64, f64)]) -> Binned<K> {
    let mut bins = Binned::<K>::ZERO;
    for &(a, b) in pairs {
        bins.deposit_product(a, b)
            .expect("the product is under the ceiling");
    }
    bins
}

/// The `Wide` limb of the top bit of the product's part on the grid, or `None` when a zero factor
/// or a product wholly below the unit leaves nothing there.
fn product_top_limb(a: f64, b: f64) -> Option<usize> {
    let w = product_truncated(a, b, 0);
    let mag = if w < Wide::ZERO { -w } else { w };
    mag.limbs().iter().rposition(|&l| l != 0)
}

/// The lowest limb a `Binned<K>` keeps after folding the products of `pairs`: the window's top
/// is the limb of the largest product on the grid, and never below limb `K - 1`.
fn window_bottom<const K: usize>(pairs: &[(f64, f64)]) -> usize {
    let top = pairs
        .iter()
        .filter(|(a, b)| a.is_finite() && b.is_finite())
        .filter_map(|&(a, b)| product_top_limb(a, b))
        .max()
        .unwrap_or(0);
    top.max(K - 1) + 1 - K
}

/// The definition of the value: every finite product truncated below `lo`, summed exactly, and a
/// special factor counted as `Wide::deposit_product` counts it.
fn truncated_dot(pairs: &[(f64, f64)], lo: usize) -> Wide {
    let mut acc = Wide::ZERO;
    for &(a, b) in pairs {
        if a.is_finite() && b.is_finite() {
            acc += product_truncated(a, b, lo);
        } else {
            acc += wide_product(a, b).expect("a special factor is never refused");
        }
    }
    acc
}

/// Pairs of any double whose product is under the ceiling, so every fold below accepts them.
fn any_pairs(rng: &mut Rng, n: usize) -> Vec<(f64, f64)> {
    (0..n)
        .map(|_| (rng.any(), rng.any()))
        .filter(|&(a, b)| wide_product(a, b) != Err(OutOfWindow::AboveCeiling))
        .collect()
}

#[test]
fn binned_product_is_the_exact_products_truncated_below_the_window_and_the_grid() {
    fn check<const K: usize>(pairs: &[(f64, f64)], what: &str) {
        let lo = window_bottom::<K>(pairs);
        let bins = fold::<K>(pairs);
        assert_eq!(bins.index() as usize, lo + K - 1, "{what}: K={K} index");
        assert_eq!(bins.into_wide(), truncated_dot(pairs, lo), "{what}: K={K}");
    }
    let mut rng = Rng(71);
    // Pairs Wide holds too, where the hand-built oracle must agree with it.
    let held: Vec<(f64, f64)> = (0..3000)
        .map(|_| (rng.on_grid(-480, 480), rng.on_grid(-480, 480)))
        .collect();
    for &(a, b) in &held {
        assert_eq!(
            product_truncated(a, b, 0),
            wide_product(a, b).unwrap(),
            "{a:e} * {b:e}: the oracle agrees with Wide on the grid"
        );
    }
    // Small normals with full mantissas, whose products reach below the unit.
    let low: Vec<(f64, f64)> = (0..3000)
        .map(|_| (rng.on_grid(-560, -460), rng.on_grid(-560, -460)))
        .collect();
    let off_grid = low
        .iter()
        .filter(|&&(a, b)| wide_product(a, b) == Err(OutOfWindow::OffGrid))
        .count();
    assert!(
        off_grid > 1000,
        "{off_grid} of the low band's products reach below the unit"
    );
    let any = any_pairs(&mut rng, 3000);
    assert!(any.iter().any(|(a, b)| !a.is_finite() || !b.is_finite()));
    for (pairs, what) in [(&held, "held"), (&low, "low"), (&any, "any")] {
        check::<1>(pairs, what);
        check::<2>(pairs, what);
        check::<3>(pairs, what);
        check::<5>(pairs, what);
        check::<33>(pairs, what);
    }
}

#[test]
fn binned_product_drops_the_bits_below_the_grid_and_keeps_the_rest() {
    // 2^-537 · 1.5·2^-537 = 1.5 · 2^-1074: Wide refuses it, here the unit bit survives and the
    // half below it is gone, and a product on limb 0 does not move the window.
    let (x, y) = (pow2(-537), 1.5 * pow2(-537));
    assert_eq!(wide_product(x, y), Err(OutOfWindow::OffGrid));
    let bins = one::<3>(x, y);
    assert_eq!(bins.into_wide().try_to_f64(), Some(5e-324));
    assert_eq!(bins.index(), 2);
    assert_eq!(one::<3>(-x, y).into_wide().try_to_f64(), Some(-5e-324));
    // Wholly below the unit: nothing is deposited and nothing moves.
    let before = fold::<3>(&[(1.0, 1.0)]);
    let mut same = before;
    for (a, b) in [
        (5e-324, 0.5),
        (5e-324, 5e-324),
        (-5e-324, 5e-324),
        (pow2(-1000), pow2(-100)),
        (f64::MIN_POSITIVE, -pow2(-60)),
        (x, 0.75 * pow2(-537)),
    ] {
        assert_eq!(
            wide_product(a, b),
            Err(OutOfWindow::OffGrid),
            "{a:e} * {b:e}"
        );
        assert_eq!(same.deposit_product(a, b), Ok(()), "{a:e} * {b:e}");
        assert_same_bits(&same, &before, "wholly below the unit");
    }
    // A full mantissa straddling the unit: the bits at or above it survive, bit for bit and
    // toward zero on both signs, whatever the window.
    let (a, b) = (
        normal(false, -512, FULL),
        normal(false, -540, 0x5_5555_5555_5555),
    );
    assert_eq!(wide_product(a, b), Err(OutOfWindow::OffGrid));
    let want = product_truncated(a, b, 0);
    assert_ne!(want, Wide::ZERO);
    assert_eq!(one::<1>(a, b).into_wide(), want);
    assert_eq!(one::<2>(a, b).into_wide(), want);
    assert_eq!(one::<33>(a, b).into_wide(), want);
    assert_eq!(one::<2>(-a, b).into_wide(), -want);
    assert_eq!(one::<2>(a, -b).into_wide(), -want);
    assert_eq!(one::<2>(-a, -b).into_wide(), want);
}

#[test]
fn binned_product_crosses_three_limbs_and_straddles_the_windows_bottom() {
    // Two full mantissas, 106 bits, at a shift of 38 within the limb: limbs 16, 17 and 18.
    let (a, b) = (normal(false, 52, FULL), normal(true, 40, FULL));
    let want = wide_product(a, b).unwrap();
    assert_eq!(
        want.limbs()
            .iter()
            .enumerate()
            .filter(|(_, &l)| l != 0)
            .count(),
        34 - 16,
        "a negative product fills every limb from 16 up"
    );
    let three = one::<3>(a, b);
    assert_eq!(three.index(), 18);
    assert!(three.limbs().iter().all(|&l| l != 0), "{three:?}");
    assert_eq!(three.into_wide(), want);
    assert_eq!(one::<2>(a, b).into_wide(), product_truncated(a, b, 17));
    assert_eq!(one::<1>(a, b).into_wide(), product_truncated(a, b, 18));
    assert_ne!(product_truncated(a, b, 18), want);

    // 2^78, the lowest bit of limb 18, fixes a one-bin window there; a product spanning limbs
    // 17 and 18 keeps only its limb-18 slice, whether it arrives before or after the one that
    // set the window.
    let big = (pow2(40), pow2(38));
    assert_eq!(product_top_limb(big.0, big.1), Some(18));
    let straddler = (normal(false, 52, FULL), normal(false, 78, FULL));
    assert_eq!(product_top_limb(straddler.0, straddler.1), Some(18));
    assert_eq!(
        product_truncated(straddler.0, straddler.1, 17),
        product_truncated(straddler.0, straddler.1, 0),
        "the straddler has nothing below limb 17"
    );
    assert_ne!(
        product_truncated(straddler.0, straddler.1, 18),
        product_truncated(straddler.0, straddler.1, 0),
        "and something in limb 17"
    );
    let first = fold::<1>(&[big, straddler]);
    let second = fold::<1>(&[straddler, big]);
    assert_same_bits(&first, &second, "the straddler either side of the rise");
    assert_eq!(
        first.into_wide(),
        product_truncated(big.0, big.1, 18) + product_truncated(straddler.0, straddler.1, 18)
    );
    assert_ne!(
        first.into_wide(),
        wide_product(big.0, big.1).unwrap() + wide_product(straddler.0, straddler.1).unwrap(),
        "the slice below limb 18 is gone"
    );
}

#[test]
fn binned_product_specials_and_zeros_follow_wides_rule_and_never_move_the_window() {
    fn specials(a: f64, b: f64) -> (i64, i64, i64) {
        let bins = one::<3>(a, b);
        assert_eq!(bins.index(), 2, "{a} * {b}: a special moves nothing");
        assert_eq!(
            bins.specials(),
            wide_product(a, b).unwrap().specials(),
            "{a} * {b}: as Wide counts it"
        );
        bins.specials()
    }
    for b in [2.0, -2.0, 5e-324, INF, NINF, NAN] {
        assert_eq!(specials(NAN, b), (1, 0, 0), "NaN * {b}");
        assert_eq!(specials(b, NAN), (1, 0, 0), "{b} * NaN");
        assert!(one::<3>(NAN, b).into_wide().to_f64().is_nan());
    }
    assert_eq!(specials(INF, INF), (0, 1, 0));
    assert_eq!(specials(NINF, NINF), (0, 1, 0));
    assert_eq!(specials(INF, NINF), (0, 0, 1));
    assert_eq!(specials(NINF, INF), (0, 0, 1));
    assert_eq!(specials(INF, 2.5), (0, 1, 0));
    assert_eq!(specials(INF, -2.5), (0, -1, 0));
    assert_eq!(one::<3>(INF, -2.5).into_wide().to_f64(), NINF);
    assert_eq!(specials(-2.5, NINF), (0, 0, -1));
    assert_eq!(one::<3>(-2.5, NINF).into_wide().to_f64(), INF);
    assert_eq!(specials(5e-324, NINF), (0, 0, 1));
    assert_eq!(specials(f64::MAX, INF), (0, 1, 0));
    // A zero factor makes the exact product zero whatever the other factor, so even 0 · ∞ and
    // 0 · NaN deposit nothing, and a special never raises the index.
    let before = fold::<3>(&[(1.0, 1.0)]);
    let mut same = before;
    for (a, b) in [
        (0.0, f64::MAX),
        (-0.0, 5e-324),
        (0.0, INF),
        (NINF, -0.0),
        (NAN, 0.0),
        (0.0, 0.0),
    ] {
        assert_eq!(same.deposit_product(a, b), Ok(()), "{a} * {b}");
        assert_same_bits(&same, &before, "a zero factor");
    }
    same.deposit_product(INF, 1e300).unwrap();
    assert_eq!(same.index(), before.index());
    assert_eq!(same.specials(), (0, 1, 0));
}

#[test]
fn binned_product_refuses_the_ceiling_by_name_and_leaves_the_state_untouched() {
    let before = fold::<3>(&[(1.0, 1.0)]);
    let mut same = before;
    for (a, b) in [
        (pow2(1000), pow2(37)),
        (f64::MAX, pow2(14)),
        (-f64::MAX, f64::MAX),
        (pow2(1023), -pow2(14)),
    ] {
        assert_eq!(
            same.deposit_product(a, b),
            Err(OutOfWindow::AboveCeiling),
            "{a:e} * {b:e}"
        );
        assert_same_bits(&same, &before, "refused");
    }
    // Just under the ceiling is accepted: the top bin sits at Wide limb 32, the 1.0 is below the
    // window, and the Wide it converts to reads the overflow it is.
    assert_eq!(same.deposit_product(pow2(1000), pow2(36)), Ok(()));
    assert_eq!(same.index(), 32);
    assert_eq!(
        same.into_wide(),
        wide_product(pow2(1000), pow2(36)).unwrap()
    );
    assert_eq!(same.into_wide().to_f64(), INF);
    // The ceiling is Wide's, pair for pair: leading bits summing to 1036 on average, so about
    // half the products reach it.
    let mut rng = Rng(73);
    let mut refused = 0;
    for _ in 0..20_000 {
        let (a, b) = (rng.on_grid(400, 640), rng.on_grid(400, 640));
        let wide = wide_product(a, b).map(|_| ());
        let mut bins = Binned::<2>::ZERO;
        assert_eq!(bins.deposit_product(a, b), wide, "{a:e} * {b:e}");
        refused += usize::from(wide.is_err());
    }
    assert!((5000..15_000).contains(&refused), "{refused} refused");
    // At the top of the grid the bin wraps and the carry counts it: 3·2^1035 lands 3·2^61 in limb
    // 32, three wraps per eight deposits, and the Wide it converts to is exact.
    let (a, b) = (pow2(1000), 1.5 * pow2(36));
    let mut top = Binned::<1>::ZERO;
    for _ in 0..100_000 {
        top.deposit_product(a, b).unwrap();
    }
    assert_eq!(top.index(), 32);
    assert_eq!(top.carries(), &[37_500]);
    assert_eq!(top.limbs(), &[0]);
    assert_eq!(top.into_wide(), wide_product(a, b).unwrap() * 100_000i64);
}

#[test]
fn binned_product_is_bit_identical_under_shuffles_and_partitions() {
    fn check<const K: usize>(seed: u64) {
        let mut rng = Rng(seed);
        let mut pairs = any_pairs(&mut rng, 2000);
        let base = fold::<K>(&pairs);
        let (n, p, q) = base.specials();
        assert!(n > 0 && p != 0 && q != 0, "the pairs carry every special");
        for round in 0..200 {
            shuffle(&mut rng, &mut pairs);
            assert_same_bits(&fold::<K>(&pairs), &base, &format!("K={K} shuffle {round}"));
        }
        for parts in 1..=32 {
            let size = pairs.len().div_ceil(parts);
            let partials: Vec<Binned<K>> = pairs.chunks(size).map(fold::<K>).collect();
            let forward = partials.iter().fold(Binned::<K>::ZERO, |acc, p| acc + *p);
            let backward = partials
                .iter()
                .rev()
                .fold(Binned::<K>::ZERO, |acc, p| acc + *p);
            assert_same_bits(&forward, &base, &format!("K={K} {parts} parts forward"));
            assert_same_bits(&backward, &base, &format!("K={K} {parts} parts backward"));
        }
    }
    check::<1>(91);
    check::<3>(93);
    check::<5>(95);
}

#[test]
fn binned_product_deposit_products_is_the_pairwise_fold_or_nothing() {
    let mut rng = Rng(81);
    // Any double times one below 2^10 is under the ceiling.
    let lhs: Vec<f64> = (0..3000).map(|_| rng.any()).collect();
    let rhs: Vec<f64> = (0..3000).map(|_| rng.on_grid(-600, 10)).collect();
    let pairs: Vec<(f64, f64)> = lhs.iter().copied().zip(rhs.iter().copied()).collect();
    assert!(pairs
        .iter()
        .all(|&(x, y)| wide_product(x, y) != Err(OutOfWindow::AboveCeiling)));
    let base = fold::<3>(&pairs);
    let mut bulk = Binned::<3>::ZERO;
    assert_eq!(bulk.deposit_products(&lhs, &rhs), Ok(()));
    assert_same_bits(&bulk, &base, "deposit_products");
    assert!(!bulk.is_zero());
    // A pair at the ceiling anywhere in the slices refuses them whole: nothing is deposited.
    let mut poisoned_lhs = lhs.clone();
    let mut poisoned_rhs = rhs.clone();
    poisoned_lhs[2999] = pow2(1000);
    poisoned_rhs[2999] = -pow2(37);
    let mut same = base;
    assert_eq!(
        same.deposit_products(&poisoned_lhs, &poisoned_rhs),
        Err(OutOfWindow::AboveCeiling)
    );
    assert_same_bits(&same, &base, "nothing deposited");
    // Empty slices deposit nothing, and a slice of specials counts them.
    assert_eq!(same.deposit_products(&[], &[]), Ok(()));
    assert_same_bits(&same, &base, "empty");
    assert_eq!(
        same.deposit_products(&[INF, NAN, 0.0], &[-1.0, 2.0, INF]),
        Ok(())
    );
    let (n, p, q) = base.specials();
    assert_eq!(same.specials(), (n + 1, p - 1, q));
}

#[test]
#[should_panic(expected = "Binned: deposit_products takes two slices of one length")]
fn binned_product_deposit_products_panics_by_name_on_a_length_mismatch() {
    let mut bins = Binned::<3>::ZERO;
    let _ = bins.deposit_products(&[1.0, 2.0], &[3.0]);
}

#[test]
fn binned_product_is_within_the_bound_with_its_floor() {
    /// `|S − Σ aⱼbⱼ| ≤ n · 2^(−64(K−1)) · max|aⱼbⱼ|` on products on the grid, where `Wide`
    /// holds the exact sum. Returns whether the fold lost anything.
    fn check<const K: usize>(pairs: &[(f64, f64)], what: &str) -> bool {
        let n = pairs.len() as f64;
        let max = pairs
            .iter()
            .map(|&(a, b)| wide_product(a, b).unwrap().to_f64().abs())
            .fold(0.0, f64::max);
        let exact = pairs
            .iter()
            .fold(Wide::ZERO, |acc, &(a, b)| acc + wide_product(a, b).unwrap());
        let diff = fold::<K>(pairs).into_wide() - exact;
        let error = diff.to_f64().abs();
        let bound = n * pow2(-64 * (K as i32 - 1)) * max;
        assert!(error <= bound, "{what}: K={K} error {error:e} > {bound:e}");
        !diff.is_zero()
    }
    let mut rng = Rng(101);
    let uniform: Vec<(f64, f64)> = (0..5000)
        .map(|_| (rng.on_grid(-480, 480), rng.on_grid(-480, 480)))
        .collect();
    // A unit product fixes the window, and every other product's 106 bits sit at its edge with
    // alternating signs, so each one loses part of itself on both sides of zero.
    let edge: Vec<(f64, f64)> = core::iter::once((1.0, 1.0))
        .chain((0..5000).map(|_| (rng.on_grid(-100, -90), rng.on_grid(-100, -90))))
        .collect();
    let mut lossy = 0;
    for (pairs, what) in [(&uniform, "uniform"), (&edge, "edge")] {
        lossy += usize::from(check::<1>(pairs, what));
        lossy += usize::from(check::<2>(pairs, what));
        lossy += usize::from(check::<3>(pairs, what));
        lossy += usize::from(check::<5>(pairs, what));
    }
    assert!(lossy >= 5, "{lossy} lossy folds exercise the bound");

    // The floor: products straddling the unit lose their bits below it, which the unfloored term
    // does not cover, and `n · 2^-1074` does. `Exact` holds the exact sum of this narrow band.
    let band: Vec<(f64, f64)> = (0..1000)
        .map(|_| (rng.on_grid(-540, -536), rng.on_grid(-540, -536)))
        .collect();
    let exact = band
        .iter()
        .fold(Exact::ZERO, |acc, &(a, b)| acc + Exact::product(a, b));
    let got = Exact::try_from(&fold::<3>(&band).into_wide()).unwrap();
    let error = (got - exact).to_f64().abs();
    let n = band.len() as f64;
    assert!(error > 0.0, "the band's sub-unit bits are lost");
    assert!(
        error >= pow2(-1074),
        "{error:e}: at least one unit, which the unfloored term of 2^-128 · 2^-1070 never allows"
    );
    assert!(error <= n * pow2(-1074), "{error:e} > n · 2^-1074");
}
