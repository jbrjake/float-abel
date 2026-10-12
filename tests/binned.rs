//! `Binned<K>`, the batch-only tier: a fold lands the same bits under any order or partition, the
//! value is the sum of the inputs truncated below the window, that truncation is inside the bound,
//! and the conversions to and from `Wide` do what they are named. The oracles are `Wide`, which
//! holds any multiset of doubles exactly, and `Exact` where the spread lets it; the truncation
//! oracle is built from the definition in DESIGN.md § Binned with `Wide`'s limbs alone. Nothing
//! here goes through the laws kit: `Binned` is not a group, and the kit would say so by failing
//! to compile.

use core::mem::{align_of, size_of};

use float_abel::{Binned, Exact, Wide};

use crate::support::{hash, pow2, Rng};

const _: () = assert!(size_of::<Binned<3>>() == 80 && align_of::<Binned<3>>() == 8);

/// The `Wide` limb that holds the top bit of a finite nonzero double's magnitude.
fn top_limb(x: f64) -> usize {
    Wide::from(x.abs())
        .limbs()
        .iter()
        .rposition(|&l| l != 0)
        .expect("a finite nonzero double has a set limb")
}

/// The `Wide` limb number of the lowest bin a `Binned<K>` keeps after folding `xs`: the window's
/// top is the limb of the largest magnitude, and never below limb `K - 1`.
fn window_bottom<const K: usize>(xs: &[f64]) -> usize {
    let top = xs
        .iter()
        .filter(|x| x.is_finite() && **x != 0.0)
        .map(|&x| top_limb(x))
        .max()
        .unwrap_or(0);
    top.max(K - 1) + 1 - K
}

/// The definition of the value: each finite input's magnitude with every limb below `lo` zeroed,
/// signed as the input was, summed exactly, and the specials counted as `Wide` counts them.
fn truncated_sum(xs: &[f64], lo: usize) -> Wide {
    let mut acc = Wide::ZERO;
    for &x in xs {
        if !x.is_finite() {
            acc += Wide::from(x);
            continue;
        }
        let mut limbs = *Wide::from(x.abs()).limbs();
        for limb in &mut limbs[..lo] {
            *limb = 0;
        }
        let term = Wide::from_parts(limbs, [0; 3]);
        acc += if x.is_sign_negative() { -term } else { term };
    }
    acc
}

fn fold<const K: usize>(xs: &[f64]) -> Binned<K> {
    let mut b = Binned::<K>::ZERO;
    for &x in xs {
        b.deposit(x);
    }
    b
}

pub(crate) fn shuffle<T>(rng: &mut Rng, xs: &mut [T]) {
    for i in (1..xs.len()).rev() {
        let j = (rng.next() % (i as u64 + 1)) as usize;
        xs.swap(i, j);
    }
}

/// Same bits, by every reading the type offers.
pub(crate) fn assert_same_bits<const K: usize>(a: &Binned<K>, b: &Binned<K>, what: &str) {
    assert_eq!(a.index(), b.index(), "{what}: index");
    assert_eq!(a.specials(), b.specials(), "{what}: specials");
    assert_eq!(a, b, "{what}: structural");
    assert_eq!(hash(a), hash(b), "{what}: hash");
    assert_eq!(a.into_wide(), b.into_wide(), "{what}: into_wide");
}

#[test]
fn binned_layout_is_sixteen_bytes_a_bin_plus_a_word_and_the_counters() {
    assert_eq!(size_of::<Binned<1>>(), 16 + 32);
    assert_eq!(size_of::<Binned<2>>(), 32 + 32);
    assert_eq!(size_of::<Binned<3>>(), 80);
    assert_eq!(size_of::<Binned<5>>(), 80 + 32);
    assert_eq!(size_of::<Binned<33>>(), 16 * 33 + 32);
    assert_eq!(align_of::<Binned<3>>(), 8);
    assert_eq!(align_of::<Binned<33>>(), 8);
    let b = Binned::<3>::from(1.5);
    let copy = b;
    assert_eq!(copy, b, "Copy");
}

#[test]
fn binned_zero_is_default_reads_zero_and_sits_at_the_bottom_of_the_grid() {
    fn check<const K: usize>() {
        let z = Binned::<K>::ZERO;
        assert_eq!(z, Binned::<K>::default());
        assert_eq!(
            z.index(),
            K as i16 - 1,
            "the window starts at the bottom of Wide's grid"
        );
        assert_eq!(z.limbs(), &[0; K]);
        assert_eq!(z.carries(), &[0; K]);
        assert_eq!(z.specials(), (0, 0, 0));
        assert!(z.is_zero());
        assert!(z.is_finite());
        assert_eq!(z.into_wide(), Wide::ZERO);
        assert_eq!(z.into_wide().to_f64().to_bits(), 0.0f64.to_bits());
        // Either zero adds nothing and moves nothing.
        let mut still = z;
        still.deposit(0.0);
        still.deposit(-0.0);
        assert_eq!(still, z);
    }
    check::<1>();
    check::<3>();
    check::<33>();
}

#[test]
fn binned_a_fold_is_bit_identical_under_a_thousand_shuffles() {
    fn check<const K: usize>(seed: u64) {
        let mut rng = Rng(seed);
        let mut xs: Vec<f64> = (0..400).map(|_| rng.any()).collect();
        let base = fold::<K>(&xs);
        let (n, p, q) = base.specials();
        assert!(
            n > 0 && p > 0 && q > 0,
            "the multiset carries every special"
        );
        assert!(!base.is_zero());
        for round in 0..1000 {
            shuffle(&mut rng, &mut xs);
            let b = fold::<K>(&xs);
            assert_same_bits(&b, &base, &format!("K={K} shuffle {round}"));
        }
    }
    check::<1>(1);
    check::<2>(2);
    check::<3>(3);
    check::<5>(5);
}

#[test]
fn binned_a_fold_is_bit_identical_across_one_to_sixty_four_merge_partitions() {
    fn check<const K: usize>(seed: u64) {
        let mut rng = Rng(seed);
        let xs: Vec<f64> = (0..1000).map(|_| rng.any()).collect();
        let base = fold::<K>(&xs);
        for parts in 1..=64 {
            let size = xs.len().div_ceil(parts);
            let partials: Vec<Binned<K>> = xs.chunks(size).map(fold::<K>).collect();
            assert!(!partials.is_empty() && partials.len() <= parts);
            // Left to right, right to left, and a balanced tree.
            let forward = partials.iter().fold(Binned::<K>::ZERO, |acc, p| acc + *p);
            let backward = partials
                .iter()
                .rev()
                .fold(Binned::<K>::ZERO, |acc, p| acc + *p);
            let mut level = partials.clone();
            while level.len() > 1 {
                level = level
                    .chunks(2)
                    .map(|pair| pair.iter().copied().sum::<Binned<K>>())
                    .collect();
            }
            assert_same_bits(&forward, &base, &format!("K={K} {parts} parts forward"));
            assert_same_bits(&backward, &base, &format!("K={K} {parts} parts backward"));
            assert_same_bits(&level[0], &base, &format!("K={K} {parts} parts tree"));
        }
    }
    check::<1>(11);
    check::<3>(13);
    check::<5>(15);
}

#[test]
fn binned_add_is_associative_and_commutative_on_random_states() {
    let mut rng = Rng(21);
    for _ in 0..2000 {
        // States of very different index, so alignment drops bins on one side or the other.
        let mut state = || {
            let len = 1 + (rng.next() % 8) as usize;
            let xs: Vec<f64> = (0..len).map(|_| rng.any()).collect();
            fold::<3>(&xs)
        };
        let (a, b, c) = (state(), state(), state());
        assert_same_bits(&(a + b), &(b + a), "commutative");
        assert_same_bits(&((a + b) + c), &(a + (b + c)), "associative");
        let mut assigned = a;
        assigned += b;
        assert_same_bits(&assigned, &(a + b), "AddAssign");
    }
}

#[test]
fn binned_into_wide_is_the_sum_of_the_inputs_truncated_below_the_window() {
    fn check<const K: usize>(xs: &[f64], what: &str) {
        let lo = window_bottom::<K>(xs);
        let b = fold::<K>(xs);
        assert_eq!(b.index() as usize, lo + K - 1, "{what}: index");
        assert_eq!(b.into_wide(), truncated_sum(xs, lo), "{what}");
    }
    let mut rng = Rng(31);
    let uniform: Vec<f64> = (0..5000).map(|_| rng.any()).collect();
    let narrow: Vec<f64> = (0..5000).map(|_| rng.on_grid(-20, 20)).collect();
    let tiny: Vec<f64> = (0..5000).map(|_| rng.on_grid(-1022, -1000)).collect();
    let huge: Vec<f64> = (0..5000).map(|_| rng.on_grid(960, 1024)).collect();
    for (xs, what) in [
        (&uniform, "uniform"),
        (&narrow, "narrow"),
        (&tiny, "tiny"),
        (&huge, "huge"),
    ] {
        check::<1>(xs, what);
        check::<2>(xs, what);
        check::<3>(xs, what);
        check::<5>(xs, what);
        check::<33>(xs, what);
    }
}

#[test]
fn binned_is_within_the_bound_of_the_exact_sum_on_random_and_adversarial_multisets() {
    /// `|S − Σx| ≤ n · 2^(−64(K−1)) · max|x|`, and so inside the design's looser
    /// `2^(−60(K−1))`. Returns whether the fold lost anything at all.
    fn check<const K: usize>(xs: &[f64], oracle: &Wide, what: &str) -> bool {
        let n = xs.len() as f64;
        let max = xs
            .iter()
            .filter(|x| x.is_finite())
            .fold(0.0f64, |m, x| m.max(x.abs()));
        let got = fold::<K>(xs).into_wide();
        assert_eq!(got.specials(), oracle.specials(), "{what}: specials");
        let diff = got - *oracle;
        assert!(diff.is_finite());
        let error = diff.to_f64().abs();
        let tight = n * pow2(-64 * (K as i32 - 1)) * max;
        let documented = n * pow2(-60 * (K as i32 - 1)) * max;
        assert!(error <= tight, "{what}: K={K} error {error:e} > {tight:e}");
        assert!(
            error <= documented,
            "{what}: K={K} error {error:e} > {documented:e}"
        );
        !diff.is_zero()
    }

    let mut rng = Rng(41);
    // A real column: spread 2^±20 fits three limbs with room, so nothing is lost.
    let narrow: Vec<f64> = (0..10_000).map(|_| rng.on_grid(-20, 20)).collect();
    let exact = narrow
        .iter()
        .fold(Exact::ZERO, |acc, &x| acc + Exact::from(x));
    let wide = narrow
        .iter()
        .fold(Wide::ZERO, |acc, &x| acc + Wide::from(x));
    assert_eq!(
        Wide::try_from(&exact).unwrap(),
        wide,
        "the two oracles agree"
    );
    assert!(!check::<3>(&narrow, &wide, "narrow"), "nothing lost at K=3");
    assert!(
        check::<1>(&narrow, &wide, "narrow"),
        "something lost at K=1"
    );

    // The adversaries: uniform exponents (most inputs fall below the window), a column at the
    // edge of the window with full mantissas (every input loses part of itself), the same with
    // alternating signs (truncation toward zero on both sides), and specials mixed in.
    let uniform: Vec<f64> = (0..10_000).map(|_| rng.finite()).collect();
    let edge: Vec<f64> = core::iter::once(1.0)
        .chain((0..10_000).map(|_| rng.on_grid(-172, -168).abs()))
        .collect();
    let signed: Vec<f64> = core::iter::once(-1.0)
        .chain((0..10_000).map(|_| rng.on_grid(-172, -168)))
        .collect();
    let mixed: Vec<f64> = (0..10_000).map(|_| rng.any()).collect();
    let mut lossy = 0;
    for (xs, what) in [
        (&uniform, "uniform"),
        (&edge, "edge"),
        (&signed, "signed"),
        (&mixed, "mixed"),
    ] {
        let wide = xs.iter().fold(Wide::ZERO, |acc, &x| acc + Wide::from(x));
        for k in [1, 2, 3, 5] {
            let lost = match k {
                1 => check::<1>(xs, &wide, what),
                2 => check::<2>(xs, &wide, what),
                3 => check::<3>(xs, &wide, what),
                _ => check::<5>(xs, &wide, what),
            };
            lossy += usize::from(lost);
        }
    }
    assert!(
        lossy >= 8,
        "the adversaries exercise the lossy path: {lossy} lossy folds"
    );
}

#[test]
fn binned_a_fresh_fold_of_the_not_a_group_history_shows_the_documented_truncation() {
    let history = [1e30, 1e-10, -1e30];
    let wide = history
        .iter()
        .fold(Wide::ZERO, |acc, &x| acc + Wide::from(x));
    assert_eq!(
        wide.try_to_f64(),
        Some(1e-10),
        "the group holds the survivor exactly"
    );
    assert_eq!(
        Binned::<3>::from(1e-10).into_wide().try_to_f64(),
        Some(1e-10)
    );

    // Every order of the history lands the same bits, and they are not 1e-10: the 1e30 fixed the
    // window, and the bits of 1e-10 below it are gone whether or not the 1e30 is still there.
    let orders = [
        [1e30, 1e-10, -1e30],
        [1e-10, 1e30, -1e30],
        [-1e30, 1e-10, 1e30],
        [1e-10, -1e30, 1e30],
        [1e30, -1e30, 1e-10],
        [-1e30, 1e30, 1e-10],
    ];
    let base = fold::<3>(&history);
    for order in &orders {
        assert_same_bits(&fold::<3>(order), &base, "history order");
    }
    let lo = window_bottom::<3>(&history);
    assert_eq!(lo, 16);
    let truncated = truncated_sum(&history, lo);
    assert_eq!(base.into_wide(), truncated);
    let survivor = truncated.to_f64();
    assert_ne!(survivor, 1e-10);
    assert!(
        survivor > 0.0 && survivor < 1e-10,
        "truncated toward zero: {survivor:e}"
    );
    assert_eq!(
        survivor,
        112_589.0 * pow2(-50),
        "16 bits of 1e-10 survive at K=3"
    );
    // At K=1 the window is one limb, and the survivor reads as zero.
    assert_eq!(fold::<1>(&history).into_wide(), Wide::ZERO);
    assert!(fold::<1>(&history).is_zero());
    assert_eq!(
        fold::<1>(&history).index(),
        18,
        "the index remembers the 1e30"
    );
}

#[test]
fn binned_carries_outlast_sixteen_deposits_and_borrows_cancel_them() {
    // (2^53 − 1) · 2^25: 53 ones landing in bits 11..64 of Wide limb 17, so almost every deposit
    // wraps the bin's limb, and the carry counts the wraps.
    let x = f64::from_bits(((77 + 1023) << 52) | ((1 << 52) - 1));
    assert_eq!(top_limb(x), 17);
    let mut b = Binned::<3>::ZERO;
    for _ in 0..100_000 {
        b.deposit(x);
    }
    assert_eq!(b.index(), 17);
    assert_eq!(b.carries()[2], 99_999);
    assert_eq!(b.limbs()[2], 0u64.wrapping_sub(100_000 * 2048));
    let mut wide = Wide::from(x) * 100_000i64;
    assert_eq!(
        b.into_wide(),
        wide,
        "exact: the window holds the whole input"
    );
    for _ in 0..60_000 {
        b.deposit(-x);
    }
    wide -= Wide::from(x) * 60_000i64;
    assert_eq!(b.into_wide(), wide);
    assert_eq!(b.carries()[2], 39_999);
    for _ in 0..40_000 {
        b.deposit(-x);
    }
    assert_eq!(b.limbs(), &[0; 3]);
    assert_eq!(b.carries(), &[0; 3]);
    assert!(b.is_zero());
    assert_eq!(b.index(), 17, "the index does not come back down");
    assert_ne!(
        b,
        Binned::<3>::ZERO,
        "a cancelled fold is not the zero: no unique zero, no group"
    );
}

#[test]
fn binned_specials_are_counted_and_ride_both_conversions() {
    let mut b = Binned::<3>::from(f64::NAN);
    assert_eq!(b.specials(), (1, 0, 0));
    assert!(!b.is_finite());
    assert!(!b.is_zero());
    b.deposit(f64::INFINITY);
    b.deposit(f64::INFINITY);
    b.deposit(f64::NEG_INFINITY);
    assert_eq!(b.specials(), (1, 2, 1));
    assert_eq!(b.index(), 2, "specials do not move the window");
    b.deposit(2.5);
    assert_eq!(b.index(), 16);
    let w = b.into_wide();
    assert_eq!(w.specials(), (1, 2, 1));
    assert!(w.to_f64().is_nan());
    assert_eq!(w.to_binned::<3>(), b);
    let sum = b + Binned::from(f64::NEG_INFINITY);
    assert_eq!(sum.specials(), (1, 2, 2));
    let mut only_inf = Wide::from(f64::INFINITY);
    only_inf += Wide::from(1e300);
    let as_binned = only_inf.to_binned::<2>();
    assert_eq!(as_binned.specials(), (0, 1, 0));
    assert_eq!(as_binned.into_wide().to_f64(), f64::INFINITY);
}

#[test]
fn binned_to_binned_keeps_the_top_k_limbs_and_round_trips_a_double_exactly() {
    let mut rng = Rng(51);
    // A double spans at most two limbs, so two bins hold it whole.
    for _ in 0..10_000 {
        let x = rng.finite();
        let w = Wide::from(x);
        assert_eq!(w.to_binned::<2>().into_wide(), w, "{x:e}");
        assert_eq!(w.to_binned::<3>().into_wide(), w, "{x:e}");
        let b = w.to_binned::<2>();
        assert_eq!(b.into_wide().to_binned::<2>(), b, "{x:e}: idempotent");
        if x != 0.0 {
            assert_eq!(b.index() as usize, top_limb(x).max(1), "{x:e}");
        }
    }
    assert_eq!(Wide::ZERO.to_binned::<3>(), Binned::<3>::ZERO);
    // A sum wider than the window: the top K limbs of the magnitude survive, the sign is kept,
    // and the index is the magnitude's top limb.
    for _ in 0..200 {
        let xs: Vec<f64> = (0..50).map(|_| rng.finite()).collect();
        let w = xs.iter().fold(Wide::ZERO, |acc, &x| acc + Wide::from(x));
        let top = w.to_binned::<1>().index() as usize;
        let mut mag = [0u64; 34];
        let negative = w < Wide::ZERO;
        let abs = if negative { -w } else { w };
        mag.copy_from_slice(abs.limbs());
        assert_eq!(mag.iter().rposition(|&l| l != 0).unwrap(), top);
        for k in [1usize, 3, 5] {
            let lo = top.max(k - 1) + 1 - k;
            for limb in &mut mag[..lo] {
                *limb = 0;
            }
            let kept = Wide::from_parts(mag, [0; 3]);
            let want = if negative { -kept } else { kept };
            let got = match k {
                1 => w.to_binned::<1>().into_wide(),
                3 => w.to_binned::<3>().into_wide(),
                _ => w.to_binned::<5>().into_wide(),
            };
            assert_eq!(got, want, "K={k}");
            mag.copy_from_slice(abs.limbs());
        }
    }
}

#[test]
fn binned_sum_and_extend_from_slice_are_the_fold() {
    let mut rng = Rng(61);
    let xs: Vec<f64> = (0..3000).map(|_| rng.any()).collect();
    let base = fold::<3>(&xs);
    let mut bulk = Binned::<3>::ZERO;
    bulk.extend_from_slice(&xs);
    assert_same_bits(&bulk, &base, "extend_from_slice");
    let owned: Binned<3> = xs.iter().map(|&x| Binned::from(x)).sum();
    assert_same_bits(&owned, &base, "Sum<Binned>");
    let singles: Vec<Binned<3>> = xs.iter().map(|&x| Binned::from(x)).collect();
    let borrowed: Binned<3> = singles.iter().sum();
    assert_same_bits(&borrowed, &base, "Sum<&Binned>");
    let empty: Binned<3> = core::iter::empty::<Binned<3>>().sum();
    assert_same_bits(&empty, &Binned::ZERO, "empty sum");
}

#[test]
fn binned_debug_names_the_value_the_index_and_the_bins() {
    let b = fold::<3>(&[1.5, 1e-30]);
    let text = format!("{b:?}");
    assert!(text.starts_with("Binned<3>("), "{text}");
    assert!(text.contains("≈1.5e0"), "{text}");
    assert!(text.contains("index 16"), "{text}");
    assert!(text.contains("nan 0"), "{text}");
    let exact = format!("{:?}", Binned::<3>::from(2.0));
    assert!(exact.contains("(2e0;"), "{exact}");
}
