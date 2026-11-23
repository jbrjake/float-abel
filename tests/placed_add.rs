//! The one-value deposit of `Wide` and `Fixed`, which adds or subtracts the shifted mantissa in
//! the limbs it lands on and lets the carry or the borrow run until it stops. Every test is named
//! `placed_add_*` so the task verify can filter on it.
//!
//! Oracle: the three-pass form written out here on a full-width term (place the mantissa, negate
//! the whole term when the value is negative, add the whole width with carry), which is the
//! definition of the sum in two's complement. States enter through `from_parts` and `from_limbs`
//! so runs of all-ones and all-zero limbs, where a carry or a borrow travels the width, are common.

use float_abel::{Fixed, Fixed256, OutOfWindow, Wide};

use crate::support::{pow2, rounds, Rng};

/// The sign, odd mantissa and exponent of a finite double: `(-1)^negative · mant · 2^exp`, the
/// lowest set bit at `exp`, so a product of two is on a grid iff its exponent sum is.
fn decompose(x: f64) -> (bool, u128, i32) {
    let bits = x.to_bits();
    let biased = ((bits >> 52) & 0x7ff) as i32;
    let frac = u128::from(bits & ((1 << 52) - 1));
    let (mant, exp) = if biased == 0 {
        (frac, -1074)
    } else {
        (frac | 1 << 52, biased - 1075)
    };
    if mant == 0 {
        return (bits >> 63 != 0, 0, 0);
    }
    let tz = mant.trailing_zeros();
    (bits >> 63 != 0, mant >> tz, exp + tz as i32)
}

/// `limbs += (-1)^negative · mant · 2^shift` in three full-width passes.
fn three_pass<const N: usize>(limbs: &mut [u64; N], negative: bool, mant: u128, shift: u32) {
    let mut term = [0u64; N];
    for bit in 0..128 {
        if mant >> bit & 1 == 1 {
            let at = shift as usize + bit;
            term[at / 64] |= 1 << (at % 64);
        }
    }
    if negative {
        let mut carry = true;
        for t in &mut term {
            let (v, c) = (!*t).overflowing_add(u64::from(carry));
            *t = v;
            carry = c;
        }
    }
    let mut carry = false;
    for (l, t) in limbs.iter_mut().zip(term) {
        let (v, c1) = l.overflowing_add(t);
        let (v, c2) = v.overflowing_add(u64::from(carry));
        *l = v;
        carry = c1 | c2;
    }
}

/// A state whose limbs are each all zeros, all ones or random, sign-extended through the top
/// limb so that no deposit inside the window can exhaust the headroom.
fn state<const N: usize>(rng: &mut Rng) -> [u64; N] {
    let mut limbs = [0u64; N];
    for l in &mut limbs {
        *l = match rng.next() % 4 {
            0 => 0,
            1 | 2 => u64::MAX,
            _ => rng.next(),
        };
    }
    limbs[N - 1] = if rng.next() & 1 == 0 { 0 } else { u64::MAX };
    limbs
}

#[test]
fn placed_add_wide_deposit_lands_the_three_pass_limbs() {
    let mut rng = Rng(0x51ed_270b_a5e1_f00d);
    let mut landed = 0usize;
    for _ in 0..rounds(20_000) {
        let limbs: [u64; 34] = state(&mut rng);
        let x = rng.finite();
        let (negative, mant, exp) = decompose(x);
        let mut want = limbs;
        three_pass(&mut want, negative, mant, (exp + 1074) as u32);
        let mut got = Wide::from_parts(limbs, [0; 3]);
        got.deposit(x);
        landed += usize::from(mant != 0);
        assert_eq!(got.limbs(), &want, "{x:e} into {limbs:x?}");
    }
    assert!(
        landed >= rounds(20_000) * 9 / 10,
        "{landed} deposits landed"
    );
}

#[test]
fn placed_add_wide_product_lands_the_three_pass_limbs() {
    let mut rng = Rng(0x0dd_ba11_5eed_cafe);
    let (mut landed, mut refused) = (0usize, 0usize);
    for _ in 0..rounds(20_000) {
        let limbs: [u64; 34] = state(&mut rng);
        // Factors within 2^±500 keep most products inside the window; the rest are refusals.
        let (a, b) = (rng.on_grid(-560, 500), rng.on_grid(-560, 500));
        let ((na, ma, ea), (nb, mb, eb)) = (decompose(a), decompose(b));
        let mut got = Wide::from_parts(limbs, [0; 3]);
        match got.deposit_product(a, b) {
            Ok(()) => {
                let mut want = limbs;
                three_pass(&mut want, na != nb, ma * mb, (ea + eb + 1074) as u32);
                assert_eq!(got.limbs(), &want, "{a:e} * {b:e} into {limbs:x?}");
                landed += 1;
            }
            Err(e) => {
                assert_eq!(e, OutOfWindow::OffGrid, "{a:e} * {b:e}");
                assert_eq!(got.limbs(), &limbs, "a refusal changed the state");
                refused += 1;
            }
        }
    }
    assert!(landed >= rounds(20_000) / 2, "{landed} products landed");
    assert!(refused >= 1 || cfg!(miri), "no product was refused");
}

/// `rounds` deposits and `rounds` products into `Fixed<LSB, LIMBS>` against the three-pass form,
/// with doubles drawn by `draw`. Returns how many landed.
fn fixed_sweep<const LSB: i32, const LIMBS: usize>(
    rng: &mut Rng,
    rounds: usize,
    draw: impl Fn(&mut Rng) -> f64,
) -> usize {
    let mut landed = 0;
    for _ in 0..rounds {
        let limbs: [u64; LIMBS] = state(rng);
        let x = draw(rng);
        let (negative, mant, exp) = decompose(x);
        let mut want = limbs;
        three_pass(&mut want, negative, mant, (exp - LSB) as u32);
        let mut got = Fixed::<LSB, LIMBS>::from_limbs(limbs);
        got.try_deposit(x)
            .unwrap_or_else(|e| panic!("{x:e} refused: {e}"));
        assert_eq!(got.limbs(), &want, "{x:e} into {limbs:x?}");
        landed += usize::from(mant != 0);

        let (a, b) = (draw(rng), draw(rng));
        let ((na, ma, ea), (nb, mb, eb)) = (decompose(a), decompose(b));
        let mut got = Fixed::<LSB, LIMBS>::from_limbs(limbs);
        if got.deposit_product(a, b).is_ok() {
            let mut want = limbs;
            if ma * mb != 0 {
                three_pass(&mut want, na != nb, ma * mb, (ea + eb - LSB) as u32);
                landed += 1;
            }
            assert_eq!(got.limbs(), &want, "{a:e} * {b:e} into {limbs:x?}");
        } else {
            assert_eq!(got.limbs(), &limbs, "a refusal changed the state");
        }
    }
    landed
}

#[test]
fn placed_add_fixed_lands_the_three_pass_limbs() {
    let mut rng = Rng(0xf1ed_0000_2148_0068);
    // The product window: every finite double and every product of two is inside it.
    let n = rounds(5_000);
    let landed = fixed_sweep::<-2148, 68>(&mut rng, n, Rng::finite);
    assert!(landed >= n * 19 / 10, "{landed} of {} landed", 2 * n);
    // Fixed256: values whose products stay on the 2^-96 grid and under the 2^95 ceiling.
    let n = rounds(20_000);
    let landed = fixed_sweep::<-96, 4>(&mut rng, n, |r| r.on_grid(4, 40));
    assert_eq!(landed, 2 * n);
}

#[test]
fn placed_add_carry_and_borrow_travel_the_whole_width() {
    let unit = 5e-324;
    // -1 unit is all ones: one unit more carries through every limb and out of the top.
    let mut w = Wide::from_parts([u64::MAX; 34], [0; 3]);
    w.deposit(unit);
    assert_eq!(w.limbs(), &[0; 34]);
    // Zero less one unit borrows through every limb.
    w.deposit(-unit);
    assert_eq!(w.limbs(), &[u64::MAX; 34]);
    // A carry that stops part-way leaves the limbs above it alone.
    let mut limbs = [u64::MAX; 34];
    limbs[20] = 7;
    limbs[33] = 0;
    let mut w = Wide::from_parts(limbs, [0; 3]);
    w.deposit(unit);
    let mut want = [0u64; 34];
    want[20] = 8;
    want[21..33].fill(u64::MAX);
    assert_eq!(w.limbs(), &want);
    // The top of the window: a term in the highest limbs a product reaches, either sign.
    let mut f = Fixed256::from_limbs([0; 4]);
    f.try_deposit(-pow2(94)).unwrap();
    assert_eq!(f.limbs(), &[0, 0, (1u64 << 62).wrapping_neg(), u64::MAX]);
    f.try_deposit(pow2(94)).unwrap();
    assert_eq!(f.limbs(), &[0; 4]);
}

#[test]
#[should_panic(expected = "headroom")]
fn placed_add_wide_deposit_past_the_positive_end_is_the_named_panic() {
    let mut limbs = [u64::MAX; 34];
    limbs[33] = u64::MAX >> 1;
    let mut w = Wide::from_parts(limbs, [0; 3]);
    w.deposit(5e-324);
}

#[test]
#[should_panic(expected = "headroom")]
fn placed_add_wide_deposit_past_the_negative_end_is_the_named_panic() {
    let mut limbs = [0u64; 34];
    limbs[33] = 1 << 63;
    let mut w = Wide::from_parts(limbs, [0; 3]);
    w.deposit(-5e-324);
}

#[test]
#[should_panic(expected = "headroom")]
fn placed_add_fixed_deposit_past_the_positive_end_is_the_named_panic() {
    let mut f = Fixed256::from_limbs([u64::MAX, u64::MAX, u64::MAX, u64::MAX >> 1]);
    f.try_deposit(pow2(-96)).unwrap();
}

#[test]
#[should_panic(expected = "headroom")]
fn placed_add_fixed_deposit_past_the_negative_end_is_the_named_panic() {
    let mut f = Fixed256::from_limbs([0, 0, 0, 1 << 63]);
    f.try_deposit(-pow2(-96)).unwrap();
}
