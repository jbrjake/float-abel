//! The crossover: the group size past which an exact float diff inside differential-dataflow,
//! with everything the engine does per change, beats re-summing the group in plain `f64`, which
//! is the question the crate's usefulness turns on. An engine cannot hold a float `SUM`
//! incrementally today (DESIGN.md § The problem), so the alternative to the crate is the
//! recompute: keep the group's floats and re-sum them whenever one changes, O(group) per change
//! and no engine around it.
//!
//! The incremental side is one timely worker. A group of `N` rows is preloaded under one key and
//! stepped to the probe, untimed; then one change per epoch is timed, each epoch stepped to the
//! probe before the next, the shape of an engine applying changes as they arrive. A change is an
//! insert and, in the next epoch, its retraction, so the group holds at `N` instead of growing
//! through the sample; an insert and a retraction cost the same (a retraction is the row at
//! multiplicity `-1`). Two dataflows see the same rows:
//!
//! - *exact*: `explode` the amount into an `Exact` diff, then `count_total`, the crate's idiom;
//! - *fixed-point*: the same with the amount scaled by 2²⁰ into an `isize` diff, the workaround
//!   that picks a range in advance, and the floor an integer diff sets.
//!
//! Two exchanges, as in `benches/dd_sum.rs`: serialized (`ProcessBinary`, every exchanged diff
//! crossing as its serde bytes) and typed (`Process`). A batched group repeats both at a thousand
//! changes an epoch, where the engine amortises an epoch over the batch. Both dataflows are
//! checked against an integer oracle before anything is timed.
//!
//! The recompute side has no engine: a `Vec<f64>` of the group re-summed per change, vectorised
//! as a column engine would and as the dependent chain a running total is, at every group size;
//! beside it one `Exact` deposit into a running slot, the incremental side with no engine either.
//! Advisory, never a gate: `cargo bench --bench crossover`, and the rows go into BENCH.md with the
//! machine and the commit.

// The oracle is integer arithmetic on units, cast on purpose; the fixed-point diff is the scaled
// amount truncated to the integer it already is.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss
)]

use std::hint::black_box;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use criterion::{
    criterion_group, criterion_main, BenchmarkId, Criterion, SamplingMode, Throughput,
};
use differential_dataflow::consolidation::consolidate;
use differential_dataflow::input::{Input, InputSession};
use differential_dataflow::operators::CountTotal;
use differential_dataflow::ExchangeData;
use float_abel::Exact;
use timely::dataflow::operators::capture::{Event, Extract};
use timely::dataflow::operators::Capture;
use timely::dataflow::ProbeHandle;
use timely::{CommunicationConfig, Config, WorkerConfig};

mod support;
use support::{amount, nth_units};

type Worker = timely::worker::Worker;
/// One row: the one account, and its amount as the double's bits, because a collection's data
/// must be `Ord` and `f64` is not.
type Row = (u32, u64);
type Session = InputSession<u64, Row, isize>;
/// One output update: data, epoch, diff.
type Update<D> = (D, u64, isize);
type Captured<D> = Receiver<Event<u64, Vec<Update<D>>>>;
/// What a dataflow builder hands back: the input, a probe on the output, and the output captured
/// for the oracle check.
type Built<D> = (Session, ProbeHandle<u64>, Captured<D>);
type Build<D> = fn(&mut Worker) -> Built<D>;

/// The group sizes, one change an epoch.
const SIZES: [u64; 5] = [100, 1_000, 10_000, 100_000, 1_000_000];
/// The group sizes at a thousand changes an epoch: from 10⁴ up, so a batch is at most a tenth of
/// the group.
const BATCHED_SIZES: [u64; 3] = [10_000, 100_000, 1_000_000];
const BATCH: u64 = 1_000;
/// 2²⁰: the fixed-point scale, which turns every amount into the integer count of units it is.
const SCALE: f64 = 1_048_576.0;

/// The `i`-th row: the one account, and the fixture's `i`-th amount.
fn row(i: u64) -> Row {
    (0, amount(nth_units(i)).to_bits())
}

/// The crate's path: the amount as an `Exact` diff, and `count_total` reads the running total out
/// as data with multiplicity one.
fn exact(worker: &mut Worker) -> Built<(u32, Exact)> {
    worker.dataflow::<u64, _, _>(|scope| {
        let (input, rows) = scope.new_collection::<Row, isize>();
        let (probe, totals) = rows
            .explode(|(account, bits)| Some((account, Exact::from(f64::from_bits(bits)))))
            .count_total()
            .probe();
        (input, probe, totals.inner.capture())
    })
}

/// The fixed-point workaround: the amount scaled to its count of units as an `isize` diff.
fn fixed_point(worker: &mut Worker) -> Built<(u32, isize)> {
    worker.dataflow::<u64, _, _>(|scope| {
        let (input, rows) = scope.new_collection::<Row, isize>();
        let (probe, totals) = rows
            .explode(|(account, bits)| Some((account, (f64::from_bits(bits) * SCALE) as isize)))
            .count_total()
            .probe();
        (input, probe, totals.inner.capture())
    })
}

/// How a diff crosses the worker's own exchange channel.
#[derive(Clone, Copy)]
enum Exchange {
    /// `ProcessBinary`: every exchanged update is serialized to bytes and decoded, on one worker
    /// as on many.
    Serialized,
    /// `Process`: typed channels carry the word itself.
    Typed,
}

impl Exchange {
    const ALL: [Self; 2] = [Self::Serialized, Self::Typed];

    fn name(self) -> &'static str {
        match self {
            Self::Serialized => "serialized",
            Self::Typed => "typed",
        }
    }

    fn config(self) -> CommunicationConfig {
        match self {
            Self::Serialized => CommunicationConfig::ProcessBinary(1),
            Self::Typed => CommunicationConfig::Process(1),
        }
    }
}

/// One sitting on one worker: build the dataflow, preload `preload` rows in epoch 0 and step to
/// the probe, then time `epochs` epochs of `batch` changes each, every epoch stepped to the probe
/// before the next. Even epochs insert fresh rows and odd epochs retract the rows the epoch before
/// inserted. Returns the time the timed epochs took and every output update the dataflow emitted.
fn run<D: ExchangeData>(
    exchange: Exchange,
    preload: u64,
    batch: u64,
    epochs: u64,
    build: Build<D>,
) -> (Duration, Vec<Update<D>>) {
    let config = Config {
        communication: exchange.config(),
        worker: WorkerConfig::default(),
    };
    let guards = timely::execute(config, move |worker| {
        let (mut input, probe, captured) = build(worker);
        for i in 0..preload {
            input.insert(row(i));
        }
        let mut epoch = 1;
        input.advance_to(epoch);
        input.flush();
        worker.step_while(|| probe.less_than(input.time()));

        let start = Instant::now();
        for e in 0..epochs {
            let fresh = preload + (e / 2) * batch;
            for i in fresh..fresh + batch {
                if e % 2 == 0 {
                    input.insert(row(i));
                } else {
                    input.remove(row(i));
                }
            }
            epoch += 1;
            input.advance_to(epoch);
            input.flush();
            worker.step_while(|| probe.less_than(input.time()));
        }
        let elapsed = start.elapsed();

        input.close();
        while worker.step() {}
        let updates: Vec<Update<D>> = captured
            .extract()
            .into_iter()
            .flat_map(|(_, batch)| batch)
            .collect();
        (elapsed, updates)
    })
    .expect("the worker starts");
    guards
        .join()
        .into_iter()
        .next()
        .expect("one worker")
        .expect("the worker finishes")
}

/// The output netted over every epoch: equal data summed, zeros dropped.
fn net<D: Ord + Clone>(updates: Vec<Update<D>>) -> Vec<(D, isize)> {
    let mut net: Vec<(D, isize)> = updates.into_iter().map(|(d, _, r)| (d, r)).collect();
    consolidate(&mut net);
    net
}

/// Both dataflows against the integer oracle on both exchanges, one change an epoch and a batch
/// an epoch: the output count is one total in epoch 0 and a retraction and an insertion in every
/// timed epoch, and the net over every epoch is the preloaded group's exact total alone, because
/// each inserted row was retracted.
fn check() {
    const PRELOAD: u64 = 1_000;
    const EPOCHS: u64 = 10;
    let total_units: i64 = (0..PRELOAD).map(nth_units).sum();
    for batch in [1, BATCH] {
        // A change of zero would leave a total unchanged and the output count short.
        for e in 0..EPOCHS / 2 {
            let fresh = PRELOAD + e * batch;
            let change: i64 = (fresh..fresh + batch).map(nth_units).sum();
            assert_ne!(change, 0, "epoch {e} at batch {batch} changes nothing");
        }
        for exchange in Exchange::ALL {
            let expected = 1 + 2 * EPOCHS as usize;

            let (_, out) = run(exchange, PRELOAD, batch, EPOCHS, exact);
            assert_eq!(out.len(), expected, "exact, {}", exchange.name());
            assert_eq!(net(out), vec![((0, Exact::from(amount(total_units))), 1)]);

            let (_, out) = run(exchange, PRELOAD, batch, EPOCHS, fixed_point);
            assert_eq!(out.len(), expected, "fixed-point, {}", exchange.name());
            assert_eq!(net(out), vec![((0, total_units as isize), 1)]);
        }
    }
}

/// Both dataflows at every group size and exchange, `batch` changes an epoch. The reported time
/// is per epoch; with `Throughput::Elements(batch)` the thrpt line is changes a second.
fn dataflows(c: &mut Criterion, group: &str, sizes: &[u64], batch: u64) {
    let mut g = c.benchmark_group(group);
    g.sampling_mode(SamplingMode::Flat);
    g.throughput(Throughput::Elements(batch));
    for &n in sizes {
        for exchange in Exchange::ALL {
            let name = exchange.name();
            g.bench_with_input(BenchmarkId::new(format!("exact/{name}"), n), &n, |b, &n| {
                b.iter_custom(|epochs| run(exchange, n, batch, epochs, exact).0);
            });
            g.bench_with_input(
                BenchmarkId::new(format!("fixed-point/{name}"), n),
                &n,
                |b, &n| {
                    b.iter_custom(|epochs| run(exchange, n, batch, epochs, fixed_point).0);
                },
            );
        }
    }
    g.finish();
}

fn one_change_an_epoch(c: &mut Criterion) {
    check();
    dataflows(c, "crossover", &SIZES, 1);
}

fn a_batch_an_epoch(c: &mut Criterion) {
    dataflows(c, "crossover/batched", &BATCHED_SIZES, BATCH);
}

/// The naive `f64` sum with eight independent accumulators, the shape a column engine re-sums a
/// block in, remainder included.
fn vectorised(xs: &[f64]) -> f64 {
    let mut acc = [0.0f64; 8];
    let chunks = xs.chunks_exact(8);
    let rest = chunks.remainder();
    for chunk in chunks {
        for (a, &x) in acc.iter_mut().zip(chunk) {
            *a += x;
        }
    }
    acc.iter().sum::<f64>() + rest.iter().sum::<f64>()
}

/// The recompute: a `Vec<f64>` of the group re-summed per change, vectorised and as the dependent
/// chain, at every group size. The reported time is one change's.
fn recompute(c: &mut Criterion) {
    let mut g = c.benchmark_group("crossover/recompute");
    for n in SIZES {
        let group: Vec<f64> = (0..n).map(|i| amount(nth_units(i))).collect();
        g.bench_with_input(
            BenchmarkId::new("f64 re-sum, vectorised", n),
            &group,
            |b, group| b.iter(|| vectorised(black_box(group))),
        );
        g.bench_with_input(
            BenchmarkId::new("f64 re-sum, chain", n),
            &group,
            |b, group| {
                b.iter(|| black_box(group).iter().sum::<f64>());
            },
        );
    }
    g.finish();
}

/// The incremental side with no engine around it: one `Exact` deposit into a running slot per
/// change, the thrpt line giving changes a second.
fn deposit(c: &mut Criterion) {
    let changes: Vec<f64> = (0..support::N as u64)
        .map(|i| amount(nth_units(i)))
        .collect();
    let mut g = c.benchmark_group("crossover/no-engine");
    g.throughput(Throughput::Elements(changes.len() as u64));
    g.bench_function("Exact deposit, one change", |b| {
        b.iter(|| {
            let mut slot = Exact::ZERO;
            for &x in &changes {
                slot += Exact::from(x);
            }
            slot
        });
    });
    g.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(5));
    targets = one_change_an_epoch, a_batch_an_epoch, recompute, deposit
}
criterion_main!(benches);
