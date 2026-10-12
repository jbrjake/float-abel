//! The SIMD kill condition, DESIGN.md § Targets and kill conditions: each tier's bulk entry
//! against its one-at-a-time deposits on the same-scale fixture, so BENCH.md can say by how much a
//! kernel beats the scalar reference on this machine. Advisory, never a gate: `cargo bench --bench
//! simd`, and the numbers go into BENCH.md with the machine and the commit. The fixtures are the
//! design's: magnitudes within 2^±20 with random signs, and, for `Wide`, the 2^±300 band whose
//! tiles span more than one window.

// The fixture builds doubles from their bits and moves bits between integer widths on purpose.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use float_abel::{Binned, Fixed256, Wide};

const N: usize = 1 << 20;

/// splitmix64, the suite's stream.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A normal double with its leading bit at 2^e for e in [lo, hi), a full random fraction, and
    /// either sign.
    fn on_grid(&mut self, lo: i32, hi: i32) -> f64 {
        let e = lo + (self.next() % (hi - lo) as u64) as i32;
        let frac = self.next() >> 12;
        let x = f64::from_bits(((e + 1023) as u64) << 52 | frac);
        if self.next() & 1 == 0 {
            x
        } else {
            -x
        }
    }
}

fn fixture(lo: i32, hi: i32) -> Vec<f64> {
    let mut rng = Rng(0xBE4C_4D41_524B);
    (0..N).map(|_| rng.on_grid(lo, hi)).collect()
}

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

criterion_group!(benches, wide, fixed, binned, naive);
criterion_main!(benches);
