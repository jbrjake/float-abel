//! The SIMD surface, DESIGN.md § Speed: every tier's bulk entries, `extend_from_slice` and
//! `deposit_products`, land the bits the one-at-a-time deposits land, on the conformance vectors
//! and on 10⁴ seeded multisets, whatever kernel this machine dispatches to. The one-at-a-time loop
//! is the definition of correct; a kernel is right when nothing here can tell them apart. Every
//! test is named `simd_*`. The portable kernel's lane-count independence is a unit test in
//! src/simd/scalar.rs under the same prefix, because the lane count is not reachable from here.

use float_abel::{Binned, Fixed, Fixed256, OutOfWindow, Wide};

use crate::support::{rounds, Rng};

/// 10⁴ seeded multisets, and twenty under miri, which runs the portable kernel alone and some
/// thousand times slower.
const MULTISETS: u64 = if cfg!(miri) { 20 } else { 10_000 };

/// The scalar definition of correct for each tier: one deposit at a time, in slice order, with
/// `Fixed`'s first refusal ending the fold.
fn wide_loop(xs: &[f64]) -> Wide {
    let mut w = Wide::ZERO;
    for &x in xs {
        w.deposit(x);
    }
    w
}

fn fixed_loop<const LSB: i32, const LIMBS: usize>(
    xs: &[f64],
) -> Result<Fixed<LSB, LIMBS>, OutOfWindow> {
    let mut f = Fixed::<LSB, LIMBS>::ZERO;
    for &x in xs {
        f.try_deposit(x)?;
    }
    Ok(f)
}

fn binned_loop<const K: usize>(xs: &[f64]) -> Binned<K> {
    let mut b = Binned::<K>::ZERO;
    for &x in xs {
        b.deposit(x);
    }
    b
}

fn wide_products_loop(a: &[f64], b: &[f64]) -> Result<Wide, OutOfWindow> {
    let mut w = Wide::ZERO;
    for (&x, &y) in a.iter().zip(b) {
        w.deposit_product(x, y)?;
    }
    Ok(w)
}

fn fixed_products_loop<const LSB: i32, const LIMBS: usize>(
    a: &[f64],
    b: &[f64],
) -> Result<Fixed<LSB, LIMBS>, OutOfWindow> {
    let mut f = Fixed::<LSB, LIMBS>::ZERO;
    for (&x, &y) in a.iter().zip(b) {
        f.deposit_product(x, y)?;
    }
    Ok(f)
}

/// The seeded multisets. Most are short, so that every tier's refusals and specials appear often;
/// every fiftieth is long enough to cross a kernel's tile more than once, and the kinds cycle as
/// the golden transcript's do: any double, any finite double, `Fixed256`'s window, and a band
/// every tier holds whole.
fn multiset(rng: &mut Rng, round: u64) -> Vec<f64> {
    let len = if round % 50 == 49 {
        1000 + (rng.next() % 3000) as usize
    } else {
        (rng.next() % 70) as usize
    };
    (0..len)
        .map(|_| match round % 4 {
            0 => rng.any(),
            1 => rng.finite(),
            2 => rng.on_grid(-44, 95),
            _ => rng.on_grid(-30, 36),
        })
        .collect()
}

/// The bitrep conformance cases' inputs: the vectors the kernels must reproduce.
fn conformance_inputs() -> Vec<(String, Vec<f64>)> {
    let v: serde_json::Value =
        serde_json::from_str(include_str!("data/bitrep/vectors.json")).unwrap();
    v["cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            let name = c["name"].as_str().unwrap().to_owned();
            let inputs = c["input_bits"]
                .as_array()
                .unwrap()
                .iter()
                .map(|h| f64::from_bits(u64::from_str_radix(h.as_str().unwrap(), 16).unwrap()))
                .collect();
            (name, inputs)
        })
        .collect()
}

/// Every input set the bulk tests run: the conformance cases first, then the multisets.
fn every_input_set() -> Vec<(String, Vec<f64>)> {
    let mut sets = conformance_inputs();
    let mut rng = Rng(0x51_4D_44);
    for round in 0..MULTISETS {
        sets.push((format!("multiset {round}"), multiset(&mut rng, round)));
    }
    sets
}

fn check_fixed<const LSB: i32, const LIMBS: usize>(name: &str, xs: &[f64]) {
    let expected = fixed_loop::<LSB, LIMBS>(xs);
    let mut bulk = Fixed::<LSB, LIMBS>::ZERO;
    let got = bulk.extend_from_slice(xs);
    match expected {
        Ok(f) => {
            assert_eq!(
                got,
                Ok(()),
                "{name}: Fixed<{LSB}, {LIMBS}> accepted the loop's inputs"
            );
            assert_eq!(bulk, f, "{name}: Fixed<{LSB}, {LIMBS}> bits");
        }
        Err(e) => {
            assert_eq!(
                got,
                Err(e),
                "{name}: Fixed<{LSB}, {LIMBS}> refuses the first refusal"
            );
            assert_eq!(
                bulk,
                Fixed::<LSB, LIMBS>::ZERO,
                "{name}: Fixed<{LSB}, {LIMBS}> left the state as it was"
            );
        }
    }
}

#[test]
fn simd_wide_extend_from_slice_lands_the_deposit_loops_bits_on_vectors_and_multisets() {
    let sets = every_input_set();
    assert_eq!(sets.len() as u64, 10 + MULTISETS);
    for (name, xs) in &sets {
        let mut bulk = Wide::ZERO;
        bulk.extend_from_slice(xs);
        assert_eq!(bulk, wide_loop(xs), "{name}");
        // Onto a state that already holds something, too: the bulk entry adds, it does not replace.
        let mut onto = Wide::from(3.5);
        onto.extend_from_slice(xs);
        assert_eq!(
            onto,
            wide_loop(xs) + Wide::from(3.5),
            "{name}: onto a state"
        );
    }
}

#[test]
fn simd_fixed_extend_from_slice_lands_the_loops_bits_and_refuses_the_first_refusal() {
    let sets = every_input_set();
    for (name, xs) in &sets {
        check_fixed::<-96, 4>(name, xs);
        // Two limbs: the smallest window, so the ceiling is hit often.
        check_fixed::<-40, 2>(name, xs);
        // Three limbs, and a grid at the bottom of the doubles.
        check_fixed::<-1074, 3>(name, xs);
        // Six limbs: wider than any kernel's window, so this is the portable path on every machine.
        check_fixed::<-200, 6>(name, xs);
    }
}

#[test]
fn simd_fixed_bulk_refusal_leaves_a_held_state_as_it_was() {
    let mut rng = Rng(7);
    for _ in 0..rounds(1000) {
        let held = Fixed256::try_from(rng.on_grid(-20, 20)).unwrap();
        let mut xs: Vec<f64> = (0..(rng.next() % 2100) as usize)
            .map(|_| rng.on_grid(-44, 95))
            .collect();
        let bad = [f64::NAN, f64::INFINITY, 1e-30, 1e40][(rng.next() % 4) as usize];
        let at = (rng.next() % (xs.len() as u64 + 1)) as usize;
        xs.insert(at, bad);
        let mut bulk = held;
        let err = bulk.extend_from_slice(&xs).unwrap_err();
        assert_eq!(err, fixed_loop::<-96, 4>(&xs).unwrap_err());
        assert_eq!(
            bulk,
            held,
            "a refusal at {at} of {} left the state",
            xs.len()
        );
    }
}

#[test]
fn simd_binned_extend_from_slice_lands_the_folds_bits_at_every_width() {
    let sets = every_input_set();
    for (name, xs) in &sets {
        let mut b1 = Binned::<1>::ZERO;
        b1.extend_from_slice(xs);
        assert_eq!(b1, binned_loop::<1>(xs), "{name}: K = 1");
        let mut b2 = Binned::<2>::ZERO;
        b2.extend_from_slice(xs);
        assert_eq!(b2, binned_loop::<2>(xs), "{name}: K = 2");
        let mut b3 = Binned::<3>::ZERO;
        b3.extend_from_slice(xs);
        assert_eq!(b3, binned_loop::<3>(xs), "{name}: K = 3");
        assert_eq!(
            b3.into_wide(),
            binned_loop::<3>(xs).into_wide(),
            "{name}: K = 3 as Wide"
        );
        // Past every kernel's bin budget: the portable path.
        let mut b5 = Binned::<5>::ZERO;
        b5.extend_from_slice(xs);
        assert_eq!(b5, binned_loop::<5>(xs), "{name}: K = 5");
        // Onto a fold that already has a window: the tile merges adaptively.
        let mut onto = Binned::<3>::from(1e30);
        onto.extend_from_slice(xs);
        assert_eq!(
            onto,
            binned_loop::<3>(xs) + Binned::<3>::from(1e30),
            "{name}: onto"
        );
    }
}

/// A tile's worst mix: zeros of both signs, subnormals, the top and the bottom of the normal range,
/// a special on some rounds, and a same-scale body.
fn mixing_multiset(rng: &mut Rng, round: u64) -> Vec<f64> {
    let len = (rng.next() % 2600) as usize;
    (0..len)
        .map(|i| match (i as u64 + round) % 11 {
            0 => 0.0,
            1 => -0.0,
            2 => f64::from_bits(rng.next() >> 12), // a subnormal
            3 => rng.on_grid(900, 1023),
            4 => rng.on_grid(-1022, -900),
            5 if round % 3 == 0 => f64::NAN,
            6 if round % 5 == 0 => f64::INFINITY,
            7 if round % 7 == 0 => f64::NEG_INFINITY,
            _ => rng.on_grid(-30, 30),
        })
        .collect()
}

#[test]
#[cfg_attr(miri, ignore = "multisets of up to 2600 doubles, four folds each")]
fn simd_wide_tiles_mixing_spans_specials_zeros_and_subnormals_match_the_loop() {
    let mut rng = Rng(0xA5);
    for round in 0..rounds(400) as u64 {
        let xs = mixing_multiset(&mut rng, round);
        let len = xs.len();
        let mut bulk = Wide::ZERO;
        bulk.extend_from_slice(&xs);
        assert_eq!(bulk, wide_loop(&xs), "round {round}, {len} inputs");
        let mut bins = Binned::<3>::ZERO;
        bins.extend_from_slice(&xs);
        assert_eq!(
            bins,
            binned_loop::<3>(&xs),
            "round {round}, {len} inputs: Binned"
        );
    }
}

#[test]
#[cfg_attr(miri, ignore = "three million deposits across a tile's lengths")]
fn simd_bulk_entries_take_every_slice_length_across_a_tile() {
    let mut rng = Rng(0x7E);
    let xs: Vec<f64> = (0..4200).map(|_| rng.on_grid(-44, 95)).collect();
    let lengths = (0..70)
        .chain(1000..1040)
        .chain(2040..2060)
        .chain([3071, 3072, 3073, 4095, 4096, 4097, 4200]);
    for len in lengths {
        let xs = &xs[..len];
        let mut w = Wide::ZERO;
        w.extend_from_slice(xs);
        assert_eq!(w, wide_loop(xs), "Wide, {len}");
        let mut f = Fixed256::ZERO;
        f.extend_from_slice(xs).unwrap();
        assert_eq!(f, fixed_loop::<-96, 4>(xs).unwrap(), "Fixed256, {len}");
        let mut b = Binned::<3>::ZERO;
        b.extend_from_slice(xs);
        assert_eq!(b, binned_loop::<3>(xs), "Binned<3>, {len}");
    }
}

#[test]
fn simd_deposit_products_match_the_pairwise_loop_or_refuse_the_first_pair_on_every_tier() {
    let sets = every_input_set();
    let mut rng = Rng(0xD0);
    for (name, a) in &sets {
        let b: Vec<f64> = a.iter().map(|_| rng.on_grid(-20, 20)).collect();

        let mut wide = Wide::from(1.0);
        let got = wide.deposit_products(a, &b);
        match wide_products_loop(a, &b) {
            Ok(w) => {
                assert_eq!(got, Ok(()), "{name}: Wide");
                assert_eq!(wide, w + Wide::from(1.0), "{name}: Wide bits");
            }
            Err(e) => {
                assert_eq!(got, Err(e), "{name}: Wide refuses the first pair");
                assert_eq!(wide, Wide::from(1.0), "{name}: Wide left the state");
            }
        }

        let mut fixed = Fixed256::try_from(1.0).unwrap();
        let got = fixed.deposit_products(a, &b);
        match fixed_products_loop::<-96, 4>(a, &b) {
            Ok(f) => {
                assert_eq!(got, Ok(()), "{name}: Fixed256");
                assert_eq!(
                    fixed,
                    f + Fixed256::try_from(1.0).unwrap(),
                    "{name}: Fixed256 bits"
                );
            }
            Err(e) => {
                assert_eq!(got, Err(e), "{name}: Fixed256 refuses the first pair");
                assert_eq!(
                    fixed,
                    Fixed256::try_from(1.0).unwrap(),
                    "{name}: Fixed256 left the state"
                );
            }
        }

        let mut bins = Binned::<3>::ZERO;
        let mut expect = Binned::<3>::ZERO;
        let mut first_err = None;
        for (&x, &y) in a.iter().zip(&b) {
            if let Err(e) = expect.deposit_product(x, y) {
                first_err = Some(e);
                break;
            }
        }
        let got = bins.deposit_products(a, &b);
        match first_err {
            None => {
                assert_eq!(got, Ok(()), "{name}: Binned");
                assert_eq!(bins, expect, "{name}: Binned bits");
            }
            Some(e) => {
                assert_eq!(got, Err(e), "{name}: Binned refuses the first pair");
                assert_eq!(bins, Binned::<3>::ZERO, "{name}: Binned left the state");
            }
        }
    }
}
