//! `Wide` and `Fixed` as differential-dataflow diffs, DESIGN.md § Wide and § Fixed: the `Copy`
//! words for an engine whose weight slot must be `Copy` or that casts over a mapping, `Fixed` the
//! declared window with a per-value door check. The five histories of docs/OVERFLOW.md complete on
//! one worker and three with `Wide` as the diff on every pair, and with `Fixed256` on the 1e12
//! pair while `Fixed256` refuses 2¹⁰⁰⁰ and 1e-300 at the door before any dataflow starts; a fold of
//! row products reaches the diff through `deposit_product` at the door and lands a `num-bigint`
//! oracle; a retraction epoch nets to nothing where `f64` leaves residue; `join`, `semijoin` and
//! `antijoin` with `isize` land on the orphan `Multiply` impls in either order; and `count_core`
//! and `distinct_core` put `From<i8>` out. Every test is named `differential_wide_*` or
//! `differential_fixed_*` so the task verify can filter on it.
//!
//! A row carries its amount as the double's bits, because a collection's data must be `Ord` and
//! `f64` is not; `explode` turns the bits into the diff, and that is where the door check runs.

use core::hash::Hash;
use core::ops::{Add, Mul};

use differential_dataflow::consolidation::consolidate_updates;
use differential_dataflow::difference::{Abelian, Multiply, Semigroup};
use differential_dataflow::input::Input;
use differential_dataflow::operators::CountTotal;
use differential_dataflow::{ExchangeData, VecCollection};
use float_abel::{Fixed256, OutOfWindow, Wide};
use num_bigint::BigInt;
use timely::dataflow::operators::Capture;

use crate::differential::{netted, sorted, Captured, Row, Update, Worker};
use crate::differential_products::{feed_epochs, histories, history_totals, pairs, Epochs};
use crate::exact_heap::{units, wide_units};
use crate::support::{pow2, Rng};

/// What the two words share here: a door that admits a double or the product of two, refusing by
/// name, and the finite part in the oracle's units, so one dataflow builder and one oracle serve
/// both. The supertraits are differential's weight traits, which is what the topic is about.
trait Word:
    ExchangeData
    + Hash
    + Copy
    + Abelian
    + Add<Output = Self>
    + Mul<i64, Output = Self>
    + Multiply<isize, Output = Self>
    + Multiply<i64, Output = Self>
    + From<i8>
{
    const NAME: &'static str;
    fn admit(x: f64) -> Result<Self, OutOfWindow>;
    fn admit_product(a: f64, b: f64) -> Result<Self, OutOfWindow>;
    /// The finite part in units of `2^-1074`, with no limb of this crate's in the reading.
    fn units(&self) -> BigInt;
}

impl Word for Wide {
    const NAME: &'static str = "Wide";

    fn admit(x: f64) -> Result<Self, OutOfWindow> {
        Ok(Wide::from(x))
    }

    fn admit_product(a: f64, b: f64) -> Result<Self, OutOfWindow> {
        let mut w = Wide::ZERO;
        w.deposit_product(a, b)?;
        Ok(w)
    }

    fn units(&self) -> BigInt {
        wide_units(self)
    }
}

impl Word for Fixed256 {
    const NAME: &'static str = "Fixed256";

    fn admit(x: f64) -> Result<Self, OutOfWindow> {
        Fixed256::try_from(x)
    }

    fn admit_product(a: f64, b: f64) -> Result<Self, OutOfWindow> {
        let mut f = Fixed256::ZERO;
        f.deposit_product(a, b)?;
        Ok(f)
    }

    /// Two's complement across the four limbs in units of `2^-96`, shifted onto the oracle's grid.
    fn units(&self) -> BigInt {
        let mut bytes = Vec::with_capacity(32);
        for limb in self.limbs() {
            bytes.extend_from_slice(&limb.to_le_bytes());
        }
        BigInt::from_signed_bytes_le(&bytes) << (1074 - 96)
    }
}

/// `x` through the door, which this topic's fixtures are all inside.
fn admit<R: Word>(x: f64) -> R {
    R::admit(x).unwrap_or_else(|e| panic!("{}: {x:e} refused at the door: {e}", R::NAME))
}

/// This worker's share of `rows`: every `peers`-th row from `index`, so three workers feed three
/// interleaved thirds and exchange reassembles the keys.
fn share<'a, T>(worker: &Worker, rows: &'a [T]) -> impl Iterator<Item = &'a T> {
    let (index, peers) = (worker.index(), worker.peers());
    rows.iter().skip(index).step_by(peers)
}

/// The plain idiom with `R` as the diff over `epochs`: `explode` the amount into the diff, the
/// door check there, then `count_total`.
fn epochs_dataflow<R: Word>(worker: &mut Worker, epochs: &Epochs) -> Captured<(u32, R), isize> {
    let (input, probe, captured) = worker.dataflow::<u64, _, _>(|scope| {
        let (input, rows) = scope.new_collection::<Row, isize>();
        let (probe, totals) = rows
            .explode(|(key, bits)| Some((key, admit::<R>(f64::from_bits(bits)))))
            .count_total()
            .probe();
        (input, probe, totals.inner.capture())
    });
    feed_epochs(worker, input, &probe, epochs);
    captured
}

/// The one key's running total after each epoch of history `name`, in `R`: B holds `big + small`
/// across an epoch, and D passes through the zero.
fn totals_in<R: Word>(name: &str, big: f64, small: f64) -> Vec<Vec<R>> {
    let (b, s) = (admit::<R>(big), admit::<R>(small));
    match name {
        "A" => vec![vec![b], vec![s]],
        "B" => vec![vec![b], vec![b + s], vec![s]],
        "C" | "E" => vec![vec![s]],
        "D" => vec![vec![b], vec![R::zero()], vec![s]],
        _ => unreachable!("history {name}"),
    }
}

/// What `count_total` emits for one key over `totals`: at an epoch where the total changes, the
/// old total retracted and the new one inserted, the zero never a row because `is_zero` deletes
/// it before the operator sees the key.
fn updates_in<R: Word>(totals: &[Vec<R>]) -> Vec<Update<(u32, R), isize>> {
    let mut updates = Vec::new();
    for key in 0..totals[0].len() {
        let mut previous = R::zero();
        for (epoch, per_key) in totals.iter().enumerate() {
            let total = per_key[key];
            if total != previous {
                if !previous.is_zero() {
                    updates.push(((key as u32, previous), epoch as u64, -1));
                }
                if !total.is_zero() {
                    updates.push(((key as u32, total), epoch as u64, 1));
                }
            }
            previous = total;
        }
    }
    sorted(updates)
}

/// Each history on both worker counts lands exactly the updates its running totals predict, and
/// every history's last live read is the small value, exactly, in the oracle's units.
fn histories_complete<R: Word>(big: f64, small: f64) {
    for (name, epochs) in histories(big, small) {
        let want = updates_in(&totals_in::<R>(name, big, small));
        assert_ne!(want.len(), 0);
        for workers in [1, 3] {
            let epochs = epochs.clone();
            let got = netted(workers, move |worker| epochs_dataflow::<R>(worker, &epochs));
            let at = format!(
                "{} {name} on {workers} workers, {big:e} and {small:e}",
                R::NAME
            );
            assert_eq!(got.len(), want.len(), "{at}");
            assert_eq!(got, want, "{at}");
            let last = got
                .iter()
                .filter(|(_, _, diff)| *diff > 0)
                .max_by_key(|(_, epoch, _)| *epoch)
                .expect("a live read");
            assert_eq!(last.0 .1.units(), units(small), "{at}");
        }
    }
}

/// `Wide` holds every pair, the 1e300 one included, with nothing to check at the door, and its
/// running totals are `tests/differential_products.rs`'s `Exact` ones through `TryFrom`.
#[test]
#[cfg_attr(miri, ignore = "three timely workers exchanging serialized batches")]
fn differential_wide_the_five_histories_complete_on_one_and_three_workers() {
    for (big, small) in pairs() {
        for (name, _) in histories(big, small) {
            let ours = totals_in::<Wide>(name, big, small);
            let theirs = history_totals(name, big, small);
            assert_eq!(ours.len(), theirs.len());
            for (w, x) in ours.iter().flatten().zip(theirs.iter().flatten()) {
                assert_eq!(
                    *w,
                    Wide::try_from(x).expect("a sum of doubles lies inside Wide")
                );
            }
        }
        histories_complete::<Wide>(big, small);
    }
}

/// `Fixed256` holds the 1e12 pair, every history completing on both worker counts, and refuses
/// the other pairs at the door by name before any dataflow starts: the per-value check doing its
/// job, where the compact `Exact` had none (docs/OVERFLOW.md § What fits).
#[test]
#[cfg_attr(miri, ignore = "three timely workers exchanging serialized batches")]
fn differential_fixed_the_1e12_pair_completes_and_the_rest_is_refused_at_the_door() {
    let [(big, small), (two_up, two_down), (huge, tiny)] = pairs();
    assert_eq!((big, small), (1e12, 1e-12));
    assert_eq!(two_up, pow2(1000));
    assert_eq!(Fixed256::try_from(two_up), Err(OutOfWindow::AboveCeiling));
    assert_eq!(Fixed256::try_from(two_down), Err(OutOfWindow::OffGrid));
    assert_eq!(Fixed256::try_from(huge), Err(OutOfWindow::AboveCeiling));
    assert_eq!(Fixed256::try_from(tiny), Err(OutOfWindow::OffGrid));
    assert_eq!(Fixed256::try_from(f64::NAN), Err(OutOfWindow::NotFinite));
    histories_complete::<Fixed256>(big, small);
}

/// A row of two factors, each as its bits.
type Pair = (u32, u64, u64);

/// A factor with a 26-bit mantissa in `[2^-20, 2^20)`, so a product of two has its lowest bit at
/// or above `2^-90`, on `Fixed256`'s grid, and its magnitude under `2^40`, inside every window.
fn factor(rng: &mut Rng) -> f64 {
    let x = rng.on_grid(-20, 20);
    f64::from_bits(x.to_bits() & !((1u64 << 27) - 1))
}

/// `n` factor pairs on each of `keys` keys.
fn factor_rows(rng: &mut Rng, keys: u32, n: usize) -> Vec<Pair> {
    let mut rows = Vec::with_capacity(keys as usize * n);
    for key in 0..keys {
        for _ in 0..n {
            rows.push((key, factor(rng).to_bits(), factor(rng).to_bits()));
        }
    }
    rows
}

/// The exact sum of `rows`' products on `key`, in units of `2^-2148`.
fn products_oracle(rows: &[Pair], key: u32) -> BigInt {
    rows.iter()
        .filter(|row| row.0 == key)
        .map(|&(_, a, b)| units(f64::from_bits(a)) * units(f64::from_bits(b)))
        .sum()
}

/// A fold of row products with `R` as the diff: `explode` each row into `deposit_product` of its
/// two factors at the door, then `count_total`. Each worker feeds its share at epoch 0 and
/// retracts its share of `retract` at epoch 1.
fn products_dataflow<R: Word>(
    worker: &mut Worker,
    rows: &[Pair],
    retract: &[Pair],
) -> Captured<(u32, R), isize> {
    let (mut input, captured) = worker.dataflow::<u64, _, _>(|scope| {
        let (input, rows) = scope.new_collection::<Pair, isize>();
        let totals = rows
            .explode(|(key, a, b)| {
                let (a, b) = (f64::from_bits(a), f64::from_bits(b));
                let product = R::admit_product(a, b).unwrap_or_else(|e| {
                    panic!("{}: {a:e} × {b:e} refused at the door: {e}", R::NAME)
                });
                Some((key, product))
            })
            .count_total();
        (input, totals.inner.capture())
    });
    for row in share(worker, rows) {
        input.insert(*row);
    }
    input.advance_to(1);
    input.flush();
    for row in share(worker, retract) {
        input.remove(*row);
    }
    input.close();
    captured
}

/// Each key's total is the oracle's bit for bit before and after a third of its rows are
/// retracted, the retraction is the old total at `-1`, and one worker and three land one answer.
fn products_fold_lands_the_oracle<R: Word>() {
    const KEYS: u32 = 4;
    let mut rng = Rng(68);
    let rows = factor_rows(&mut rng, KEYS, 24);
    let retract: Vec<Pair> = rows.iter().copied().step_by(3).collect();
    let kept: Vec<Pair> = rows
        .iter()
        .copied()
        .filter(|row| !retract.contains(row))
        .collect();
    assert_eq!(kept.len() + retract.len(), rows.len());
    for key in 0..KEYS {
        let (before, after) = (products_oracle(&rows, key), products_oracle(&kept, key));
        assert!(before != BigInt::from(0) && before != after, "key {key}");
    }
    let runs: Vec<Vec<Update<(u32, R), isize>>> = [1, 3]
        .into_iter()
        .map(|workers| {
            let (rows, retract) = (rows.clone(), retract.clone());
            netted(workers, move |worker| {
                products_dataflow::<R>(worker, &rows, &retract)
            })
        })
        .collect();
    assert_eq!(runs[0].len(), 3 * KEYS as usize, "{}", R::NAME);
    assert_eq!(runs[0], runs[1], "{}: one worker and three", R::NAME);
    let got = &runs[0];
    for key in 0..KEYS {
        let at = |epoch: u64, diff: isize| -> R {
            let mut found = got
                .iter()
                .filter(|((k, _), t, d)| *k == key && *t == epoch && *d == diff)
                .map(|((_, total), _, _)| *total);
            let total = found
                .next()
                .unwrap_or_else(|| panic!("key {key} at {epoch} {diff:+}"));
            assert!(found.next().is_none(), "key {key} at {epoch} {diff:+}");
            total
        };
        let before = at(0, 1);
        assert_eq!(
            before.units() << 1074,
            products_oracle(&rows, key),
            "key {key}"
        );
        assert_eq!(at(1, -1), before, "key {key}");
        assert_eq!(
            at(1, 1).units() << 1074,
            products_oracle(&kept, key),
            "key {key}"
        );
    }
}

#[test]
#[cfg_attr(miri, ignore = "three timely workers exchanging serialized batches")]
fn differential_wide_a_fold_of_row_products_lands_the_oracle_through_the_door() {
    products_fold_lands_the_oracle::<Wide>();
}

#[test]
#[cfg_attr(miri, ignore = "three timely workers exchanging serialized batches")]
fn differential_fixed_a_fold_of_row_products_lands_the_oracle_through_the_door() {
    products_fold_lands_the_oracle::<Fixed256>();
}

/// `n` rows over sixteen keys with amounts in tenths, `-4.8` to `4.8`: values an `f64` sum does
/// not hold exactly, every one on `Fixed256`'s grid.
fn tenths(n: u64) -> Vec<Row> {
    (0..n)
        .map(|i| {
            let key = ((i * 7919) % 16) as u32;
            let amount = (((i * 31) % 97) as f64 - 48.0) / 10.0;
            (key, amount.to_bits())
        })
        .collect()
}

/// The SUM dataflow with `R` as the diff: this worker feeds its share of `inserts` at epoch 0,
/// retracts its share of `retracts` at epoch 1, and closes the input.
fn sum_dataflow<R: Word>(
    worker: &mut Worker,
    inserts: &[Row],
    retracts: &[Row],
) -> Captured<(u32, R), isize> {
    let (mut input, captured) = worker.dataflow::<u64, _, _>(|scope| {
        let (input, rows) = scope.new_collection::<Row, isize>();
        let totals = rows
            .explode(|(key, bits)| Some((key, admit::<R>(f64::from_bits(bits)))))
            .count_total();
        (input, totals.inner.capture())
    });
    for row in share(worker, inserts) {
        input.insert(*row);
    }
    input.advance_to(1);
    input.flush();
    for row in share(worker, retracts) {
        input.remove(*row);
    }
    input.close();
    captured
}

/// Epoch 0 inserts the rows and epoch 1 retracts every one of them, each retraction arriving as
/// `-1 × R` through `Multiply<isize>`. Epoch 1's output is the retraction of epoch 0's totals and
/// nothing else, and the two epochs net to nothing, where the same rows folded in `f64` leave
/// residue.
fn retraction_nets_to_nothing<R: Word>() {
    let rows = tenths(2_000);
    let mut folded = [0.0f64; 16];
    for &(key, bits) in &rows {
        folded[key as usize] += f64::from_bits(bits);
    }
    for &(key, bits) in &rows {
        folded[key as usize] -= f64::from_bits(bits);
    }
    assert!(
        folded.iter().any(|residue| *residue != 0.0),
        "the f64 control left no residue, so these rows do not show the difference"
    );

    let out = netted(3, move |worker| sum_dataflow::<R>(worker, &rows, &rows));
    let (epoch0, epoch1): (Vec<_>, Vec<_>) = out.iter().copied().partition(|(_, t, _)| *t == 0);
    assert_eq!(epoch0.len(), 16, "{}", R::NAME);
    let retracted: Vec<_> = epoch0.iter().map(|&(d, _, r)| (d, 1, -r)).collect();
    assert_eq!(epoch1, retracted, "{}", R::NAME);
    let mut net: Vec<((u32, R), (), isize)> = out.into_iter().map(|(d, _, r)| (d, (), r)).collect();
    consolidate_updates(&mut net);
    assert!(net.is_empty(), "{}: residue {net:?}", R::NAME);
}

#[test]
#[cfg_attr(miri, ignore = "three timely workers exchanging serialized batches")]
fn differential_wide_a_retraction_epoch_nets_to_nothing_where_f64_leaves_residue() {
    retraction_nets_to_nothing::<Wide>();
}

#[test]
#[cfg_attr(miri, ignore = "three timely workers exchanging serialized batches")]
fn differential_fixed_a_retraction_epoch_nets_to_nothing_where_f64_leaves_residue() {
    retraction_nets_to_nothing::<Fixed256>();
}

/// The fixture the operator tests share: `(key, value)` rows weighted by `R`.
type Fixture<'scope, R> = VecCollection<'scope, u64, (u32, u32), R>;

/// Runs `op` over the fixture on three workers and returns the netted output. Worker 0 feeds
/// every row and exchange spreads them; the pair on key 3 cancels to the zero, which `is_zero`
/// deletes before any operator sees the key. Every weight is on `Fixed256`'s grid.
fn over_fixture<R, D, R2, F>(op: F) -> Vec<Update<D, R2>>
where
    R: Word,
    D: ExchangeData,
    R2: ExchangeData + Semigroup,
    F: for<'scope> Fn(Fixture<'scope, R>) -> VecCollection<'scope, u64, D, R2>
        + Send
        + Sync
        + 'static,
{
    netted(3, move |worker| {
        let (mut input, captured) = worker.dataflow::<u64, _, _>(|scope| {
            let (input, weighted) = scope.new_collection::<(u32, u32), R>();
            (input, op(weighted).inner.capture())
        });
        if worker.index() == 0 {
            for (row, weight) in [
                ((1, 10), 0.1),
                ((1, 10), 0.2),
                ((1, 11), 1.0),
                ((2, 20), -0.3),
                ((3, 30), 0.5),
                ((3, 30), -0.5),
            ] {
                input.update(row, admit::<R>(weight));
            }
        }
        input.close();
        captured
    })
}

/// The keys of the fixture with `isize` multiplicities, key 2 twice: the other side of every join.
fn keys<'scope, R: Word>(weighted: &Fixture<'scope, R>) -> VecCollection<'scope, u64, u32, isize> {
    let keys = weighted.clone().map(|(k, _)| k).distinct();
    keys.clone().concat(keys.filter(|k| *k == 2))
}

/// `join` with the word on the left and `isize` on the right, and the other way round, land on
/// `Multiply<isize> for R` and `Multiply<R> for isize`, the multiplicity scaling the weight;
/// `semijoin` keeps the rows whose key the `isize` collection has, scaled, and `antijoin` the
/// rows whose key it lacks, through `Multiply<isize, Output = R>` and `Abelian`; a collection
/// concatenated with its own negation consolidates to nothing.
fn joins_with_isize_in_either_order<R: Word>()
where
    isize: Multiply<R, Output = R>,
{
    let w110 = admit::<R>(0.1) + admit::<R>(0.2);
    let (one, neg3) = (admit::<R>(1.0), admit::<R>(-0.3));

    let word_times_isize = over_fixture::<R, _, _, _>(|weighted| {
        let tagged = keys(&weighted).map(|k| (k, k * 100));
        weighted.join(tagged)
    });
    assert_eq!(
        word_times_isize,
        sorted(vec![
            ((1, (10, 100)), 0, w110),
            ((1, (11, 100)), 0, one),
            ((2, (20, 200)), 0, neg3 * 2i64),
        ]),
        "{}",
        R::NAME
    );

    let isize_times_word = over_fixture::<R, _, _, _>(|weighted| {
        let tagged = keys(&weighted).map(|k| (k, k * 100));
        tagged.join(weighted)
    });
    assert_eq!(
        isize_times_word,
        sorted(vec![
            ((1, (100, 10)), 0, w110),
            ((1, (100, 11)), 0, one),
            ((2, (200, 20)), 0, neg3 * 2i64),
        ]),
        "{}",
        R::NAME
    );

    let kept = over_fixture::<R, _, _, _>(|weighted| {
        let keys = keys(&weighted);
        weighted.semijoin(keys)
    });
    assert_eq!(
        kept,
        sorted(vec![
            ((1, 10), 0, w110),
            ((1, 11), 0, one),
            ((2, 20), 0, neg3 * 2i64),
        ]),
        "{}",
        R::NAME
    );

    let rest = over_fixture::<R, _, _, _>(|weighted| {
        let one = keys(&weighted).filter(|k| *k == 1);
        weighted.antijoin(one)
    });
    assert_eq!(rest, sorted(vec![((2, 20), 0, neg3)]), "{}", R::NAME);

    let cancelled = over_fixture::<R, _, _, _>(|weighted| {
        weighted.clone().concat(weighted.negate()).consolidate()
    });
    assert!(cancelled.is_empty(), "{}: {cancelled:?}", R::NAME);
}

#[test]
#[cfg_attr(miri, ignore = "three timely workers exchanging serialized batches")]
fn differential_wide_join_semijoin_and_antijoin_with_isize_in_either_order() {
    joins_with_isize_in_either_order::<Wide>();
}

#[test]
#[cfg_attr(miri, ignore = "three timely workers exchanging serialized batches")]
fn differential_fixed_join_semijoin_and_antijoin_with_isize_in_either_order() {
    joins_with_isize_in_either_order::<Fixed256>();
}

/// `count_total` carries a key's total as data with an `isize` diff; `count_core::<R>` and
/// `distinct_core::<R>` put `R::from(1i8)` out; `threshold` maps an `R` weight to an `R` weight.
fn count_distinct_and_threshold_take_the_word<R: Word>() {
    let total = admit::<R>(0.1) + admit::<R>(0.2) + admit::<R>(1.0);
    let neg3 = admit::<R>(-0.3);
    let one = R::from(1i8);

    let counted = over_fixture::<R, _, _, _>(|weighted| weighted.map(|(k, _)| k).count_total());
    assert_eq!(
        counted,
        sorted(vec![((1, total), 0, 1), ((2, neg3), 0, 1)]),
        "{}",
        R::NAME
    );

    let counted_word =
        over_fixture::<R, _, _, _>(|weighted| weighted.map(|(k, _)| k).count_core::<R>());
    assert_eq!(
        counted_word,
        sorted(vec![((1, total), 0, one), ((2, neg3), 0, one)]),
        "{}",
        R::NAME
    );

    let distinct =
        over_fixture::<R, _, _, _>(|weighted| weighted.map(|(k, _)| k).distinct_core::<R>());
    assert_eq!(
        distinct,
        sorted(vec![(1, 0, one), (2, 0, one)]),
        "{}",
        R::NAME
    );

    let doubled = over_fixture::<R, _, _, _>(|weighted| {
        weighted.map(|(k, _)| k).threshold(|_k, w| *w * 2i64)
    });
    assert_eq!(
        doubled,
        sorted(vec![(1, 0, total * 2i64), (2, 0, neg3 * 2i64)]),
        "{}",
        R::NAME
    );
}

#[test]
#[cfg_attr(miri, ignore = "three timely workers exchanging serialized batches")]
fn differential_wide_count_distinct_and_threshold_take_the_word() {
    count_distinct_and_threshold_take_the_word::<Wide>();
}

#[test]
#[cfg_attr(miri, ignore = "three timely workers exchanging serialized batches")]
fn differential_fixed_count_distinct_and_threshold_take_the_word() {
    count_distinct_and_threshold_take_the_word::<Fixed256>();
}

/// `From<i8>` is the integer, which is what `count_core` and `distinct_core` put out as one.
fn from_i8_is_the_integer<R: Word>() {
    for k in [i8::MIN, -1, 0, 1, 2, i8::MAX] {
        assert_eq!(R::from(k), admit::<R>(f64::from(k)), "{} from {k}", R::NAME);
        assert_eq!(
            R::from(k).units(),
            units(f64::from(k)),
            "{} from {k}",
            R::NAME
        );
    }
    assert!(R::from(0).is_zero());
    assert_eq!(R::from(1) * 3i64, admit::<R>(3.0));
}

#[test]
fn differential_wide_from_i8_is_the_integer() {
    from_i8_is_the_integer::<Wide>();
}

#[test]
fn differential_fixed_from_i8_is_the_integer() {
    from_i8_is_the_integer::<Fixed256>();
}

/// `From<f64>` is total on `Wide`, so a NaN or an infinity is a row like any other: counted into
/// the diff, exchanged, summed with multiplicity, and retracted exactly, where an `f64` diff could
/// never retract a NaN. `Fixed256` refuses the same rows at the door by name, because its contract
/// is exact or refused.
#[test]
#[cfg_attr(miri, ignore = "three timely workers exchanging serialized batches")]
fn differential_wide_specials_ride_the_diff_and_retract_exactly() {
    let rows: Vec<Row> = [1.5, f64::NAN, f64::INFINITY, 2.5, 2.5]
        .map(|x| (7, x.to_bits()))
        .to_vec();
    let retract = vec![(7, f64::NAN.to_bits())];
    let out = netted(3, move |worker| {
        sum_dataflow::<Wide>(worker, &rows, &retract)
    });
    let w = Wide::from;
    let with_nan = w(1.5) + w(f64::NAN) + w(f64::INFINITY) + w(2.5) * 2i64;
    let without = w(1.5) + w(f64::INFINITY) + w(2.5) * 2i64;
    assert!(with_nan.to_f64().is_nan());
    assert_eq!(with_nan.specials(), (1, 1, 0));
    assert_eq!(without.to_f64(), f64::INFINITY);
    assert_eq!(without.specials(), (0, 1, 0));
    assert_eq!(out.len(), 3);
    assert_eq!(
        out,
        sorted(vec![
            ((7, with_nan), 0, 1),
            ((7, with_nan), 1, -1),
            ((7, without), 1, 1),
        ])
    );
    for special in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(Fixed256::try_from(special), Err(OutOfWindow::NotFinite));
    }
}
