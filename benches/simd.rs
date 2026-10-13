//! The SIMD kill condition, DESIGN.md § Targets and kill conditions: each tier's bulk entry
//! against its one-at-a-time deposits on the same-scale fixture, so BENCH.md can say by how much a
//! kernel beats the scalar reference on this machine. Advisory, never a gate: `cargo bench --bench
//! simd`, and the numbers go into BENCH.md with the machine and the commit. The fixtures are the
//! design's: magnitudes within 2^±20 with random signs, and, for `Wide`, the 2^±300 band whose
//! tiles span more than one window.
//!
//! The `products` group is the product-kernel question, DESIGN.md § Speed: each tier's bulk
//! `deposit_products`, the scalar check-then-deposit as it stands, against the same tier's bulk
//! `deposit` kernel on one corpus, so the ratio says what a product kernel could gain.

mod support;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use float_abel::{Binned, Fixed256, Wide};

use support::{fixture, product_fixture, N};

fn wide(c: &mut Criterion) {
    for (name, xs) in [
        ("wide/same-scale", fixture(-20, 20)),
        ("wide/2^±300", fixture(-300, 300)),
    ] {
        let mut g = c.benchmark_group(name);
        g.throughput(Throughput::Elements(N as u64));
        g.bench_function("loop", |b| {
            b.iter(|| {
                let mut w = Wide::ZERO;
                for &x in &xs {
                    w.deposit(x);
                }
                w
            });
        });
        g.bench_function("extend_from_slice", |b| {
            b.iter(|| {
                let mut w = Wide::ZERO;
                w.extend_from_slice(&xs);
                w
            });
        });
        g.finish();
    }
}

fn fixed(c: &mut Criterion) {
    let same_scale = fixture(-20, 20);
    let mut g = c.benchmark_group("fixed256/same-scale");
    g.throughput(Throughput::Elements(N as u64));
    g.bench_function("loop", |b| {
        b.iter(|| {
            let mut f = Fixed256::ZERO;
            for &x in &same_scale {
                f.try_deposit(x).unwrap();
            }
            f
        });
    });
    g.bench_function("extend_from_slice", |b| {
        b.iter(|| {
            let mut f = Fixed256::ZERO;
            f.extend_from_slice(&same_scale).unwrap();
            f
        });
    });
    g.finish();
}

fn binned(c: &mut Criterion) {
    let same_scale = fixture(-20, 20);
    let mut g = c.benchmark_group("binned3/same-scale");
    g.throughput(Throughput::Elements(N as u64));
    g.bench_function("loop", |b| {
        b.iter(|| {
            let mut bins = Binned::<3>::ZERO;
            for &x in &same_scale {
                bins.deposit(x);
            }
            bins
        });
    });
    g.bench_function("extend_from_slice", |b| {
        b.iter(|| {
            let mut bins = Binned::<3>::ZERO;
            bins.extend_from_slice(&same_scale);
            bins
        });
    });
    g.finish();
}

/// The naive sum that is wrong in its last bits: the floor every exact tier is measured against,
/// with eight independent accumulators so the compiler vectorises it as a column engine would.
fn naive(c: &mut Criterion) {
    let same_scale = fixture(-20, 20);
    let mut g = c.benchmark_group("f64/same-scale");
    g.throughput(Throughput::Elements(N as u64));
    g.bench_function("vectorised naive sum", |b| {
        b.iter(|| {
            let mut acc = [0.0f64; 8];
            for chunk in same_scale.chunks_exact(8) {
                for (a, &x) in acc.iter_mut().zip(chunk) {
                    *a += x;
                }
            }
            acc.iter().sum::<f64>()
        });
    });
    g.finish();
}

/// The product-kernel row, per tier: the bulk `deposit_products` as it stands (every pair checked
/// against the window, then deposited one at a time), the one-at-a-time `deposit_product` loop
/// that shows the check pass's share, and the same tier's bulk `deposit` kernel on the `a` slice
/// of the same corpus. The ratio of the first to the last bounds what a product kernel could
/// gain; a kernel's lane does the multiply where the sum's lane does a shift.
fn products(c: &mut Criterion) {
    let (lhs, rhs) = product_fixture();

    let mut g = c.benchmark_group("products/wide");
    g.throughput(Throughput::Elements(N as u64));
    g.bench_function("deposit_products", |bch| {
        bch.iter(|| {
            let mut w = Wide::ZERO;
            w.deposit_products(&lhs, &rhs).unwrap();
            w
        });
    });
    g.bench_function("deposit_product loop", |bch| {
        bch.iter(|| {
            let mut w = Wide::ZERO;
            for (&x, &y) in lhs.iter().zip(&rhs) {
                w.deposit_product(x, y).unwrap();
            }
            w
        });
    });
    g.bench_function("extend_from_slice", |bch| {
        bch.iter(|| {
            let mut w = Wide::ZERO;
            w.extend_from_slice(&lhs);
            w
        });
    });
    g.finish();

    let mut g = c.benchmark_group("products/fixed256");
    g.throughput(Throughput::Elements(N as u64));
    g.bench_function("deposit_products", |bch| {
        bch.iter(|| {
            let mut f = Fixed256::ZERO;
            f.deposit_products(&lhs, &rhs).unwrap();
            f
        });
    });
    g.bench_function("deposit_product loop", |bch| {
        bch.iter(|| {
            let mut f = Fixed256::ZERO;
            for (&x, &y) in lhs.iter().zip(&rhs) {
                f.deposit_product(x, y).unwrap();
            }
            f
        });
    });
    g.bench_function("extend_from_slice", |bch| {
        bch.iter(|| {
            let mut f = Fixed256::ZERO;
            f.extend_from_slice(&lhs).unwrap();
            f
        });
    });
    g.finish();

    let mut g = c.benchmark_group("products/binned3");
    g.throughput(Throughput::Elements(N as u64));
    g.bench_function("deposit_products", |bch| {
        bch.iter(|| {
            let mut bins = Binned::<3>::ZERO;
            bins.deposit_products(&lhs, &rhs).unwrap();
            bins
        });
    });
    g.bench_function("deposit_product loop", |bch| {
        bch.iter(|| {
            let mut bins = Binned::<3>::ZERO;
            for (&x, &y) in lhs.iter().zip(&rhs) {
                bins.deposit_product(x, y).unwrap();
            }
            bins
        });
    });
    g.bench_function("extend_from_slice", |bch| {
        bch.iter(|| {
            let mut bins = Binned::<3>::ZERO;
            bins.extend_from_slice(&lhs);
            bins
        });
    });
    g.finish();
}

criterion_group!(benches, wide, fixed, binned, naive, products);
criterion_main!(benches);
