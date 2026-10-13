//! `Exact` as a differential-dataflow diff, DESIGN.md § differential-dataflow: three workers with
//! serialized exchange (`CommunicationConfig::ProcessBinary(3)`), every operator the design lists,
//! the `explode` + `count_total` idiom, an `antijoin` with `isize`, and a retraction epoch whose
//! netted output is empty. Every test is named `differential_*` so the task verify can filter on
//! it.
//!
//! A row carries its amount as the double's bits, because a collection's data must be `Ord` and
//! `f64` is not; `explode` turns the bits back into a double and the double into the diff.

use std::sync::mpsc::Receiver;

use differential_dataflow::consolidation::consolidate_updates;
use differential_dataflow::difference::Semigroup;
use differential_dataflow::input::Input;
use differential_dataflow::operators::{CountTotal, Iterate, ThresholdTotal};
use differential_dataflow::{ExchangeData, VecCollection};
use float_abel::Exact;
use timely::dataflow::operators::capture::{Event, Extract};
use timely::dataflow::operators::Capture;
use timely::{CommunicationConfig, Config, WorkerConfig};

use crate::support::pow2;

type Worker = timely::worker::Worker;
/// One update: data, epoch, diff.
type Update<D, R> = (D, u64, R);
type Captured<D, R> = Receiver<Event<u64, Vec<Update<D, R>>>>;
/// An account and an amount, the amount as the double's bits.
type Row = (u32, u64);
/// The fixture the operator tests share: `(key, value)` rows weighted by `Exact`.
type Fixture<'scope> = VecCollection<'scope, u64, (u32, u32), Exact>;

fn e(x: f64) -> Exact {
    Exact::from(x)
}

/// Runs `build` on `workers` worker threads with serialized exchange, the shape the design's probe
/// ran on, and returns every update the captured stream emitted on any worker, netted: sorted,
/// equal `(data, epoch)` summed, zeros dropped. `build` makes the dataflow, feeds this worker's
/// share of the input and closes it; the harness steps the worker to completion before it reads.
fn netted<D, R, F>(workers: usize, build: F) -> Vec<Update<D, R>>
where
    D: ExchangeData,
    R: ExchangeData + Semigroup,
    F: Fn(&mut Worker) -> Captured<D, R> + Send + Sync + 'static,
{
    let config = Config {
        communication: CommunicationConfig::ProcessBinary(workers),
        worker: WorkerConfig::default(),
    };
    let guards = timely::execute(config, move |worker| {
        let captured = build(worker);
        while worker.step() {}
        captured.extract()
    })
    .expect("the workers start");
    let mut updates: Vec<Update<D, R>> = guards
        .join()
        .into_iter()
        .map(|finished| finished.expect("every worker finishes"))
        .flat_map(|events| events.into_iter().flat_map(|(_, batch)| batch))
        .collect();
    consolidate_updates(&mut updates);
    updates
}

/// `expected`, in the order `netted` returns.
fn sorted<D: Ord, R: Semigroup>(mut expected: Vec<Update<D, R>>) -> Vec<Update<D, R>> {
    consolidate_updates(&mut expected);
    expected
}

/// This worker's share of `rows`: every `peers`-th row from `index`, so three workers feed three
/// interleaved thirds and exchange reassembles the accounts.
fn share<'a>(worker: &Worker, rows: &'a [Row]) -> impl Iterator<Item = &'a Row> {
    let (index, peers) = (worker.index(), worker.peers());
    rows.iter().skip(index).step_by(peers)
}

/// The SUM dataflow, the idiom the design documents: `explode` the amount into the diff, then
/// `count_total`, which carries each account's total as data with multiplicity one. This worker
/// feeds its share of `inserts` at epoch 0, retracts its share of `retracts` at epoch 1, and
/// closes the input.
fn sum_dataflow(
    worker: &mut Worker,
    inserts: &[Row],
    retracts: &[Row],
) -> Captured<(u32, Exact), isize> {
    let (mut input, captured) = worker.dataflow::<u64, _, _>(|scope| {
        let (input, rows) = scope.new_collection::<Row, isize>();
        let totals = rows
            .explode(|(account, bits)| Some((account, e(f64::from_bits(bits)))))
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

/// `n` rows over sixteen accounts, each amount a signed count of units of `2^-20`, below `2^20` in
/// magnitude. The exact total of any subset is an integer count of units an `i64` holds, which is
/// the oracle.
fn units(n: u64) -> Vec<(u32, i64)> {
    (0..n)
        .map(|i| {
            let account = ((i * 7919) % 16) as u32;
            let units = ((i * 2_654_435_761) % (1 << 21)) as i64 - (1 << 20);
            (account, units)
        })
        .collect()
}

/// `units × 2^-20` as a double.
fn amount(units: i64) -> f64 {
    units as f64 * pow2(-20)
}

fn dyadic(ledger: &[(u32, i64)]) -> Vec<Row> {
    ledger
        .iter()
        .map(|&(account, units)| (account, amount(units).to_bits()))
        .collect()
}

/// The exact total per account, by integer arithmetic on the units; the bits the dataflow must
/// land on. One `From<f64>` of an exactly representable double, never a float sum.
fn oracle(ledger: &[(u32, i64)]) -> Vec<(u32, Exact)> {
    let mut totals = [0i64; 16];
    for &(account, units) in ledger {
        totals[account as usize] += units;
    }
    totals
        .iter()
        .enumerate()
        .map(|(account, &units)| (account as u32, e(amount(units))))
        .collect()
}

/// `n` rows over sixteen accounts with amounts in tenths, `-4.8` to `4.8`: values an `f64` sum does
/// not hold exactly, so the order of a fold shows in its bits.
fn tenths(n: u64) -> Vec<Row> {
    (0..n)
        .map(|i| {
            let account = ((i * 7919) % 16) as u32;
            let amount = (((i * 31) % 97) as f64 - 48.0) / 10.0;
            (account, amount.to_bits())
        })
        .collect()
}

/// Runs `op` over the fixture on three workers and returns the netted output. Worker 0 feeds every
/// row and exchange spreads them; the pair on key 3 cancels to the one zero, which `is_zero`
/// deletes before any operator sees the key.
fn over_fixture<D, R, F>(op: F) -> Vec<Update<D, R>>
where
    D: ExchangeData,
    R: ExchangeData + Semigroup,
    F: for<'scope> Fn(Fixture<'scope>) -> VecCollection<'scope, u64, D, R> + Send + Sync + 'static,
{
    netted(3, move |worker| {
        let (mut input, captured) = worker.dataflow::<u64, _, _>(|scope| {
            let (input, weighted) = scope.new_collection::<(u32, u32), Exact>();
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
                input.update(row, e(weight));
            }
        }
        input.close();
        captured
    })
}

/// Key 1's weight on value 10 after the fixture consolidates.
fn w110() -> Exact {
    e(0.1) + e(0.2)
}

#[test]
#[cfg_attr(miri, ignore = "three timely workers exchanging serialized batches")]
fn differential_explode_and_count_total_land_the_oracle_bits_on_three_workers() {
    let ledger = units(2_000);
    let expected = sorted(oracle(&ledger).into_iter().map(|d| (d, 0, 1)).collect());
    let rows = dyadic(&ledger);
    let out = netted(3, move |worker| sum_dataflow(worker, &rows, &[]));
    assert_eq!(out.len(), 16);
    assert_eq!(out, expected);
}

/// One worker and three, the rows as drawn and reversed: four runs, one answer, byte for byte.
#[test]
#[cfg_attr(miri, ignore = "three timely workers exchanging serialized batches")]
fn differential_worker_count_and_input_order_do_not_move_the_bits() {
    let forward = tenths(2_000);
    let mut backward = forward.clone();
    backward.reverse();
    let runs = [(1, &forward), (3, &forward), (1, &backward), (3, &backward)];
    let outputs: Vec<Vec<[u8; Exact::BYTES]>> = runs
        .into_iter()
        .map(|(workers, rows)| {
            let rows = rows.clone();
            netted(workers, move |worker| sum_dataflow(worker, &rows, &[]))
                .into_iter()
                .map(|((_, total), _, _)| total.to_bytes())
                .collect()
        })
        .collect();
    assert_eq!(outputs[0].len(), 16);
    assert!(outputs.iter().all(|bytes| *bytes == outputs[0]));
}

/// Epoch 0 inserts the rows and epoch 1 retracts every one of them, each retraction arriving as
/// `-1 × Exact` through `Multiply<isize>`. Epoch 1's output is the retraction of epoch 0's totals
/// and nothing else, and the two epochs net to nothing: every total cancelled to the one zero,
/// which `is_zero` deleted. The control folds the same rows in `f64`, where the same inserts and
/// retractions leave residue.
#[test]
#[cfg_attr(miri, ignore = "three timely workers exchanging serialized batches")]
fn differential_a_retraction_epoch_nets_to_nothing_where_f64_leaves_residue() {
    let rows = tenths(2_000);
    let mut folded = [0.0f64; 16];
    for &(account, bits) in &rows {
        folded[account as usize] += f64::from_bits(bits);
    }
    for &(account, bits) in &rows {
        folded[account as usize] -= f64::from_bits(bits);
    }
    assert!(
        folded.iter().any(|residue| *residue != 0.0),
        "the f64 control left no residue, so these rows do not show the difference"
    );

    let out = netted(3, move |worker| sum_dataflow(worker, &rows, &rows));
    let (epoch0, epoch1): (Vec<_>, Vec<_>) = out.iter().copied().partition(|(_, t, _)| *t == 0);
    assert_eq!(epoch0.len(), 16);
    let retracted: Vec<_> = epoch0.iter().map(|(d, _, r)| (*d, 1, -r)).collect();
    assert_eq!(epoch1, retracted);
    let mut net: Vec<((u32, Exact), (), isize)> =
        out.into_iter().map(|(d, _, r)| (d, (), r)).collect();
    consolidate_updates(&mut net);
    assert!(net.is_empty(), "residue: {net:?}");
}

/// `From<f64>` is total, so a NaN or an infinity is a row like any other: counted into the diff,
/// exchanged, summed with multiplicity, and retracted exactly, where an `f64` diff could never
/// retract a NaN. The duplicate row arrives with multiplicity two and scales through
/// `Multiply<isize>`.
#[test]
#[cfg_attr(miri, ignore = "three timely workers exchanging serialized batches")]
fn differential_specials_ride_the_diff_and_retract_exactly() {
    let rows: Vec<Row> = [1.5, f64::NAN, f64::INFINITY, 2.5, 2.5]
        .map(|x| (7, x.to_bits()))
        .to_vec();
    let retract = vec![(7, f64::NAN.to_bits())];
    let out = netted(3, move |worker| sum_dataflow(worker, &rows, &retract));
    let with_nan = e(1.5) + e(f64::NAN) + e(f64::INFINITY) + e(2.5) * 2i64;
    let without = e(1.5) + e(f64::INFINITY) + e(2.5) * 2i64;
    assert!(with_nan.to_f64().is_nan());
    assert_eq!(with_nan.specials(), (1, 1, 0));
    assert_eq!(without.to_f64(), f64::INFINITY);
    assert_eq!(without.specials(), (0, 1, 0));
    assert_eq!(
        out,
        sorted(vec![
            ((7, with_nan), 0, 1),
            ((7, with_nan), 1, -1),
            ((7, without), 1, 1),
        ])
    );
}

/// The linear operators and both arrangements on a collection whose diff is `Exact`: `map`,
/// `flat_map`, `concat`, `negate` and `consolidate`, then `arrange_by_key` and `arrange_by_self`
/// read back as collections. A collection concatenated with its own negation consolidates to
/// nothing, and the rest is a known multiset.
#[test]
#[cfg_attr(miri, ignore = "three timely workers exchanging serialized batches")]
fn differential_linear_operators_and_arrangements_carry_exact_diffs() {
    let out = over_fixture(|weighted| {
        let doubled = weighted.clone().map(|(k, v)| (k, v * 2));
        let split = weighted.clone().flat_map(|(k, v)| [(k, v), (k + 100, v)]);
        let cancelled = weighted
            .clone()
            .concat(weighted.clone().negate())
            .consolidate();
        let by_key = weighted
            .clone()
            .arrange_by_key()
            .as_collection(|k, v| (*k, *v));
        let by_self = weighted
            .map(|(k, _)| k)
            .arrange_by_self()
            .as_collection(|k, ()| (*k, 0));
        doubled
            .concat(split)
            .concat(cancelled)
            .concat(by_key)
            .concat(by_self)
            .consolidate()
    });
    assert_eq!(
        out,
        sorted(vec![
            ((1, 0), 0, w110() + e(1.0)),
            ((1, 10), 0, w110() + w110()),
            ((1, 11), 0, e(1.0) + e(1.0)),
            ((1, 20), 0, w110()),
            ((1, 22), 0, e(1.0)),
            ((101, 10), 0, w110()),
            ((101, 11), 0, e(1.0)),
            ((2, 0), 0, e(-0.3)),
            ((2, 20), 0, e(-0.3) + e(-0.3)),
            ((2, 40), 0, e(-0.3)),
            ((102, 20), 0, e(-0.3)),
        ])
    );
}

/// `reduce` with `Exact` as the output diff (each key's total weight) and with `isize` as the
/// output diff (how many values the key has).
#[test]
#[cfg_attr(miri, ignore = "three timely workers exchanging serialized batches")]
fn differential_reduce_puts_exact_or_isize_out() {
    let totals = over_fixture(|weighted| {
        weighted.reduce(|_k, values, output: &mut Vec<(u8, Exact)>| {
            let total = values.iter().fold(Exact::ZERO, |t, (_, w)| t + *w);
            output.push((0, total));
        })
    });
    assert_eq!(
        totals,
        sorted(vec![((1, 0u8), 0, w110() + e(1.0)), ((2, 0), 0, e(-0.3)),])
    );

    let arity = over_fixture(|weighted| {
        weighted.reduce(|_k, values, output: &mut Vec<(u8, isize)>| {
            output.push((values.len() as u8, 1));
        })
    });
    assert_eq!(arity, sorted(vec![((1, 2u8), 0, 1), ((2, 1), 0, 1)]));
}

/// `count` and `count_total` carry a key's `Exact` total as data with an `isize` diff;
/// `count_core::<Exact>` and `distinct_core::<Exact>` put `Exact::from(1i8)` out; `threshold` and
/// `threshold_total` map an `Exact` weight to an `Exact` weight.
#[test]
#[cfg_attr(miri, ignore = "three timely workers exchanging serialized batches")]
fn differential_count_distinct_and_threshold_take_exact() {
    let total = w110() + e(1.0);
    let counted = over_fixture(|weighted| weighted.map(|(k, _)| k).count());
    assert_eq!(
        counted,
        sorted(vec![((1, total), 0, 1), ((2, e(-0.3)), 0, 1)])
    );
    let counted_total = over_fixture(|weighted| weighted.map(|(k, _)| k).count_total());
    assert_eq!(counted_total, counted);

    let counted_exact = over_fixture(|weighted| weighted.map(|(k, _)| k).count_core::<Exact>());
    assert_eq!(
        counted_exact,
        sorted(vec![((1, total), 0, e(1.0)), ((2, e(-0.3)), 0, e(1.0))])
    );
    let distinct = over_fixture(|weighted| weighted.map(|(k, _)| k).distinct_core::<Exact>());
    assert_eq!(distinct, sorted(vec![(1, 0, e(1.0)), (2, 0, e(1.0))]));

    let doubled = over_fixture(|weighted| weighted.map(|(k, _)| k).threshold(|_k, w| *w * 2i64));
    assert_eq!(
        doubled,
        sorted(vec![(1, 0, total * 2i64), (2, 0, e(-0.3) * 2i64)])
    );
    let doubled_total = over_fixture(|weighted| {
        weighted
            .map(|(k, _)| k)
            .threshold_total(|_k, w: &Exact| *w * 2i64)
    });
    assert_eq!(doubled_total, doubled);
}

/// The keys of the fixture with `isize` multiplicities, key 2 twice: the other side of every join.
fn keys<'scope>(weighted: &Fixture<'scope>) -> VecCollection<'scope, u64, u32, isize> {
    let keys = weighted.clone().map(|(k, _)| k).distinct();
    keys.clone().concat(keys.filter(|k| *k == 2))
}

/// `join` with `Exact` on the left and `isize` on the right, and the other way round: both land
/// on `Multiply<isize> for Exact` and `Multiply<Exact> for isize`, and the multiplicity scales
/// the weight.
#[test]
#[cfg_attr(miri, ignore = "three timely workers exchanging serialized batches")]
fn differential_join_multiplies_exact_by_isize_in_either_order() {
    let exact_times_isize = over_fixture(|weighted| {
        let tagged = keys(&weighted).map(|k| (k, k * 100));
        weighted.join(tagged)
    });
    assert_eq!(
        exact_times_isize,
        sorted(vec![
            ((1, (10, 100)), 0, w110()),
            ((1, (11, 100)), 0, e(1.0)),
            ((2, (20, 200)), 0, e(-0.3) * 2i64),
        ])
    );

    let isize_times_exact = over_fixture(|weighted| {
        let tagged = keys(&weighted).map(|k| (k, k * 100));
        tagged.join(weighted)
    });
    assert_eq!(
        isize_times_exact,
        sorted(vec![
            ((1, (100, 10)), 0, w110()),
            ((1, (100, 11)), 0, e(1.0)),
            ((2, (200, 20)), 0, e(-0.3) * 2i64),
        ])
    );
}

/// `semijoin` keeps the rows whose key the `isize` collection has, scaled by its multiplicity;
/// `antijoin` keeps the rows whose key it lacks, through `Multiply<isize, Output = Exact>`.
#[test]
#[cfg_attr(miri, ignore = "three timely workers exchanging serialized batches")]
fn differential_semijoin_and_antijoin_with_isize() {
    let kept = over_fixture(|weighted| {
        let keys = keys(&weighted);
        weighted.semijoin(keys)
    });
    assert_eq!(
        kept,
        sorted(vec![
            ((1, 10), 0, w110()),
            ((1, 11), 0, e(1.0)),
            ((2, 20), 0, e(-0.3) * 2i64),
        ])
    );

    let rest = over_fixture(|weighted| {
        let one = keys(&weighted).filter(|k| *k == 1);
        weighted.antijoin(one)
    });
    assert_eq!(rest, sorted(vec![((2, 20), 0, e(-0.3))]));
}

/// Reachability by `iterate`, whose variable needs `Abelian`: the roots carry `Exact` weights
/// into a nested scope, each step joins them with `isize` edges and `distinct_core::<Exact>`
/// normalises, and the fixed point leaves with every reachable node at weight one.
#[test]
#[cfg_attr(miri, ignore = "three timely workers exchanging serialized batches")]
fn differential_iterate_carries_exact_diffs() {
    let out = netted(3, |worker| {
        let (mut roots, mut edges, captured) = worker.dataflow::<u64, _, _>(|scope| {
            let (roots_in, roots) = scope.new_collection::<u32, Exact>();
            let (edges_in, edges) = scope.new_collection::<(u32, u32), isize>();
            let reached = roots.clone().iterate(|scope, inner| {
                let edges = edges.enter(scope);
                let roots = roots.enter(scope);
                inner
                    .map(|k| (k, ()))
                    .join_map(edges, |_src, (), dst| *dst)
                    .concat(roots)
                    .distinct_core::<Exact>()
            });
            (roots_in, edges_in, reached.inner.capture())
        });
        if worker.index() == 0 {
            roots.update(1, e(0.5));
            for edge in [(1, 2), (2, 3), (3, 1), (3, 4), (5, 6)] {
                edges.insert(edge);
            }
        }
        roots.close();
        edges.close();
        captured
    });
    assert_eq!(
        out,
        sorted(vec![
            (1, 0, e(1.0)),
            (2, 0, e(1.0)),
            (3, 0, e(1.0)),
            (4, 0, e(1.0)),
        ])
    );
}

/// `From<i8>` is the integer, which is what `count_core` and `distinct_core` put out as one.
#[test]
fn differential_from_i8_is_the_integer() {
    for k in [i8::MIN, -1, 0, 1, 2, i8::MAX] {
        assert_eq!(Exact::from(k), e(f64::from(k)), "{k}");
    }
    assert!(Exact::from(0i8).is_zero());
    assert_eq!(Exact::from(1i8) * 3i64, e(3.0));
}
