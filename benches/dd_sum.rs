//! The end-to-end kill condition, DESIGN.md § Targets and kill conditions: a differential-dataflow
//! `SUM` over 10⁷ updates with `Exact` as the diff against the same dataflow with `isize`, at one
//! worker and at eight. The fixture is the integration harness's same-scale one
//! (tests/differential.rs): sixteen accounts, each amount a signed count of units of 2⁻²⁰ below
//! 2²⁰, which `Exact` carries as the double and `isize` as the count, so both runs fold one
//! multiset and an integer oracle checks the `Exact` totals bit for bit before anything is timed.
//! The dataflow is the design's idiom, `explode` into the diff then `count_total`, run twice over:
//! on serialized exchange (`ProcessBinary`, the shape the probe ran on, where every exchanged diff
//! crosses the wire as its serde bytes even on one worker) and on typed exchange (`Process`, the
//! in-process deployment, where it crosses as the word), so the two ratios split the diff's
//! arithmetic from its serialization. Advisory, never a gate: `cargo bench --bench dd_sum`, and
//! the numbers go into BENCH.md with the machine and the commit. Peak RSS comes from
//! `/usr/bin/time` around the bench binary, not from here.
//!
//! A sample is one whole run, workers spawned to workers joined, so the harness takes ten of them
//! flat rather than criterion's hundred.

// The fixture is integer arithmetic on indices and units, cast on purpose.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss
)]

use std::hint::black_box;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion, SamplingMode, Throughput};
use differential_dataflow::consolidation::consolidate_updates;
use differential_dataflow::difference::Semigroup;
use differential_dataflow::input::Input;
use differential_dataflow::operators::CountTotal;
use differential_dataflow::ExchangeData;
use float_abel::Exact;
use timely::dataflow::operators::capture::{Event, Extract};
use timely::dataflow::operators::Capture;
use timely::{CommunicationConfig, Config, WorkerConfig};

/// The design's update count.
const UPDATES: u64 = 10_000_000;
const ACCOUNTS: u64 = 16;
/// Workers per run: the one-worker shape, and the eight-worker shape the design names.
const WORKERS: [usize; 2] = [1, 8];

type Worker = timely::worker::Worker;
/// One update: data, epoch, diff.
type Update<D, R> = (D, u64, R);
type Captured<D, R> = Receiver<Event<u64, Vec<Update<D, R>>>>;
/// An account and an amount as a signed count of units of 2⁻²⁰.
type Row = (u32, i64);

/// `n` rows over sixteen accounts, each amount a signed count of units of 2⁻²⁰ below 2²⁰ in
/// magnitude: the integration harness's fixture at the design's count.
fn units(n: u64) -> Vec<Row> {
    (0..n)
        .map(|i| {
            let account = ((i * 7919) % ACCOUNTS) as u32;
            let units = ((i * 2_654_435_761) % (1 << 21)) as i64 - (1 << 20);
            (account, units)
        })
        .collect()
}

/// `units × 2⁻²⁰` as a double, exactly: the power of two by its bits, never `powi`.
fn amount(units: i64) -> f64 {
    units as f64 * f64::from_bits(((-20i64 + 1023) as u64) << 52)
}

/// The exact total per account by integer arithmetic on the units.
fn oracle(rows: &[Row]) -> Vec<(u32, i64)> {
    let mut totals = [0i64; ACCOUNTS as usize];
    for &(account, units) in rows {
        totals[account as usize] += units;
    }
    totals
        .iter()
        .enumerate()
        .map(|(account, &units)| (account as u32, units))
        .collect()
}

/// This worker's share of `rows`: every `peers`-th row from `index`.
fn share<'a>(worker: &Worker, rows: &'a [Row]) -> impl Iterator<Item = &'a Row> {
    let (index, peers) = (worker.index(), worker.peers());
    rows.iter().skip(index).step_by(peers)
}

/// The SUM with `Exact` as the diff: `explode` the amount into the diff, then `count_total`
/// carries each account's exact total as data with multiplicity one.
fn exact_sum(worker: &mut Worker, rows: &[Row]) -> Captured<(u32, Exact), isize> {
    let (mut input, captured) = worker.dataflow::<u64, _, _>(|scope| {
        let (input, rows) = scope.new_collection::<Row, isize>();
        let totals = rows
            .explode(|(account, units)| Some((account, Exact::from(amount(units)))))
            .count_total();
        (input, totals.inner.capture())
    });
    for row in share(worker, rows) {
        input.insert(*row);
    }
    input.close();
    captured
}

/// The same SUM with `isize` as the diff, the baseline: the count of units is the diff.
fn isize_sum(worker: &mut Worker, rows: &[Row]) -> Captured<(u32, isize), isize> {
    let (mut input, captured) = worker.dataflow::<u64, _, _>(|scope| {
        let (input, rows) = scope.new_collection::<Row, isize>();
        let totals = rows
            .explode(|(account, units)| Some((account, units as isize)))
            .count_total();
        (input, totals.inner.capture())
    });
    for row in share(worker, rows) {
        input.insert(*row);
    }
    input.close();
    captured
}

/// How a diff crosses between workers.
#[derive(Clone, Copy)]
enum Exchange {
    /// `ProcessBinary`: every exchanged update is serialized to bytes and decoded, on one worker
    /// as on eight.
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

    fn config(self, workers: usize) -> CommunicationConfig {
        match self {
            Self::Serialized => CommunicationConfig::ProcessBinary(workers),
            Self::Typed => CommunicationConfig::Process(workers),
        }
    }
}

/// One whole run: `workers` workers on the given exchange, each building the dataflow and feeding
/// its share, stepped to completion and joined. Returns every update the captured stream emitted
/// on any worker, consolidated.
fn run<D, R, F>(workers: usize, exchange: Exchange, build: F) -> Vec<Update<D, R>>
where
    D: ExchangeData,
    R: ExchangeData + Semigroup,
    F: Fn(&mut Worker) -> Captured<D, R> + Send + Sync + 'static,
{
    let config = Config {
        communication: exchange.config(workers),
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

/// Both dataflows on every worker count and exchange, after one checked run of each against the
/// oracle.
fn sum(c: &mut Criterion) {
    let rows = Arc::new(units(UPDATES));
    let expected = oracle(&rows);

    for exchange in Exchange::ALL {
        for workers in WORKERS {
            let shared = Arc::clone(&rows);
            let exact = run(workers, exchange, move |worker| exact_sum(worker, &shared));
            let totals: Vec<(u32, Exact)> = exact.into_iter().map(|(d, _, _)| d).collect();
            let oracle: Vec<(u32, Exact)> = expected
                .iter()
                .map(|&(account, units)| (account, Exact::from(amount(units))))
                .collect();
            assert_eq!(totals.len(), ACCOUNTS as usize);
            assert_eq!(totals, oracle, "Exact totals on {workers} workers");

            let shared = Arc::clone(&rows);
            let baseline = run(workers, exchange, move |worker| isize_sum(worker, &shared));
            let totals: Vec<(u32, isize)> = baseline.into_iter().map(|(d, _, _)| d).collect();
            let oracle: Vec<(u32, isize)> = expected
                .iter()
                .map(|&(account, units)| (account, units as isize))
                .collect();
            assert_eq!(totals, oracle, "isize totals on {workers} workers");
        }
    }

    let mut g = c.benchmark_group("dd_sum");
    g.throughput(Throughput::Elements(UPDATES));
    g.sampling_mode(SamplingMode::Flat);
    for exchange in Exchange::ALL {
        for workers in WORKERS {
            let name = exchange.name();
            let shared = Arc::clone(&rows);
            g.bench_function(format!("exact/{workers}w/{name}"), |b| {
                b.iter(|| {
                    let rows = Arc::clone(&shared);
                    black_box(run(workers, exchange, move |worker| {
                        exact_sum(worker, &rows)
                    }))
                });
            });
            let shared = Arc::clone(&rows);
            g.bench_function(format!("isize/{workers}w/{name}"), |b| {
                b.iter(|| {
                    let rows = Arc::clone(&shared);
                    black_box(run(workers, exchange, move |worker| {
                        isize_sum(worker, &rows)
                    }))
                });
            });
        }
    }
    g.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(30));
    targets = sum
}
criterion_main!(benches);
