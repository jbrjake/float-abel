//! Conversions between the exact tiers, DESIGN.md § Operations and conversions. `Wide: From<Fixed>`
//! is exact and total, pinned at compile time to the `(LSB, LIMBS)` pairs it can be exact for;
//! `Exact: TryFrom<&Wide>` and `Exact: TryFrom<&Fixed>` refuse past the 127-bit width by name;
//! `Wide::try_from(&Exact)` refuses below the unit and at the ceiling by name, exactly where
//! `Wide::deposit_product` would. Every round trip is checked bit for bit against the state the
//! same deposits build in the target tier directly.

use float_abel::{Exact, Fixed256, OutOfWindow, PastCap, Wide};

use crate::support::{pow2, Rng};

/// `n` on-grid doubles with their leading bits in `[lo, hi)`, each paired with whether it is a
/// retraction, so a multiset exercises both signs of the limbs and of the counts.
fn multiset(rng: &mut Rng, lo: i32, hi: i32, n: usize) -> Vec<(f64, bool)> {
    (0..n)
        .map(|_| (rng.on_grid(lo, hi), rng.next() % 4 == 0))
        .collect()
}

fn fixed_of(set: &[(f64, bool)]) -> Fixed256 {
    let mut f = Fixed256::ZERO;
    for &(x, retract) in set {
        let term = Fixed256::try_from(x).unwrap();
        if retract {
            f -= term;
        } else {
            f += term;
        }
    }
    f
}

fn wide_of(set: &[(f64, bool)]) -> Wide {
    let mut w = Wide::ZERO;
    for &(x, retract) in set {
        if retract {
            w -= Wide::from(x);
        } else {
            w += Wide::from(x);
        }
    }
    w
}

fn exact_of(set: &[(f64, bool)]) -> Exact {
    let mut e = Exact::ZERO;
    for &(x, retract) in set {
        if retract {
            e -= Exact::from(x);
        } else {
            e += Exact::from(x);
        }
    }
    e
}

#[test]
fn convert_fixed256_states_round_trip_through_wide_and_exact_bit_for_bit() {
    let mut rng = Rng(0xc0_4e17);
    let (mut negative, mut positive) = (0, 0);
    for _ in 0..64 {
        // Leading bits in [2^0, 2^60) over full 53-bit fractions: the sum of 48 spans at most 119
        // bits, inside Exact's width, and lies inside Fixed256's window.
        let count = 1 + (rng.next() % 48) as usize;
        let set = multiset(&mut rng, 0, 60, count);
        let (fixed, wide, exact) = (fixed_of(&set), wide_of(&set), exact_of(&set));
        match fixed.to_f64() {
            x if x < 0.0 => negative += 1,
            x if x > 0.0 => positive += 1,
            _ => {}
        }
        let via_wide = Wide::from(fixed);
        assert_eq!(via_wide, wide, "{set:?}");
        assert_eq!(via_wide.limbs(), wide.limbs());
        assert_eq!(Exact::try_from(&fixed), Ok(exact), "{set:?}");
        assert_eq!(Exact::try_from(&wide), Ok(exact), "{set:?}");
        assert_eq!(Wide::try_from(&exact), Ok(wide), "{set:?}");
        assert_eq!(
            Wide::try_from(&Exact::try_from(&fixed).unwrap()),
            Ok(via_wide)
        );
    }
    assert!(
        negative >= 8 && positive >= 8,
        "{negative} negative sums, {positive} positive: both sign extensions must be exercised"
    );
}

#[test]
fn convert_exact_takes_a_fixed256_spanning_127_bits_and_refuses_128_as_width() {
    // 2^-44 + 2^-96 sets the lowest bit of Fixed256's grid; 2^30 above it spans 127 bits exactly.
    let floor = f64::from_bits(((-44 + 1023) as u64) << 52 | 1);
    assert_eq!(floor, pow2(-44) + pow2(-96));
    let mut at = Fixed256::ZERO;
    at.try_deposit(floor).unwrap();
    at.try_deposit(pow2(30)).unwrap();
    let want = Exact::from(floor) + Exact::from(pow2(30));
    assert_eq!(Exact::try_from(&at), Ok(want));
    assert_eq!(Exact::try_from(&-at), Ok(-want));
    assert_eq!(Exact::try_from(&Wide::from(at)), Ok(want));
    assert_eq!(Wide::try_from(&want), Ok(Wide::from(at)));

    let mut past = Fixed256::ZERO;
    past.try_deposit(floor).unwrap();
    past.try_deposit(pow2(31)).unwrap();
    assert_eq!(Exact::try_from(&past), Err(PastCap::Width));
    assert_eq!(Exact::try_from(&-past), Err(PastCap::Width));
    assert_eq!(Exact::try_from(&Wide::from(past)), Err(PastCap::Width));
    // Wide holds it regardless: the width is Exact's cap, not Wide's.
    assert_eq!(Wide::from(past), Wide::from(floor) + Wide::from(pow2(31)));
}

#[test]
fn convert_wide_states_with_specials_round_trip_through_exact() {
    let mut rng = Rng(0x5bec_1a15);
    let (mut with_specials, mut negative_counts) = (0, 0);
    for _ in 0..64 {
        // A 60-binade band anywhere in the normal range, so the subnormal unit and the top of the
        // range both appear across the run, and the sum of 48 stays inside Exact's width.
        let lo = -1022 + (rng.next() % (963 + 1022 + 1)) as i32;
        let count = 1 + (rng.next() % 48) as usize;
        let mut set = multiset(&mut rng, lo, lo + 60, count);
        for _ in 0..rng.next() % 4 {
            let special = [f64::NAN, f64::INFINITY, f64::NEG_INFINITY][(rng.next() % 3) as usize];
            set.push((special, rng.next() % 2 == 0));
        }
        let (wide, exact) = (wide_of(&set), exact_of(&set));
        let (n, p, q) = wide.specials();
        if (n, p, q) != (0, 0, 0) {
            with_specials += 1;
        }
        if n < 0 || p < 0 || q < 0 {
            negative_counts += 1;
        }
        assert_eq!(Exact::try_from(&wide), Ok(exact), "{set:?}");
        assert_eq!(exact.specials(), (n as i32, p as i32, q as i32));
        let back = Wide::try_from(&exact).unwrap();
        assert_eq!(back, wide, "{set:?}");
        assert_eq!(back.limbs(), wide.limbs());
        assert_eq!(back.specials(), (n, p, q));
    }
    assert!(
        with_specials >= 16 && negative_counts >= 4,
        "{with_specials} states with specials, {negative_counts} with a negative count"
    );
}

#[test]
fn convert_exact_refuses_a_wide_counter_outside_i32_as_count_and_keeps_one_inside() {
    let zero = [0u64; 34];
    let past = i64::from(i32::MAX) + 1;
    assert_eq!(
        Exact::try_from(&Wide::from_parts(zero, [past, 0, 0])),
        Err(PastCap::Count)
    );
    assert_eq!(
        Exact::try_from(&Wide::from_parts(zero, [0, -past, 0])),
        Err(PastCap::Count)
    );
    // i32::MIN is kept out so that negation stays total.
    assert_eq!(
        Exact::try_from(&Wide::from_parts(zero, [0, 0, i64::from(i32::MIN)])),
        Err(PastCap::Count)
    );
    let edge = [i64::from(i32::MAX), i64::from(i32::MIN) + 1, -7];
    let e = Exact::try_from(&Wide::from_parts(zero, edge)).unwrap();
    assert_eq!(e.specials(), (i32::MAX, i32::MIN + 1, -7));
    assert_eq!(Wide::try_from(&e), Ok(Wide::from_parts(zero, edge)));
    assert_eq!(
        Wide::try_from(&-e),
        Ok(Wide::from_parts(zero, [-edge[0], -edge[1], 7]))
    );
}

#[test]
fn convert_wide_try_from_exact_product_matches_deposit_product_on_random_factors() {
    let mut rng = Rng(0x9d0d);
    let (mut ok, mut off_grid, mut above) = (0, 0, 0);
    for _ in 0..4096 {
        let (a, b) = (rng.any(), rng.any());
        let mut w = Wide::ZERO;
        let want = w.deposit_product(a, b).map(|()| w);
        match want {
            Ok(_) => ok += 1,
            Err(OutOfWindow::OffGrid) => off_grid += 1,
            Err(OutOfWindow::AboveCeiling) => above += 1,
            Err(OutOfWindow::NotFinite) => unreachable!("Wide counts a special factor"),
        }
        assert_eq!(Wide::try_from(&Exact::product(a, b)), want, "{a:e} × {b:e}");
    }
    assert!(
        ok >= 256 && off_grid >= 64 && above >= 64,
        "{ok} accepted, {off_grid} off the grid, {above} above the ceiling"
    );
}

#[test]
fn convert_wide_try_from_exact_refuses_by_name_past_the_range_and_accepts_at_the_edge() {
    // The unit: 2^-1074 is on the grid, and the product 2^-1000 · 2^-75 lies one bit below it.
    let unit = Exact::from(pow2(-1074));
    assert_eq!(Wide::try_from(&unit), Ok(Wide::from(pow2(-1074))));
    assert_eq!(Wide::try_from(&-unit), Ok(-Wide::from(pow2(-1074))));
    let below = Exact::product(pow2(-1000), pow2(-75));
    assert_eq!(Wide::try_from(&below), Err(OutOfWindow::OffGrid));
    assert_eq!(Wide::try_from(&-below), Err(OutOfWindow::OffGrid));

    // The ceiling is 2^1037: (2^63 − 1) · 2^974 and 2^1000 · 2^36 have their leading bit at
    // 2^1036 and pass; 2^1037 is refused whatever the sign.
    assert_eq!(Wide::CEILING_EXP, 1037);
    let under = Exact::from(pow2(974)) * i64::MAX;
    assert_eq!(Wide::try_from(&under), Ok(Wide::from(pow2(974)) * i64::MAX));
    let mut at = Wide::ZERO;
    assert_eq!(at.deposit_product(pow2(1000), pow2(36)), Ok(()));
    assert_eq!(
        Wide::try_from(&Exact::product(pow2(1000), pow2(36))),
        Ok(at)
    );
    assert_eq!(
        Wide::try_from(&Exact::product(pow2(1000), pow2(37))),
        Err(OutOfWindow::AboveCeiling)
    );
    let over = Exact::from(pow2(975)) * i64::MAX;
    assert_eq!(Wide::try_from(&over), Err(OutOfWindow::AboveCeiling));
    assert_eq!(Wide::try_from(&-over), Err(OutOfWindow::AboveCeiling));

    // A refused finite part refuses the whole state: the counts never come through alone.
    let mixed = Exact::from(f64::NAN) + Exact::product(pow2(1000), pow2(37));
    assert_eq!(Wide::try_from(&mixed), Err(OutOfWindow::AboveCeiling));
    // The zero and a bare special pass.
    assert_eq!(Wide::try_from(&Exact::ZERO), Ok(Wide::ZERO));
    assert_eq!(
        Wide::try_from(&Exact::from(f64::NEG_INFINITY)),
        Ok(Wide::from(f64::NEG_INFINITY))
    );
}
