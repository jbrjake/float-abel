//! The prototype table re-taken on the crate, DESIGN.md § Measured, on a prototype and § Targets
//! and kill conditions: the compact deposit (`from_f64` + add), the compact add with both operands
//! built, the correctly rounded read, `Wide`'s add and the `f64 +=` floor, on the prototype's two
//! magnitude bands and on an adversarial one where every finite exponent is equally likely. Beside
//! the timings, the width-cap rate, counted rather than timed because the kill condition reads a
//! rate: how many pairwise adds and how many deposits of a running fold the 127-bit word refuses.
//! Advisory, never a gate: `cargo bench --bench ops`, and the numbers go into BENCH.md with the
//! machine and the commit.
//!
//! Every timed add is a running slot, each operation depending on the last, which is the shape an
//! engine's `plus_equals` into one aggregate has. A fold that caps restarts at the refused value,
//! so the slot carries an operand of realistic width on every band and the restarts are the cap
//! rate; a refused deposit leaves the word as it was, which is what the checked methods promise.
//! The read is independent per word, the shape of a column read out.

mod support;

use std::hint::black_box;

use criterion::{criterion_group, Criterion, Throughput};
use float_abel::{Exact, Wide};

use support::{adversarial, fixture, N};

/// `Wide` operands held in L2, not streamed from memory: 4096 words are 1.2 MB, where 2²⁰ would
/// be 310 MB and the add would measure the memory bus.
const WIDE_N: usize = 4096;

/// The three bands, by name.
fn bands() -> [(&'static str, Vec<f64>); 3] {
    [
        ("same-scale", fixture(-20, 20)),
        ("2^±300", fixture(-300, 300)),
        ("adversarial", adversarial()),
    ]
}

/// The same-scale factor pairs for the product deposit: full mantissas, because `Exact` has no
/// grid, so a product's lowest bit lies near 2⁻¹⁴⁴ against a sum's 2⁻⁷².
fn same_scale_pairs() -> (Vec<f64>, Vec<f64>) {
    let xs = fixture(-20, 20);
    let (lhs, rhs) = xs.split_at(N / 2);
    (lhs.to_vec(), rhs.to_vec())
}

/// Folds `xs` into a running word: the final word and how many deposits left it in the heap
/// form, which the compact fast path hands over past the width. Any cap is a bug in the fixture,
/// and says so.
fn fold(xs: &[f64]) -> (Exact, usize) {
    let mut acc = Exact::ZERO;
    let mut promoted = 0;
    for &x in xs {
        acc.checked_deposit(x)
            .unwrap_or_else(|cap| panic!("a finite double reached the {cap:?} cap"));
        promoted += usize::from(!acc.is_compact());
    }
    (acc, promoted)
}

/// Every running word the fold passes through, so an add or a read is timed on operands of the
/// width and form a real accumulator has on that band.
fn states(xs: &[f64]) -> Vec<Exact> {
    let mut acc = Exact::ZERO;
    xs.iter()
        .map(|&x| {
            acc.deposit(x);
            acc.clone()
        })
        .collect()
}

/// `part` of `whole` as a percentage; a count under 2²⁰ is exact in a double.
#[allow(clippy::cast_precision_loss)]
fn percent(part: usize, whole: usize) -> f64 {
    100.0 * part as f64 / whole as f64
}

/// The promotion rates, DESIGN.md § Targets and kill conditions: how often the compact fast path
/// hands a sum to the heap form. The kill condition reads the same-scale fold's rate, and the
/// other bands bound it. Printed as the rows BENCH.md carries.
fn rates() {
    println!("promotions on Exact, {N} values a band (counted, not timed)");
    println!("| band | pairwise adds promoted | running fold: deposits in the heap form | first promotion |");
    println!("|---|--:|--:|--:|");
    for (name, xs) in bands() {
        let pairs = xs.len() / 2;
        let pairwise = xs
            .chunks_exact(2)
            .filter(|p| !(Exact::from_f64(p[0]) + Exact::from_f64(p[1])).is_compact())
            .count();
        let (_, promoted) = fold(&xs);
        let first = {
            let mut acc = Exact::ZERO;
            xs.iter().position(|&x| {
                acc.deposit(x);
                !acc.is_compact()
            })
        };
        println!(
            "| {name} | {pairwise} of {pairs} ({:.2}%) | {promoted} of {} ({:.2}%) | {} |",
            percent(pairwise, pairs),
            xs.len(),
            percent(promoted, xs.len()),
            first.map_or_else(|| "never".to_owned(), |i| format!("deposit #{}", i + 1)),
        );
    }
    let (lhs, rhs) = same_scale_pairs();
    let mut acc = Exact::ZERO;
    let mut promoted = 0usize;
    let mut first = None;
    for (i, (&x, &y)) in lhs.iter().zip(&rhs).enumerate() {
        acc.deposit_product(x, y);
        if !acc.is_compact() {
            promoted += 1;
            first.get_or_insert(i + 1);
        }
    }
    println!(
        "| same-scale products | | {promoted} of {} ({:.2}%) | {} |",
        lhs.len(),
        percent(promoted, lhs.len()),
        first.map_or_else(|| "never".to_owned(), |i| format!("deposit #{i}")),
    );
}

/// The naive `f64 +=` into one slot: the floor every row is read against, and wrong in its last
/// bits.
fn floor(c: &mut Criterion) {
    for (name, xs) in bands() {
        let mut g = c.benchmark_group(format!("f64/{name}"));
        g.throughput(Throughput::Elements(xs.len() as u64));
        g.bench_function("+=", |b| {
            b.iter(|| {
                let mut acc = 0.0f64;
                for &x in &xs {
                    acc += x;
                }
                acc
            });
        });
        g.finish();
    }
}

/// The compact tier's rows: the deposit, the add with both operands built, and the read.
fn exact(c: &mut Criterion) {
    for (name, xs) in bands() {
        let built: Vec<Exact> = xs.iter().map(|&x| Exact::from_f64(x)).collect();
        let states = states(&xs);
        let mut g = c.benchmark_group(format!("exact/{name}"));
        g.throughput(Throughput::Elements(xs.len() as u64));
        g.bench_function("deposit (from_f64 + add)", |b| b.iter(|| fold(&xs)));
        g.bench_function("add, both operands built", |b| {
            b.iter(|| {
                let mut acc = Exact::ZERO;
                for e in &built {
                    acc += e;
                }
                acc
            });
        });
        g.bench_function("to_f64", |b| {
            b.iter(|| {
                states
                    .iter()
                    .fold(0u64, |h, s| h ^ black_box(s).to_f64().to_bits())
            });
        });
        g.finish();
    }

    // The engine's own call, `plus_equals` into a slot through `AddAssign<&Exact>`, which panics
    // past the cap and so runs only on the band that never caps.
    let xs = fixture(-20, 20);
    let built: Vec<Exact> = xs.iter().map(|&x| Exact::from_f64(x)).collect();
    let mut g = c.benchmark_group("exact/same-scale");
    g.throughput(Throughput::Elements(xs.len() as u64));
    g.bench_function("+=, both operands built", |b| {
        b.iter(|| {
            let mut acc = Exact::ZERO;
            for e in &built {
                acc += e;
            }
            acc
        });
    });
    g.finish();

    let (lhs, rhs) = same_scale_pairs();
    let mut g = c.benchmark_group("exact/same-scale products");
    g.throughput(Throughput::Elements(lhs.len() as u64));
    g.bench_function("deposit_product", |bch| {
        bch.iter(|| {
            let mut acc = Exact::ZERO;
            for (&x, &y) in lhs.iter().zip(&rhs) {
                if acc.checked_deposit_product(x, y).is_err() {
                    acc = Exact::product(x, y);
                }
            }
            acc
        });
    });
    g.finish();
}

/// `Wide + Wide` with both operands built, the design's "34-limb wide add": a running slot over
/// the first `WIDE_N` running sums of each band, which hold every double exactly.
fn wide(c: &mut Criterion) {
    for (name, xs) in bands() {
        let mut acc = Wide::ZERO;
        let operands: Vec<Wide> = xs[..WIDE_N]
            .iter()
            .map(|&x| {
                acc.deposit(x);
                acc
            })
            .collect();
        let mut g = c.benchmark_group(format!("wide/{name}"));
        g.throughput(Throughput::Elements(WIDE_N as u64));
        g.bench_function("add, both operands built", |b| {
            b.iter(|| {
                let mut acc = Wide::ZERO;
                for w in &operands {
                    acc += *w;
                }
                acc
            });
        });
        g.finish();
    }
}

criterion_group!(benches, floor, exact, wide);

/// `criterion_main!` with the rates printed first: they are a count, not a timing, and criterion
/// has no row for a count.
fn main() {
    rates();
    benches();
    Criterion::default().configure_from_args().final_summary();
}
