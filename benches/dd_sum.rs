//! The end-to-end kill condition, DESIGN.md § Targets and kill conditions: a differential-dataflow
//! `SUM` over 10⁷ updates with `Exact` as the diff against the same dataflow with `isize`, at one
//! worker and at eight, and beside it the banded idiom (docs/OVERFLOW.md § The banded idiom) and
//! `Wide` as the diff, the `Copy` word (DESIGN.md § Wide). Two fixtures. The same-scale one is the
//! integration harness's (tests/differential.rs): sixteen accounts, each amount a signed count of
//! units of 2⁻²⁰ below 2²⁰, which `Exact` and `Wide` carry as the double and `isize` as the count,
//! so every run folds one multiset and an integer oracle checks the `Exact` and `Wide` totals bit
//! for bit before anything is timed. The mixed one spreads the amounts over 2^±300, where the
//! plain idiom's running diffs promote to the heap form and the banded idiom's never do, checked
//! against a `Wide` oracle per account; `isize` and `Wide` have no row there, the one for want of
//! magnitudes to mix and the other because its cost does not depend on them. The plain idiom is
//! the design's, `explode` into the diff then `count_total`; the banded one is
//! `BandedSum::banded_sum`. Each runs twice over: on serialized exchange (`ProcessBinary`, the
//! shape the probe ran on, where every exchanged diff crosses the wire as its serde bytes even on
//! one worker) and on typed exchange (`Process`, the in-process deployment, where it crosses as
//! the word), so the two ratios split the diff's arithmetic from its serialization. Advisory,
//! never a gate: `cargo bench --bench dd_sum`, and the numbers go into BENCH.md with the machine
//! and the commit. Peak RSS comes from `/usr/bin/time` around the bench binary, not from here.
//!
//! A sample is one whole run, workers spawned to workers joined, so the harness takes ten of them
//! flat rather than criterion's hundred.

// The oracle is integer arithmetic on indices and units, cast on purpose.
#![allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]

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
use float_abel::{BandedSum, Exact, Wide};
use timely::dataflow::operators::capture::{Event, Extract};
use timely::dataflow::operators::Capture;
use timely::{CommunicationConfig, Config, WorkerConfig};

mod support;
use support::{amount, mixed, units, Bits, Row, ACCOUNTS};

/// The design's update count.
const UPDATES: u64 = 10_000_000;
/// Workers per run: the one-worker shape, and the eight-worker shape the design names.
const WORKERS: [usize; 2] = [1, 8];

type Worker = timely::worker::Worker;
/// One update: data, epoch, diff.
type Update<D, R> = (D, u64, R);
type Captured<D, R> = Receiver<Event<u64, Vec<Update<D, R>>>>;

/// The exact total per account by integer arithmetic on the units.
fn units_oracle(rows: &[Row]) -> Vec<(u32, i64)> {
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

/// The exact total per account through `Wide`, which holds every sum of doubles, as the `Exact`
/// both idioms must land on the mixed fixture.
fn wide_oracle(rows: &[Bits]) -> Vec<(u32, Exact)> {
    let mut totals = vec![Wide::ZERO; ACCOUNTS as usize];
    for &(account, bits) in rows {
        totals[account as usize].deposit(f64::from_bits(bits));
    }
    totals
        .iter()
        .enumerate()
        .map(|(account, w)| {
            let total = Exact::try_from(w).expect("every Wide converts under alloc");
            (account as u32, total)
        })
        .collect()
}

/// The same-scale fixture as bits, so one dataflow builder serves both fixtures.
fn as_bits(rows: &[Row]) -> Vec<Bits> {
    rows.iter()
        .map(|&(account, units)| (account, amount(units).to_bits()))
        .collect()
}

/// This worker's share of `rows`: every `peers`-th row from `index`.
fn share<'a, R>(worker: &Worker, rows: &'a [R]) -> impl Iterator<Item = &'a R> {
    let (index, peers) = (worker.index(), worker.peers());
    rows.iter().skip(index).step_by(peers)
}

/// The SUM with `Exact` as the diff, the plain idiom: `explode` the amount into the diff, then
/// `count_total` carries each account's exact total as data with multiplicity one.
fn exact_sum(worker: &mut Worker, rows: &[Bits]) -> Captured<(u32, Exact), isize> {
    let (mut input, captured) = worker.dataflow::<u64, _, _>(|scope| {
        let (input, rows) = scope.new_collection::<Bits, isize>();
        let totals = rows
            .explode(|(account, bits)| Some((account, Exact::from(f64::from_bits(bits)))))
            .count_total();
        (input, totals.inner.capture())
    });
    for row in share(worker, rows) {
        input.insert(*row);
    }
    input.close();
    captured
}

/// The same SUM through the banded idiom, which lands the same totals.
fn banded_sum(worker: &mut Worker, rows: &[Bits]) -> Captured<(u32, Exact), isize> {
    let (mut input, captured) = worker.dataflow::<u64, _, _>(|scope| {
        let (input, rows) = scope.new_collection::<Bits, isize>();
        (input, rows.banded_sum().inner.capture())
    });
    for row in share(worker, rows) {
        input.insert(*row);
    }
    input.close();
    captured
}

/// The same SUM with `Wide` as the diff, the `Copy` word: every exchanged update carries 296
/// bytes of weight, 312 in the tuple, against `Exact`'s 32 and 48.
fn wide_sum(worker: &mut Worker, rows: &[Bits]) -> Captured<(u32, Wide), isize> {
    let (mut input, captured) = worker.dataflow::<u64, _, _>(|scope| {
        let (input, rows) = scope.new_collection::<Bits, isize>();
        let totals = rows
            .explode(|(account, bits)| Some((account, Wide::from(f64::from_bits(bits)))))
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

/// Which idiom forms the `Exact` totals.
#[derive(Clone, Copy)]
enum Idiom {
    Plain,
    Banded,
}

impl Idiom {
    const ALL: [Self; 2] = [Self::Plain, Self::Banded];

    fn name(self) -> &'static str {
        match self {
            Self::Plain => "exact",
            Self::Banded => "banded",
        }
    }

    fn totals(self, worker: &mut Worker, rows: &[Bits]) -> Captured<(u32, Exact), isize> {
        match self {
            Self::Plain => exact_sum(worker, rows),
            Self::Banded => banded_sum(worker, rows),
        }
    }
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

/// One checked run of `idiom` over `rows`: the totals it lands are `oracle`'s, bit for bit.
fn check(
    idiom: Idiom,
    exchange: Exchange,
    workers: usize,
    rows: &Arc<Vec<Bits>>,
    oracle: &[(u32, Exact)],
    fixture: &str,
) {
    let shared = Arc::clone(rows);
    let out = run(workers, exchange, move |worker| {
        idiom.totals(worker, &shared)
    });
    let totals: Vec<(u32, Exact)> = out.into_iter().map(|(d, _, _)| d).collect();
    assert_eq!(totals.len(), ACCOUNTS as usize);
    assert_eq!(
        totals,
        oracle,
        "{} totals on {workers} workers, {} exchange, {fixture}",
        idiom.name(),
        exchange.name()
    );
}

/// Every dataflow on every worker count and exchange, after one checked run of each against its
/// oracle.
fn sum(c: &mut Criterion) {
    let rows = Arc::new(units(UPDATES));
    let same = Arc::new(as_bits(&rows));
    let mixed = Arc::new(mixed(UPDATES));
    let expected = units_oracle(&rows);
    let same_oracle: Vec<(u32, Exact)> = expected
        .iter()
        .map(|&(account, units)| (account, Exact::from(amount(units))))
        .collect();
    let isize_oracle: Vec<(u32, isize)> = expected
        .iter()
        .map(|&(account, units)| (account, units as isize))
        .collect();
    let mixed_oracle = wide_oracle(&mixed);
    assert!(
        mixed_oracle.iter().all(|(_, total)| !total.is_compact()),
        "the mixed fixture's totals all span past the width"
    );

    for exchange in Exchange::ALL {
        for workers in WORKERS {
            for idiom in Idiom::ALL {
                check(idiom, exchange, workers, &same, &same_oracle, "same-scale");
                check(idiom, exchange, workers, &mixed, &mixed_oracle, "mixed");
            }
            let shared = Arc::clone(&rows);
            let baseline = run(workers, exchange, move |worker| isize_sum(worker, &shared));
            let totals: Vec<(u32, isize)> = baseline.into_iter().map(|(d, _, _)| d).collect();
            assert_eq!(totals, isize_oracle, "isize totals on {workers} workers");
            let shared = Arc::clone(&same);
            let wide = run(workers, exchange, move |worker| wide_sum(worker, &shared));
            let totals: Vec<(u32, Exact)> = wide
                .into_iter()
                .map(|((account, total), _, _)| {
                    let total = Exact::try_from(&total).expect("a sum of doubles converts");
                    (account, total)
                })
                .collect();
            assert_eq!(
                totals,
                same_oracle,
                "Wide totals on {workers} workers, {} exchange",
                exchange.name()
            );
        }
    }

    let mut g = c.benchmark_group("dd_sum");
    g.throughput(Throughput::Elements(UPDATES));
    g.sampling_mode(SamplingMode::Flat);
    for exchange in Exchange::ALL {
        for workers in WORKERS {
            let name = exchange.name();
            for idiom in Idiom::ALL {
                let shared = Arc::clone(&same);
                g.bench_function(format!("{}/{workers}w/{name}", idiom.name()), |b| {
                    b.iter(|| {
                        let rows = Arc::clone(&shared);
                        black_box(run(workers, exchange, move |worker| {
                            idiom.totals(worker, &rows)
                        }))
                    });
                });
            }
            let shared = Arc::clone(&rows);
            g.bench_function(format!("isize/{workers}w/{name}"), |b| {
                b.iter(|| {
                    let rows = Arc::clone(&shared);
                    black_box(run(workers, exchange, move |worker| {
                        isize_sum(worker, &rows)
                    }))
                });
            });
            let shared = Arc::clone(&same);
            g.bench_function(format!("wide/{workers}w/{name}"), |b| {
                b.iter(|| {
                    let rows = Arc::clone(&shared);
                    black_box(run(workers, exchange, move |worker| {
                        wide_sum(worker, &rows)
                    }))
                });
            });
            for idiom in Idiom::ALL {
                let shared = Arc::clone(&mixed);
                g.bench_function(format!("{}/mixed/{workers}w/{name}", idiom.name()), |b| {
                    b.iter(|| {
                        let rows = Arc::clone(&shared);
                        black_box(run(workers, exchange, move |worker| {
                            idiom.totals(worker, &rows)
                        }))
                    });
                });
            }
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
