//! `Wide`: 34 limbs in units of 2^-1074 plus three signed special counters. The carrier and its
//! reads: total `From<f64>`, bitrep's limb layout, products, division and rounding at both ends of
//! the double range. Every test is named `wide_*` so the task verify can filter on it;
//! `tests/wide_group.rs` holds the group laws, folds, order and projection.
//!
//! Oracles: bitrep's committed conformance vectors and bitrep itself (the same multiset must land
//! the same 272 limb bytes and the same rounded value); IEEE `a + b`, `a * b`, `a.mul_add(b, c)`
//! and `a / n` for `n < 2^53`, each the correctly rounded value of the exact two-term expression;
//! and the NaN algebra of DESIGN.md § NaN and ±∞ on the counters.

use float_abel::{OutOfWindow, Wide};

use crate::support::Rng;

const _: () = assert!(core::mem::size_of::<Wide>() == 296);

const NAN: f64 = f64::NAN;
const INF: f64 = f64::INFINITY;
const NINF: f64 = f64::NEG_INFINITY;

fn w(x: f64) -> Wide {
    Wide::from(x)
}

fn product(a: f64, b: f64) -> Wide {
    let mut p = Wide::ZERO;
    p.deposit_product(a, b)
        .unwrap_or_else(|e| panic!("{a:e} * {b:e} refused: {e}"));
    p
}

/// The limbs as bitrep encodes them: little-endian `u64`s, least significant limb first.
fn limb_bytes(w: &Wide) -> Vec<u8> {
    w.limbs().iter().flat_map(|l| l.to_le_bytes()).collect()
}

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

/// bitrep's flag byte: bit 0 saw NaN, bit 1 saw +∞, bit 2 saw −∞. Insertions alone leave every
/// counter at or above zero, so a set flag is a positive counter.
fn flags_of(w: &Wide) -> u8 {
    let (n, p, q) = w.specials();
    u8::from(n > 0) | u8::from(p > 0) << 1 | u8::from(q > 0) << 2
}

#[test]
fn wide_size_and_constants() {
    assert_eq!(core::mem::size_of::<Wide>(), 296);
    assert_eq!(Wide::LIMBS, 34);
    assert_eq!(Wide::LSB, -1074);
    // Products are accepted below 2^1037: 2176 bits less a sign bit and 64 of headroom, over 2^-1074.
    assert_eq!(Wide::CEILING_EXP, 1037);
    assert_eq!(Wide::ZERO.limbs(), &[0u64; 34]);
    assert_eq!(Wide::ZERO.specials(), (0, 0, 0));
    assert_eq!(Wide::default(), Wide::ZERO);
    assert!(Wide::ZERO.is_zero());
    assert!(Wide::ZERO.is_finite());
}

#[test]
fn wide_reproduces_every_bitrep_conformance_vector() {
    let v: serde_json::Value =
        serde_json::from_str(include_str!("data/bitrep/vectors.json")).unwrap();
    assert_eq!(
        v["format"], 1,
        "a new vector format: re-read bitrep's FORMAT.md"
    );
    let cases = v["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 10);
    let mut finite_inputs = 0;
    for c in cases {
        let name = c["name"].as_str().unwrap();
        let inputs: Vec<f64> = c["input_bits"]
            .as_array()
            .unwrap()
            .iter()
            .map(|h| f64::from_bits(u64::from_str_radix(h.as_str().unwrap(), 16).unwrap()))
            .collect();
        let state = unhex(c["state_hex"].as_str().unwrap());
        assert_eq!(state.len(), 272 + 1 + 8, "{name}");
        let value = u64::from_str_radix(c["value_bits"].as_str().unwrap(), 16).unwrap();
        finite_inputs += inputs.iter().filter(|x| x.is_finite()).count();

        let forward: Wide = inputs.iter().map(|&x| w(x)).sum();
        assert_eq!(limb_bytes(&forward), &state[..272], "{name}: limbs");
        assert_eq!(forward.to_f64().to_bits(), value, "{name}: value");
        assert_eq!(flags_of(&forward), state[272], "{name}: specials");

        // The same multiset backwards, and as two shards merged, lands the same bits.
        let backward: Wide = inputs.iter().rev().map(|&x| w(x)).sum();
        assert_eq!(backward, forward, "{name}: order");
        let cut = inputs.len() / 3;
        let left: Wide = inputs[..cut].iter().map(|&x| w(x)).sum();
        let right: Wide = inputs[cut..].iter().map(|&x| w(x)).sum();
        assert_eq!(left + right, forward, "{name}: shards");
    }
    assert!(finite_inputs > 1_000, "{finite_inputs} finite inputs");
}

#[test]
fn wide_limbs_and_value_match_bitrep_on_random_multisets() {
    let mut rng = Rng(10);
    for round in 0..300 {
        let len = 1 + (rng.next() % 400) as usize;
        let xs: Vec<f64> = (0..len)
            .map(|_| match round % 3 {
                0 => rng.any(),
                1 => rng.finite(),
                _ => rng.on_grid(-600, 600),
            })
            .collect();
        let cut = len / 2;
        let (mut ours, mut ours_left) = (Wide::ZERO, Wide::ZERO);
        let (mut theirs, mut theirs_left) = (bitrep::SumF64::new(), bitrep::SumF64::new());
        for (i, &x) in xs.iter().enumerate() {
            if i < cut {
                ours_left.deposit(x);
                theirs_left.add(x);
            } else {
                ours.deposit(x);
                theirs.add(x);
            }
        }
        ours += ours_left;
        theirs.merge(&theirs_left);
        let bytes = theirs.to_bytes();
        assert_eq!(limb_bytes(&ours), &bytes[..272], "round {round}");
        assert_eq!(flags_of(&ours), bytes[272], "round {round}");
        assert_eq!(
            ours.to_f64().to_bits(),
            theirs.value().to_bits(),
            "round {round}: {:e} vs {:e}",
            ours.to_f64(),
            theirs.value()
        );
    }
}

#[test]
fn wide_from_f64_is_total_and_single_deposits_read_back_bit_for_bit() {
    let mut rng = Rng(11);
    for _ in 0..50_000 {
        let x = rng.finite();
        let s = w(x);
        assert_eq!(s.to_f64().to_bits(), x.to_bits(), "{x:e}");
        assert_eq!(s.try_to_f64(), Some(x));
        assert!(s.is_finite());
        assert_eq!(s.is_zero(), x == 0.0);
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
    ] {
        assert_eq!(w(x).to_f64().to_bits(), x.to_bits(), "{x:e}");
    }
    // Both zeros are the one zero, which reads +0.0.
    assert_eq!(w(0.0), Wide::ZERO);
    assert_eq!(w(-0.0), Wide::ZERO);
    assert!(w(-0.0).is_zero());
    assert_eq!(w(-0.0).to_f64().to_bits(), 0.0f64.to_bits());
    assert_eq!((w(-0.0) + w(-0.0)).to_f64().to_bits(), 0.0f64.to_bits());
    assert_eq!((w(1.5) - w(1.5)).to_f64().to_bits(), 0.0f64.to_bits());
    // Specials are counted, never refused, and never touch the limbs.
    assert!(w(NAN).to_f64().is_nan());
    assert_eq!(w(NAN).specials(), (1, 0, 0));
    assert_eq!(w(INF).to_f64(), INF);
    assert_eq!(w(INF).specials(), (0, 1, 0));
    assert_eq!(w(NINF).to_f64(), NINF);
    assert_eq!(w(NINF).specials(), (0, 0, 1));
    for s in [w(NAN), w(INF), w(NINF)] {
        assert!(!s.is_finite());
        assert!(!s.is_zero());
        assert_eq!(s.limbs(), &[0; 34]);
    }
    // A NaN is a NaN whatever its payload or sign.
    assert_eq!(w(f64::from_bits(0xFFF8_0000_0000_0001)), w(NAN));
    assert_eq!(w(f64::from_bits(0x7FF0_0000_0000_0001)), w(NAN));
}

#[test]
fn wide_pair_sums_round_like_ieee_addition() {
    let mut rng = Rng(12);
    for i in 0..100_000 {
        let (a, b) = if i % 2 == 0 {
            (rng.finite(), rng.finite())
        } else {
            (rng.on_grid(-1000, 1000), rng.on_grid(-1000, 1000))
        };
        let sum = w(a) + w(b);
        let ieee = a + b;
        assert_eq!(sum.to_f64().to_bits(), ieee.to_bits(), "{a:e} + {b:e}");
        assert_eq!(sum.try_to_f64().is_some(), sum == w(ieee), "{a:e} + {b:e}");
    }
    // The classic.
    assert_eq!((w(0.1) + w(0.2)).to_f64(), 0.1 + 0.2);
    assert_eq!((w(0.1) + w(0.2)).try_to_f64(), None);
    // A cancellation across 400 orders of magnitude keeps the small term, in any order.
    let small = 1e-300;
    assert_eq!((w(1e100) + w(-1e100) + w(small)).try_to_f64(), Some(small));
    assert_eq!((w(1e100) + w(small) + w(-1e100)).try_to_f64(), Some(small));
    assert_eq!(
        (w(small) + w(1e100) + w(-1e100)).to_f64().to_bits(),
        small.to_bits()
    );
    assert_eq!(
        (1e100 + small) + -1e100,
        0.0,
        "the f64 fold loses the small term in this order"
    );
}

#[test]
fn wide_products_round_like_ieee_multiply_and_fma() {
    let mut rng = Rng(13);
    for _ in 0..50_000 {
        // Leading bits within ±480: the product's lowest bit is at or above 2^-1064, on the grid,
        // and its magnitude is below 2^962, under the ceiling.
        let (a, b) = (rng.on_grid(-480, 480), rng.on_grid(-480, 480));
        let c = rng.on_grid(-1000, 1000);
        let p = product(a, b);
        assert_eq!(p.to_f64().to_bits(), (a * b).to_bits(), "{a:e} * {b:e}");
        assert_eq!(
            (p + w(c)).to_f64().to_bits(),
            a.mul_add(b, c).to_bits(),
            "{a:e} * {b:e} + {c:e}"
        );
    }
    // A product IEEE would flush to a subnormal or zero is exact while its lowest bit reaches the
    // unit.
    assert_eq!(
        product(2f64.powi(-537), 2f64.powi(-537)).try_to_f64(),
        Some(5e-324)
    );
    assert_eq!(
        product(3.0 * 2f64.powi(-537), 2f64.powi(-537)).try_to_f64(),
        Some(f64::from_bits(3))
    );
    // 2^-1070 is 16 units; `powi(-1070)` would reciprocate an infinite 2^1070 and return 0.
    assert_eq!(
        product(2f64.powi(-600), 2f64.powi(-470)).try_to_f64(),
        Some(f64::from_bits(16))
    );
    // Below the unit or at the ceiling the product is refused by name, and the state is untouched.
    let mut p = w(1.0);
    assert_eq!(p.deposit_product(5e-324, 0.5), Err(OutOfWindow::OffGrid));
    assert_eq!(
        p.deposit_product(2f64.powi(-537), 1.5 * 2f64.powi(-537)),
        Err(OutOfWindow::OffGrid)
    );
    assert_eq!(
        p.deposit_product(2f64.powi(1000), 2f64.powi(37)),
        Err(OutOfWindow::AboveCeiling)
    );
    assert_eq!(
        p.deposit_product(f64::MAX, 2f64.powi(14)),
        Err(OutOfWindow::AboveCeiling)
    );
    assert_eq!(p, w(1.0));
    // Just under the ceiling is accepted, and reads as the overflow it is.
    assert_eq!(p.deposit_product(2f64.powi(1000), 2f64.powi(36)), Ok(()));
    assert_eq!(p.to_f64(), INF);
    assert_eq!(product(f64::MAX, 2f64.powi(13)).to_f64(), INF);
    assert_eq!(
        (product(f64::MAX, 2f64.powi(13)) - product(f64::MAX, 2f64.powi(13))),
        Wide::ZERO
    );
    // A zero factor makes the exact product zero, whatever the other factor: bilinearity, not
    // IEEE, so even 0 · ∞ and 0 · NaN deposit nothing.
    let mut z = w(1.0);
    assert_eq!(z.deposit_product(0.0, f64::MAX), Ok(()));
    assert_eq!(z.deposit_product(-0.0, 5e-324), Ok(()));
    assert_eq!(z.deposit_product(0.0, INF), Ok(()));
    assert_eq!(z.deposit_product(NINF, -0.0), Ok(()));
    assert_eq!(z.deposit_product(NAN, 0.0), Ok(()));
    assert_eq!(z, w(1.0));
}

#[test]
fn wide_products_with_special_factors_follow_the_nan_algebra() {
    // NaN absorbs: one NaN count, whichever side it is on and whatever the other nonzero factor.
    for b in [2.0, -2.0, 5e-324, INF, NINF, NAN] {
        assert_eq!(product(NAN, b).specials(), (1, 0, 0), "NaN * {b}");
        assert_eq!(product(b, NAN).specials(), (1, 0, 0), "{b} * NaN");
        assert!(product(NAN, b).to_f64().is_nan());
    }
    // The sign rule on two infinities.
    assert_eq!(product(INF, INF).specials(), (0, 1, 0));
    assert_eq!(product(NINF, NINF).specials(), (0, 1, 0));
    assert_eq!(product(INF, NINF).specials(), (0, 0, 1));
    assert_eq!(product(NINF, INF).specials(), (0, 0, 1));
    // A finite factor scales the count by its sign alone; a negative one flips the infinity
    // through the projection rule.
    assert_eq!(product(INF, 2.5).specials(), (0, 1, 0));
    assert_eq!(product(INF, -2.5).specials(), (0, -1, 0));
    assert_eq!(product(INF, -2.5).to_f64(), NINF);
    assert_eq!(product(-2.5, NINF).specials(), (0, 0, -1));
    assert_eq!(product(-2.5, NINF).to_f64(), INF);
    assert_eq!(product(5e-324, NINF).specials(), (0, 0, 1));
    assert_eq!(product(f64::MAX, INF).specials(), (0, 1, 0));
    // The counts cancel like any other term, so a retraction restores the finite sum.
    let mut s = w(1.0);
    s.deposit_product(INF, -3.0).unwrap();
    assert_eq!(s.to_f64(), NINF);
    s -= product(INF, -3.0);
    assert_eq!(s, w(1.0));
    // Bilinearity, not IEEE: (+∞)·(−1) + (+∞) is (−1 + 1)·(+∞) = 0 · ∞ = 0.
    assert_eq!(product(INF, -1.0) + w(INF), Wide::ZERO);
    assert_ne!(
        product(INF, -1.0),
        w(NINF),
        "a negated +∞ count is not a −∞ count, though both read −∞"
    );
    assert_eq!(product(INF, -1.0).to_f64(), w(NINF).to_f64());
}

#[test]
fn wide_div_f64_rounds_like_ieee_division() {
    let mut rng = Rng(17);
    for i in 0..50_000 {
        let x = if i % 2 == 0 {
            rng.finite()
        } else {
            rng.on_grid(-1000, 1000)
        };
        let n = (rng.next() >> 11).max(1); // < 2^53, so `n as f64` is exact
        assert_eq!(
            w(x).div_f64(n).to_bits(),
            (x / n as f64).to_bits(),
            "{x:e} / {n}"
        );
    }
    // The mean of ten exact tenths is exactly a tenth; the f64 fold lands elsewhere.
    let tenths: Wide = (0..10).map(|_| w(0.1)).sum();
    assert_eq!(tenths.try_to_f64(), None);
    assert_eq!(tenths.to_f64(), 1.0);
    assert_eq!(tenths.div_f64(10), 0.1);
    assert_ne!((0..10).map(|_| 0.1f64).sum::<f64>(), 1.0);
    // Division by zero follows IEEE, and a special projects before any division.
    assert!(Wide::ZERO.div_f64(0).is_nan());
    assert_eq!(w(1.0).div_f64(0), INF);
    assert_eq!(w(-1.0).div_f64(0), NINF);
    assert_eq!(Wide::ZERO.div_f64(7).to_bits(), 0.0f64.to_bits());
    assert_eq!(w(INF).div_f64(3), INF);
    assert_eq!(w(NINF).div_f64(3), NINF);
    assert!(w(NAN).div_f64(3).is_nan());
    assert_eq!(w(INF).div_f64(0), INF);
    assert!(w(NAN).div_f64(0).is_nan());
    assert!((w(INF) + w(NINF)).div_f64(2).is_nan());
    // A dividend past the double range with a quotient inside it.
    assert_eq!((w(f64::MAX) + w(f64::MAX)).div_f64(2), f64::MAX);
    assert_eq!((w(f64::MAX) + w(f64::MAX)).div_f64(1), INF);
    // Subnormal quotients: half a unit ties to the even zero, keeping the sign as IEEE does.
    assert_eq!(w(5e-324).div_f64(2).to_bits(), 0.0f64.to_bits());
    assert_eq!(w(-5e-324).div_f64(2).to_bits(), (-0.0f64).to_bits());
    assert_eq!(w(3.0 * 5e-324).div_f64(2), 2.0 * 5e-324);
    assert_eq!(w(7.0).div_f64(u64::MAX), 7.0 * 2f64.powi(-64));
}

#[test]
fn wide_reads_round_at_both_ends_of_the_double_range() {
    // Overflow: MAX + MAX reads ∞. MAX + half an ulp ties to even, and MAX's mantissa is odd, so
    // it rounds away, to ∞. MAX + less than half an ulp stays MAX.
    assert_eq!((w(f64::MAX) + w(f64::MAX)).to_f64(), INF);
    assert_eq!((w(-f64::MAX) + w(-f64::MAX)).to_f64(), NINF);
    assert_eq!((w(f64::MAX) + w(2f64.powi(970))).to_f64(), INF);
    assert_eq!((w(f64::MAX) + w(2f64.powi(969))).to_f64(), f64::MAX);
    assert_eq!((w(f64::MAX) + w(2f64.powi(969))).try_to_f64(), None);
    // 2^1024 is ∞ as a double but a finite state; retracting half of it is exact.
    let two_1024 = w(2f64.powi(1023)) + w(2f64.powi(1023));
    assert_eq!(two_1024.to_f64(), INF);
    assert_eq!(
        (two_1024 - w(2f64.powi(1023))).try_to_f64(),
        Some(2f64.powi(1023))
    );
    // Every nonzero state is at least one unit, and the unit is the smallest subnormal, so the
    // finite part never rounds to zero and the smallest sums read exactly.
    assert_eq!(
        (w(5e-324) + w(5e-324)).try_to_f64(),
        Some(f64::from_bits(2))
    );
    assert_eq!(
        (w(5e-324) + w(5e-324) + w(5e-324)).try_to_f64(),
        Some(f64::from_bits(3))
    );
    assert_eq!(
        (w(f64::MIN_POSITIVE) - w(5e-324)).to_f64().to_bits(),
        (1u64 << 52) - 1,
        "the largest subnormal"
    );
    assert_eq!(
        (w(-f64::MIN_POSITIVE) + w(5e-324)).to_f64().to_bits(),
        (1u64 << 63) | ((1u64 << 52) - 1)
    );
    // Cancellation across the whole range is exact.
    assert_eq!((w(f64::MAX) + w(5e-324) - w(f64::MAX)).to_f64(), 5e-324);
    assert_eq!((w(1e308) + w(-1e308) + w(-5e-324)).to_f64(), -5e-324);
    assert_eq!(
        (1e308f64 + 5e-324) - 1e308,
        0.0,
        "the f64 fold loses the small term"
    );
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "headroom")]
fn wide_exhausting_the_headroom_is_caught_in_debug() {
    // f64::MAX is 2098 bits over the unit; 2^77 copies fill the 2175 magnitude bits, and one more
    // doubling reaches the sign bit. That wraps in release and is a named assertion in debug.
    let mut f = w(f64::MAX);
    for _ in 0..77 {
        f += f;
    }
    let _ = f + f;
}

#[test]
fn wide_debug_names_the_value_the_counters_and_the_live_limbs() {
    // 1.5 is 3 · 2^-1: 3 << 1073, which is limb 16 at offset 49.
    let s = format!("{:?}", w(1.5));
    assert!(s.starts_with("Wide(1.5e0;"), "{s}");
    assert!(s.contains("0x6000000000000"), "{s}");
    assert!(s.contains("17 × 0x0"), "{s}");
    let n = format!("{:?}", -w(1.5));
    assert!(n.starts_with("Wide(-1.5e0;"), "{n}");
    assert!(n.contains("17 × 0xffffffffffffffff"), "{n}");
    let x = format!("{:?}", w(0.1) + w(0.2) + w(NAN) - w(INF));
    assert!(x.contains("NaN"), "{x}");
    assert!(x.contains("nan 1, +inf -1, -inf 0"), "{x}");
    let z = format!("{:?}", Wide::ZERO);
    assert!(z.starts_with("Wide(0e0;"), "{z}");
    assert!(z.contains("[]"), "{z}");
}
