//! The banded `SUM` idiom on the heap `Exact`, docs/OVERFLOW.md § The banded idiom: `Exact::band`
//! puts a double's magnitude band beside the key, and `BandedSum::banded_sum` runs `explode` +
//! `count_total` per `(key, band)` and reduces a key's band totals into one `Exact`, so every diff
//! differential forms is a sum inside one band and the compact word never promotes there. The
//! fifteen histories and the seeded 2^±300 multiset of `tests/differential_products.rs` land the
//! plain idiom's updates through the banded one on one worker and three, every band total compact
//! where most key totals are heap values, and a cancellation across two bands emits nothing, as
//! `count_total` emits nothing for a key whose total is the zero. Every test is named
//! `differential_banded_*` so the task verify can filter on it.

use std::collections::BTreeSet;

use differential_dataflow::input::Input;
use differential_dataflow::operators::CountTotal;
use float_abel::{BandedSum, Exact};
use timely::dataflow::operators::Capture;

use crate::differential::{netted, Captured, Row, Worker};
use crate::differential_products::{
    count_total_updates, epochs_dataflow, feed_epochs, histories, history_totals, multiset, pairs,
    Epochs, Totals,
};
use crate::support::{pow2, rounds, Rng};

fn e(x: f64) -> Exact {
    Exact::from(x)
}

/// The banded idiom over `epochs`, fed as `epochs_dataflow` feeds the plain one.
fn banded_dataflow(worker: &mut Worker, epochs: &Epochs) -> Captured<(u32, Exact), isize> {
    let (input, probe, captured) = worker.dataflow::<u64, _, _>(|scope| {
        let (input, rows) = scope.new_collection::<Row, isize>();
        let (probe, totals) = rows.banded_sum().probe();
        (input, probe, totals.inner.capture())
    });
    feed_epochs(worker, input, &probe, epochs);
    captured
}

/// The idiom's first half alone, `explode` + `count_total` per `(key, band)`, so a test can read
/// the band totals the operator forms: the control that gives the idiom its reason.
fn band_totals_dataflow(
    worker: &mut Worker,
    epochs: &Epochs,
) -> Captured<((u32, u8), Exact), isize> {
    let (input, probe, captured) = worker.dataflow::<u64, _, _>(|scope| {
        let (input, rows) = scope.new_collection::<Row, isize>();
        let (probe, totals) = rows
            .explode(|(key, bits)| {
                let x = f64::from_bits(bits);
                Some(((key, Exact::band(x)), e(x)))
            })
            .count_total()
            .probe();
        (input, probe, totals.inner.capture())
    });
    feed_epochs(worker, input, &probe, epochs);
    captured
}

/// The exponents of a finite nonzero double's leading and lowest set bits, read off its bits.
fn bit_span(x: f64) -> (i32, i32) {
    let bits = x.to_bits();
    let biased = ((bits >> 52) & 0x7FF) as i32;
    let frac = bits & ((1 << 52) - 1);
    assert!(
        biased != 0x7FF && (biased != 0 || frac != 0),
        "{x} has no leading bit"
    );
    if biased == 0 {
        (
            -1074 + 63 - frac.leading_zeros() as i32,
            -1074 + frac.trailing_zeros() as i32,
        )
    } else {
        let mant = frac | (1 << 52);
        (biased - 1023, biased - 1075 + mant.trailing_zeros() as i32)
    }
}

/// A double with its leading bit at `2^e`, a full random fraction and either sign, subnormal
/// where `e` is under −1022.
fn with_leading_bit(rng: &mut Rng, e: i32) -> f64 {
    let bits = if e < -1022 {
        let lead = 1u64 << (e + 1074);
        lead | (rng.next() & (lead - 1))
    } else {
        ((e + 1023) as u64) << 52 | (rng.next() >> 12)
    };
    let x = f64::from_bits(bits);
    if rng.next() & 1 == 0 {
        x
    } else {
        -x
    }
}

/// Every finite double lands in `0..=65` with its leading bit inside its band's 32 binades and
/// its lowest set bit at most 52 under the band's floor, so it lies within 84 bits of the band's
/// top; a power of two at every exponent visits all 66 bands; the zero is in band 0, negation
/// moves nothing, and NaN and ±∞ take 255.
#[test]
fn differential_banded_every_finite_double_lies_within_84_bits_of_its_bands_top() {
    let mut rng = Rng(67);
    let mut sweep: Vec<f64> = (0..rounds(100_000)).map(|_| rng.finite()).collect();
    sweep.extend((-1074..=1023).map(pow2));
    sweep.extend([f64::MAX, f64::MIN_POSITIVE, 1.0, 0.1, 1e300, 1e-300]);
    let mut seen = BTreeSet::new();
    for x in sweep {
        let band = Exact::band(x);
        assert_eq!(Exact::band(-x), band, "{x:e}");
        assert!(band <= 65, "{x:e} in band {band}");
        seen.insert(band);
        let floor = 32 * i32::from(band) - 1074;
        let (leading, lowest) = bit_span(x);
        assert!(
            (floor..floor + 32).contains(&leading),
            "{x:e}: leading bit {leading} outside band {band}"
        );
        assert!(
            lowest >= floor - 52,
            "{x:e}: lowest bit {lowest} under band {band}'s floor"
        );
        assert!(
            floor + 31 - lowest < 84,
            "{x:e} spans past 84 bits from band {band}'s top"
        );
    }
    assert_eq!(seen.len(), 66);
    assert_eq!(Exact::band(0.0), 0);
    assert_eq!(Exact::band(-0.0), 0);
    assert_eq!(Exact::band(pow2(-1074)), 0);
    assert_eq!(Exact::band(1.0), 33);
    assert_eq!(Exact::band(f64::MAX), 65);
    for s in [f64::NAN, -f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(Exact::band(s), 255, "{s}");
    }
}

/// On every band a fold of 2¹⁰ full-mantissa doubles of either sign stays compact at every step,
/// 84 + 10 bits inside the width, which is the bound the idiom rests on. The control: one value
/// from band 30 and one from band 34, 128 binades apart, promote at the first add.
#[test]
fn differential_banded_a_fold_inside_one_band_never_promotes() {
    let mut rng = Rng(68);
    for band in 0..=65u8 {
        let floor = 32 * i32::from(band) - 1074;
        let top = (floor + 31).min(1023);
        let mut sum = Exact::ZERO;
        for i in 0..rounds(1024) {
            let x = with_leading_bit(&mut rng, floor + (i as i32 % (top - floor + 1)));
            assert_eq!(Exact::band(x), band, "{x:e}");
            sum += &e(x);
            assert!(sum.is_compact(), "band {band} promoted at value {i}");
        }
    }
    let low = f64::from_bits(((-114i64 + 1023) as u64) << 52 | 1);
    let (high, low_band, high_band) = (pow2(45), Exact::band(low), Exact::band(pow2(45)));
    assert_eq!((low_band, high_band), (30, 34));
    assert_eq!(bit_span(low), (-114, -166));
    assert!(!(e(high) + e(low)).is_compact());
}

/// Each history lands exactly the updates the plain idiom lands (`tests/differential_products.rs`),
/// history B's held `big + small` included as the reduce's heap output, with the last live read
/// the small value exactly, on one worker and three. The two values of every pair lie in
/// different bands, so inside the operator each band total is one of them alone.
#[test]
#[cfg_attr(miri, ignore = "three timely workers exchanging serialized batches")]
fn differential_banded_the_five_histories_land_the_plain_idioms_updates() {
    for (big, small) in pairs() {
        assert_ne!(
            Exact::band(big),
            Exact::band(small),
            "{big:e} and {small:e}"
        );
        for (name, epochs) in histories(big, small) {
            let want = count_total_updates(&history_totals(name, big, small));
            for workers in [1, 3] {
                let epochs = epochs.clone();
                let got = netted(workers, move |worker| banded_dataflow(worker, &epochs));
                assert_eq!(
                    got, want,
                    "{name} on {workers} workers, {big:e} and {small:e}"
                );
                let last = got
                    .iter()
                    .filter(|(_, _, diff)| *diff > 0)
                    .max_by_key(|(_, epoch, _)| *epoch)
                    .expect("a live read");
                assert_eq!(last.0 .1.try_to_f64(), Some(small));
            }
        }
    }
}

/// The seeded multiset through both idioms on one input: the banded idiom lands the `Wide` oracle
/// per (key, epoch) and the plain idiom's updates exactly, on one worker and three. The control
/// that gives the idiom its reason: more than half the key totals it emits are heap values, while
/// every band total `count_total` forms on the way is compact, across the twenty bands the
/// multiset occupies.
#[test]
#[cfg_attr(miri, ignore = "three timely workers exchanging serialized batches")]
fn differential_banded_the_seeded_multiset_lands_the_oracle_with_every_band_total_compact() {
    let (epochs, totals) = multiset();
    let want = count_total_updates(&totals);
    let promoted = want.iter().filter(|((_, t), _, _)| !t.is_compact()).count();
    assert!(
        promoted * 2 > want.len(),
        "{promoted} of {} key totals are heap values",
        want.len()
    );
    for workers in [1, 3] {
        let input = epochs.clone();
        let got = netted(workers, move |worker| banded_dataflow(worker, &input));
        assert_eq!(got.len(), want.len(), "{workers} workers");
        assert_eq!(got, want, "{workers} workers");
        let input = epochs.clone();
        let plain = netted(workers, move |worker| epochs_dataflow(worker, &input));
        assert_eq!(plain, got, "{workers} workers: the two idioms differ");

        let input = epochs.clone();
        let bands = netted(workers, move |worker| band_totals_dataflow(worker, &input));
        assert!(bands.len() > want.len(), "{} band totals", bands.len());
        let heap = bands
            .iter()
            .filter(|((_, t), _, _)| !t.is_compact())
            .count();
        assert_eq!(
            heap,
            0,
            "{heap} of {} band totals promoted on {workers} workers",
            bands.len()
        );
        let occupied: BTreeSet<u8> = bands.iter().map(|(((_, band), _), _, _)| *band).collect();
        assert_eq!(occupied.len(), 20, "{occupied:?}");
        assert!(occupied.contains(&255));
        let finite: Vec<u8> = occupied.iter().copied().filter(|b| *b != 255).collect();
        assert_eq!((finite[0], finite[finite.len() - 1]), (24, 42));
    }
}

/// `2^14` opens band 34 and `2^13` closes band 33, so `+2^14 | −2^13 −2^13` holds two live
/// nonzero band totals that sum to the zero across bands, and the reduce emits nothing for the
/// key, as `count_total` emits nothing for a total `is_zero` deleted; a later `+2^13` reads out
/// `2^13`. One worker and three.
#[test]
#[cfg_attr(miri, ignore = "three timely workers exchanging serialized batches")]
fn differential_banded_a_cancellation_across_two_bands_emits_nothing() {
    let (big, half) = (pow2(14), pow2(13));
    assert_eq!((Exact::band(big), Exact::band(half)), (34, 33));
    let (b, h) = ((0u32, big.to_bits()), (0u32, half.to_bits()));
    let epochs: Epochs = vec![vec![(b, 1)], vec![(h, -2)], vec![(h, 1)]];
    let totals: Totals = vec![vec![e(big)], vec![Exact::ZERO], vec![e(half)]];
    let want = count_total_updates(&totals);
    assert_eq!(want.len(), 3);
    for workers in [1, 3] {
        let input = epochs.clone();
        let got = netted(workers, move |worker| banded_dataflow(worker, &input));
        assert_eq!(got, want, "{workers} workers");
        let input = epochs.clone();
        let bands = netted(workers, move |worker| band_totals_dataflow(worker, &input));
        assert!(bands.contains(&(((0, 34), e(big)), 0, 1)), "{bands:?}");
        assert!(bands.contains(&(((0, 33), -e(big)), 1, 1)), "{bands:?}");
        assert!(bands.contains(&(((0, 33), -e(big)), 2, -1)), "{bands:?}");
        assert!(bands.contains(&(((0, 33), -e(half)), 2, 1)), "{bands:?}");
    }
}
