//! The heap `Exact` inside differential-dataflow, DESIGN.md § differential-dataflow and
//! docs/OVERFLOW.md: the five histories complete through the plain `explode` + `count_total` idiom
//! reading the small value exactly, the spike's seeded 2^±300 multiset lands a `Wide` oracle per
//! (key, epoch), a `join` of two `Exact`-weighted collections and a depth-3 join chain fold to a
//! `num-bigint` oracle, and `iterate` carries a float weight through three rounds of
//! `Exact × Exact`. One worker and three with serialized exchange, one answer. Every test is named
//! `differential_products_*` so the task verify can filter on it.
//!
//! A row carries its amount as the double's bits (`Row`), and `explode` turns the bits into the
//! weight; where a collection is one side of a join the bits stay in the data too, so each row is
//! its own value.

use differential_dataflow::consolidation::consolidate_updates;
use differential_dataflow::input::{Input, InputSession};
use differential_dataflow::operators::{CountTotal, Iterate};
use differential_dataflow::VecCollection;
use float_abel::{Exact, Wide};
use num_bigint::BigInt;
use timely::dataflow::operators::Capture;
use timely::dataflow::ProbeHandle;

use crate::differential::{netted, share, sorted, Captured, Row, Update, Worker};
use crate::exact_heap::{units, wide_units};
use crate::exact_heap_ring::product_units;
use crate::support::{pow2, Rng};

/// Epochs of `(row, multiplicity)` updates, worker 0 feeding each epoch in the order written and
/// stepping the dataflow to completion before the next, so each epoch's running totals are formed
/// before the next epoch's updates arrive, as the spike's probes fed them.
pub(crate) type Epochs = Vec<Vec<(Row, isize)>>;
/// Each key's `Exact` total after each epoch: `totals[epoch][key]`.
pub(crate) type Totals = Vec<Vec<Exact>>;

/// The pairs of `docs/OVERFLOW.md`'s histories, and the one `tests/exact_heap.rs` adds; each sum
/// of the two promotes.
pub(crate) fn pairs() -> [(f64, f64); 3] {
    [(1e12, 1e-12), (pow2(1000), pow2(-1000)), (1e300, 1e-300)]
}

fn e(x: f64) -> Exact {
    Exact::from(x)
}

fn amount(row: Row) -> Exact {
    e(f64::from_bits(row.1))
}

/// Feeds `epochs` as the type's doc says and closes the input, so the plain idiom and the banded
/// one (`tests/differential_banded.rs`) run one input the same way.
pub(crate) fn feed_epochs(
    worker: &mut Worker,
    mut input: InputSession<u64, Row, isize>,
    probe: &ProbeHandle<u64>,
    epochs: &Epochs,
) {
    for (epoch, updates) in epochs.iter().enumerate() {
        if worker.index() == 0 {
            for &(row, multiplicity) in updates {
                input.update(row, multiplicity);
            }
        }
        input.advance_to(epoch as u64 + 1);
        input.flush();
        worker.step_while(|| probe.less_than(input.time()));
    }
    input.close();
}

/// The plain idiom over `epochs`.
pub(crate) fn epochs_dataflow(
    worker: &mut Worker,
    epochs: &Epochs,
) -> Captured<(u32, Exact), isize> {
    let (input, probe, captured) = worker.dataflow::<u64, _, _>(|scope| {
        let (input, rows) = scope.new_collection::<Row, isize>();
        let (probe, totals) = rows
            .explode(|(key, bits)| Some((key, e(f64::from_bits(bits)))))
            .count_total()
            .probe();
        (input, probe, totals.inner.capture())
    });
    feed_epochs(worker, input, &probe, epochs);
    captured
}

/// The five histories of docs/OVERFLOW.md on one pair, each ending at the multiset `{small}`,
/// with `|` between epochs: A `+big | −big +small`, B `+big | +small | −big`, C `+big +small
/// −big`, D `+big | −big | +small`, E `+big −big +small`.
pub(crate) fn histories(big: f64, small: f64) -> [(&'static str, Epochs); 5] {
    let (b, s) = ((0u32, big.to_bits()), (0u32, small.to_bits()));
    [
        ("A", vec![vec![(b, 1)], vec![(b, -1), (s, 1)]]),
        ("B", vec![vec![(b, 1)], vec![(s, 1)], vec![(b, -1)]]),
        ("C", vec![vec![(b, 1), (s, 1), (b, -1)]]),
        ("D", vec![vec![(b, 1)], vec![(b, -1)], vec![(s, 1)]]),
        ("E", vec![vec![(b, 1), (b, -1), (s, 1)]]),
    ]
}

/// The one key's running total after each epoch of history `name`: B holds the promoted
/// `big + small` across an epoch, and D passes through the zero.
pub(crate) fn history_totals(name: &str, big: f64, small: f64) -> Totals {
    let (b, s) = (e(big), e(small));
    match name {
        "A" => vec![vec![b], vec![s]],
        "B" => vec![vec![b.clone()], vec![b + &s], vec![s]],
        "C" | "E" => vec![vec![s]],
        "D" => vec![vec![b], vec![Exact::ZERO], vec![s]],
        _ => unreachable!("history {name}"),
    }
}

/// What `count_total` emits for one key over `totals`: at an epoch where the total changes, the
/// old total retracted and the new one inserted, the zero never a row because `is_zero` deletes
/// it before the operator sees the key.
pub(crate) fn count_total_updates(totals: &Totals) -> Vec<Update<(u32, Exact), isize>> {
    let mut updates = Vec::new();
    for key in 0..totals[0].len() {
        let mut previous = Exact::ZERO;
        for (epoch, per_key) in totals.iter().enumerate() {
            let total = &per_key[key];
            if *total != previous {
                if !previous.is_zero() {
                    updates.push(((key as u32, previous.clone()), epoch as u64, -1));
                }
                if !total.is_zero() {
                    updates.push(((key as u32, total.clone()), epoch as u64, 1));
                }
            }
            previous.clone_from(total);
        }
    }
    sorted(updates)
}

/// Each history on both worker counts lands exactly the updates its running totals predict, so
/// history B holds the promoted `big + small` across an epoch as data and C forms it on the way
/// to `small` inside one epoch, and every history's last live read is the small value exactly.
/// The Copy word panicked in A, B and C on both pairs (docs/OVERFLOW.md § Why `Exact` panics).
#[test]
#[cfg_attr(miri, ignore = "three timely workers exchanging serialized batches")]
fn differential_products_the_five_histories_complete_on_one_and_three_workers() {
    for (big, small) in pairs() {
        let held = e(big) + e(small);
        assert!(!held.is_compact(), "{big:e} + {small:e} did not promote");
        for (name, epochs) in histories(big, small) {
            let totals = history_totals(name, big, small);
            assert_eq!(
                totals.iter().flatten().filter(|t| **t == held).count(),
                usize::from(name == "B")
            );
            let want = count_total_updates(&totals);
            for workers in [1, 3] {
                let epochs = epochs.clone();
                let got = netted(workers, move |worker| epochs_dataflow(worker, &epochs));
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

/// The panic needed no dataflow: differential's own `consolidate_updates` over the three updates
/// of history C forms `big + small` on the way to `small` when its unstable sort leaves them in
/// that order, and the Copy word refused it at the width (the spike's `repro`). Both orders now
/// consolidate to `[small]` in the compact form.
#[test]
fn differential_products_consolidate_updates_lands_the_small_value_in_the_c_order() {
    for (big, small) in pairs() {
        let mut c = vec![(1u32, 0u64, e(big)), (1, 0, e(small)), (1, 0, e(-big))];
        let mut e_order = vec![(1u32, 0u64, e(big)), (1, 0, e(-big)), (1, 0, e(small))];
        consolidate_updates(&mut c);
        consolidate_updates(&mut e_order);
        assert_eq!(c, vec![(1, 0, e(small))], "{big:e} and {small:e}");
        assert_eq!(e_order, c);
        assert!(c[0].2.is_compact());
    }
}

/// The spike's `band` probe's multiset through the plain idiom: ten epochs over five keys, each
/// inserting 400 rows spread over 2^±300 with full mantissas and either sign, then retracting a
/// third of what is live, with a NaN or an infinity one row in about 300 on key 4 alone. Returns
/// the epochs and each key's total after each epoch through `Wide`, which is the oracle.
pub(crate) fn multiset() -> (Epochs, Totals) {
    const KEYS: u32 = 5;
    let mut rng = Rng(63);
    let mut live: Vec<Row> = Vec::new();
    let mut epochs = Epochs::new();
    let mut totals = Totals::new();
    for _ in 0..10 {
        let mut epoch = Vec::new();
        for _ in 0..400 {
            let key = (rng.next() % u64::from(KEYS)) as u32;
            let x = if key == 4 && rng.next() % 300 == 0 {
                [f64::NAN, f64::INFINITY, f64::NEG_INFINITY][(rng.next() % 3) as usize]
            } else {
                rng.on_grid(-300, 300)
            };
            let row = (key, x.to_bits());
            epoch.push((row, 1));
            live.push(row);
        }
        let mut i = 0;
        while i < live.len() {
            if rng.next() % 3 == 0 {
                epoch.push((live.swap_remove(i), -1));
            } else {
                i += 1;
            }
        }
        epochs.push(epoch);
        let mut per_key = vec![Wide::ZERO; KEYS as usize];
        for &(key, bits) in &live {
            per_key[key as usize].deposit(f64::from_bits(bits));
        }
        totals.push(
            per_key
                .iter()
                .map(|w| Exact::try_from(w).expect("every Wide converts under alloc"))
                .collect(),
        );
    }
    (epochs, totals)
}

/// The multiset that panicked on one worker and hung on three through the plain idiom on the
/// Copy word (docs/OVERFLOW.md § The banded idiom) lands every (key, epoch) total on the `Wide`
/// oracle on both worker counts, nearly every total a heap value and key 4's carrying counts.
#[test]
#[cfg_attr(miri, ignore = "three timely workers exchanging serialized batches")]
fn differential_products_the_seeded_multiset_lands_the_wide_oracle_per_key_and_epoch() {
    let (epochs, totals) = multiset();
    let updates: usize = epochs.iter().map(Vec::len).sum();
    assert!(updates > 7_000, "{updates}");
    let promoted = totals.iter().flatten().filter(|t| !t.is_compact()).count();
    assert!(promoted >= 45, "{promoted} of 50 totals promoted");
    assert!(totals.iter().any(|per_key| !per_key[4].is_finite()));
    let want = count_total_updates(&totals);
    assert!(want.len() >= 90, "{}", want.len());
    for workers in [1, 3] {
        let epochs = epochs.clone();
        let got = netted(workers, move |worker| epochs_dataflow(worker, &epochs));
        assert_eq!(got.len(), want.len(), "{workers} workers");
        assert_eq!(got, want, "{workers} workers");
    }
}

/// `n` rows on each of `keys` keys, each amount a double over 2^±300.
fn band_rows(rng: &mut Rng, keys: u32, n: usize) -> Vec<Row> {
    let mut rows = Vec::with_capacity(keys as usize * n);
    for key in 0..keys {
        for _ in 0..n {
            rows.push((key, rng.on_grid(-300, 300).to_bits()));
        }
    }
    rows
}

/// The exact sum of `rows`' amounts on `key`, as the ring folds it.
fn key_total(rows: &[Row], key: u32) -> Exact {
    rows.iter()
        .filter(|row| row.0 == key)
        .map(|&row| amount(row))
        .sum()
}

/// The same sum in units of `2^-1074`, with no limb of this crate's in it.
fn key_units(rows: &[Row], key: u32) -> BigInt {
    rows.iter()
        .filter(|row| row.0 == key)
        .map(|&(_, bits)| units(f64::from_bits(bits)))
        .sum()
}

/// `rows` as `(key, bits)` data weighted by the double the bits spell, one side of a join.
fn weighted(rows: VecCollection<'_, u64, Row, isize>) -> VecCollection<'_, u64, (u32, u64), Exact> {
    rows.explode(|(key, bits)| Some(((key, bits), e(f64::from_bits(bits)))))
}

/// Two `Exact`-weighted collections joined on the key and folded by `count_total`, so each key's
/// total is the sum over every pair of the two weights' ring product, `Σᵢⱼ aᵢ·bⱼ`. Each worker
/// feeds its share of both sides at epoch 0 and retracts its share of `retract` from the left at
/// epoch 1.
fn join_dataflow(
    worker: &mut Worker,
    left: &[Row],
    right: &[Row],
    retract: &[Row],
) -> Captured<(u32, Exact), isize> {
    let (mut l, mut r, captured) = worker.dataflow::<u64, _, _>(|scope| {
        let (l_in, l) = scope.new_collection::<Row, isize>();
        let (r_in, r) = scope.new_collection::<Row, isize>();
        let totals = weighted(l)
            .join(weighted(r))
            .map(|(key, _)| key)
            .count_total();
        (l_in, r_in, totals.inner.capture())
    });
    for row in share(worker, left) {
        l.insert(*row);
    }
    for row in share(worker, right) {
        r.insert(*row);
    }
    l.advance_to(1);
    l.flush();
    r.advance_to(1);
    r.flush();
    for row in share(worker, retract) {
        l.remove(*row);
    }
    l.close();
    r.close();
    captured
}

/// A `float × float` join folded by `count_total`: every key's total is `(Σaᵢ)·(Σbⱼ)` as the
/// ring computes it from the two heap sums, which the `num-bigint` oracle confirms in units of
/// `2^-2148`, and retracting a third of the left side moves the total with it. One worker and
/// three land the same updates.
#[test]
#[cfg_attr(miri, ignore = "three timely workers exchanging serialized batches")]
fn differential_products_a_join_of_two_exact_weighted_collections_folds_to_the_oracle() {
    const KEYS: u32 = 5;
    let mut rng = Rng(64);
    let left = band_rows(&mut rng, KEYS, 30);
    let right = band_rows(&mut rng, KEYS, 30);
    let retract: Vec<Row> = left.iter().copied().step_by(3).collect();
    let kept: Vec<Row> = left
        .iter()
        .copied()
        .filter(|row| !retract.contains(row))
        .collect();
    assert_eq!(kept.len() + retract.len(), left.len());
    let mut want = Vec::new();
    for key in 0..KEYS {
        let before = key_total(&left, key) * key_total(&right, key);
        let after = key_total(&kept, key) * key_total(&right, key);
        assert!(!before.is_compact() && !after.is_compact(), "key {key}");
        assert_eq!(
            product_units(&before),
            key_units(&left, key) * key_units(&right, key),
            "key {key}"
        );
        assert_eq!(
            product_units(&after),
            key_units(&kept, key) * key_units(&right, key),
            "key {key}"
        );
        want.push(((key, before.clone()), 0, 1));
        want.push(((key, before), 1, -1));
        want.push(((key, after), 1, 1));
    }
    let want = sorted(want);
    for workers in [1, 3] {
        let (left, right, retract) = (left.clone(), right.clone(), retract.clone());
        let got = netted(workers, move |worker| {
            join_dataflow(worker, &left, &right, &retract)
        });
        assert_eq!(got.len(), 3 * KEYS as usize, "{workers} workers");
        assert_eq!(got, want, "{workers} workers");
    }
}

/// Three `Exact`-weighted collections joined in a chain and folded by `count_total`, each key's
/// total `Σᵢⱼₖ aᵢ·bⱼ·cₖ`; every worker feeds its share of each side.
fn chain_dataflow(
    worker: &mut Worker,
    a: &[Row],
    b: &[Row],
    c: &[Row],
) -> Captured<(u32, Exact), isize> {
    let (mut a_in, mut b_in, mut c_in, captured) = worker.dataflow::<u64, _, _>(|scope| {
        let (a_in, a) = scope.new_collection::<Row, isize>();
        let (b_in, b) = scope.new_collection::<Row, isize>();
        let (c_in, c) = scope.new_collection::<Row, isize>();
        let totals = weighted(a)
            .join(weighted(b))
            .join(weighted(c))
            .map(|(key, _)| key)
            .count_total();
        (a_in, b_in, c_in, totals.inner.capture())
    });
    for row in share(worker, a) {
        a_in.insert(*row);
    }
    for row in share(worker, b) {
        b_in.insert(*row);
    }
    for row in share(worker, c) {
        c_in.insert(*row);
    }
    a_in.close();
    b_in.close();
    c_in.close();
    captured
}

/// A depth-3 join chain: each key's total is `(Σa)(Σb)(Σc)` in units of `2^-3222` against the
/// `num-bigint` oracle, the same bits on one worker and three. A fold of three-factor products
/// is past anything the `Copy` word or `Wide::deposit_product` holds.
#[test]
#[cfg_attr(miri, ignore = "three timely workers exchanging serialized batches")]
fn differential_products_a_depth_three_join_chain_lands_the_oracle_bits() {
    const KEYS: u32 = 5;
    let mut rng = Rng(65);
    let a = band_rows(&mut rng, KEYS, 8);
    let b = band_rows(&mut rng, KEYS, 8);
    let c = band_rows(&mut rng, KEYS, 8);
    let mut want = Vec::new();
    for key in 0..KEYS {
        let total = key_total(&a, key) * key_total(&b, key) * key_total(&c, key);
        assert!(!total.is_compact(), "key {key}");
        let w = Wide::try_from(&total).expect("three factors of this band lie inside Wide");
        assert_eq!(
            wide_units(&w) << 2148,
            key_units(&a, key) * key_units(&b, key) * key_units(&c, key),
            "key {key}"
        );
        want.push(((key, total), 0, 1));
    }
    let want = sorted(want);
    for workers in [1, 3] {
        let (a, b, c) = (a.clone(), b.clone(), c.clone());
        let got = netted(workers, move |worker| chain_dataflow(worker, &a, &b, &c));
        assert_eq!(got.len(), KEYS as usize, "{workers} workers");
        assert_eq!(got, want, "{workers} workers");
    }
}

/// `carriers` as `(node, 0)` weighted by the node's double, iterated: each round joins the rows
/// under round 3 with the node's `Exact`-weighted factor and advances the round, so the fixed
/// point holds `(node, k)` at weight `carrier × factorᵏ` for `k` in `0..=3`. Worker 0 feeds both.
fn rounds_dataflow(
    worker: &mut Worker,
    carriers: &[Row],
    factors: &[Row],
) -> Captured<(u32, u8), Exact> {
    let (mut carriers_in, mut factors_in, captured) = worker.dataflow::<u64, _, _>(|scope| {
        let (carriers_in, carriers) = scope.new_collection::<Row, isize>();
        let (factors_in, factors) = scope.new_collection::<Row, isize>();
        let roots = carriers.explode(|(node, bits)| Some(((node, 0u8), e(f64::from_bits(bits)))));
        let factors = factors.explode(|(node, bits)| Some(((node, ()), e(f64::from_bits(bits)))));
        let rounds = roots.clone().iterate(|scope, inner| {
            let factors = factors.enter(scope);
            let roots = roots.enter(scope);
            inner
                .filter(|(_, round)| *round < 3)
                .join(factors)
                .map(|(node, (round, ()))| (node, round + 1))
                .concat(roots)
                .consolidate()
        });
        (carriers_in, factors_in, rounds.inner.capture())
    });
    if worker.index() == 0 {
        for row in carriers {
            carriers_in.insert(*row);
        }
        for row in factors {
            factors_in.insert(*row);
        }
    }
    carriers_in.close();
    factors_in.close();
    captured
}

/// `iterate` with a float carrier through three rounds of `Exact × Exact`: the variable needs
/// `Abelian` and each round's join lands on `Multiply<Exact, Output = Exact>`, and the weights of
/// rounds 2 and 3 span 159 and 212 bits, heap values riding the feedback loop, the arrangement
/// and the exchange. Each node's four weights are the oracle's `carrier × factorᵏ` in units of
/// `2^-1074(k+1)`, on one worker and three.
#[test]
#[cfg_attr(miri, ignore = "three timely workers exchanging serialized batches")]
fn differential_products_iterate_carries_a_float_weight_through_three_rounds_of_products() {
    const NODES: u32 = 12;
    let mut rng = Rng(66);
    let carriers: Vec<Row> = (0..NODES)
        .map(|node| (node, rng.on_grid(-300, 300).to_bits()))
        .collect();
    let factors: Vec<Row> = (0..NODES)
        .map(|node| (node, rng.on_grid(-100, 100).to_bits()))
        .collect();
    let mut want = Vec::new();
    let mut promoted = 0;
    for (&carrier, &factor) in carriers.iter().zip(&factors) {
        let (node, f) = (carrier.0, f64::from_bits(factor.1));
        let mut weight = amount(carrier);
        for round in 0..=3u8 {
            let w = Wide::try_from(&weight).expect("four factors of these bands lie inside Wide");
            assert_eq!(
                wide_units(&w) << (1074 * u32::from(round)),
                units(f64::from_bits(carrier.1)) * units(f).pow(u32::from(round)),
                "node {node} round {round}"
            );
            promoted += usize::from(!weight.is_compact());
            want.push(((node, round), 0, weight.clone()));
            weight = weight * e(f);
        }
    }
    assert_eq!(promoted, 2 * NODES as usize);
    let want = sorted(want);
    for workers in [1, 3] {
        let (carriers, factors) = (carriers.clone(), factors.clone());
        let got = netted(workers, move |worker| {
            rounds_dataflow(worker, &carriers, &factors)
        });
        assert_eq!(got.len(), 4 * NODES as usize, "{workers} workers");
        assert_eq!(got, want, "{workers} workers");
    }
}
