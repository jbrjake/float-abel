//! The layout probe for fl-nhcdk92: the cost probe of fl-t154dsp (`isize`, plain `Exact`, the
//! banded idiom, the 40-byte enum proxy `H`, `Wide`) plus the ruled 32-byte niche form of the
//! heap arm as a newtype in two block shapes, on three bands, one worker, typed exchange
//! (`execute_directly`, timely's Thread allocator) and serialized exchange
//! (`CommunicationConfig::ProcessBinary(1)`, every exchanged diff crossing as its serde bytes).
//! Ten runs each, the fastest kept. Not criterion: a budget for fl-th7g69s.
//!
//! `N<INLINE, DROP>` is the niche form. Its 32 bytes are `repr(C)` `{lo, hi, exp, nan, pos, neg}`, the
//! compact `Exact` word bit for bit when `lo` is odd or zero (a canonical compact mantissa is odd
//! or the zero form, so an even nonzero `lo` is never a compact value). When `lo` is even and
//! nonzero the word is the heap form: `lo` carries the limb count and `hi` the pointer to one
//! allocation. Shape A (`INLINE = true`): the block is the magnitude's limbs alone, and the sign,
//! the exponent and the three counts stay in the word (`lo` bit 1 the sign, `exp` and the counts
//! in their own fields). Shape B (`INLINE = false`): the block opens with a three-limb header
//! holding the sign, the exponent and the counts, then the limbs, and the word's `exp` and count
//! fields are zero. The heap magnitude is sign-magnitude, normalised: odd, no leading zero limb,
//! so one value has one form and a value fits the compact word iff its bit length is at most 127.
//! `plus_equals` goes through `Exact::checked_add` first; a refusal promotes both sides to a
//! stack-buffered working form, adds there, and lands compact or allocates. Every heap result is
//! a fresh block and the old one is freed (no capacity reuse), so the heap path's cost here is an
//! upper bound on the real arm's.

mod big;
mod niche;
mod proxies;

use std::time::{Duration, Instant};

use differential_dataflow::input::Input;
use differential_dataflow::operators::CountTotal;
use float_abel::{Exact, Wide};
use timely::{CommunicationConfig, Config, WorkerConfig};

use niche::N;
use proxies::{C, H, W};

// ------------------------------------------------------------------------------- the fixture

fn band(x: f64) -> u8 {
    if !x.is_finite() {
        return 255;
    }
    let bits = x.to_bits();
    let exp = ((bits >> 52) & 0x7ff) as i32;
    let frac = bits & ((1u64 << 52) - 1);
    if exp == 0 && frac == 0 {
        return 0;
    }
    let lead = if exp == 0 {
        -1074 + (63 - frac.leading_zeros() as i32)
    } else {
        exp - 1023
    };
    ((lead + 1074) >> 5) as u8
}

/// Rows as (key, amount bits); `units` is the same amount as an integer count for `isize` on the
/// same-scale band. `spread` 0 is same-scale; otherwise magnitudes are uniform over 2^±spread
/// with full mantissas and either sign.
fn fixture(n: u64, spread: i32) -> (Vec<(u32, u64)>, Vec<(u32, i64)>) {
    let mut rows = Vec::with_capacity(n as usize);
    let mut units = Vec::with_capacity(n as usize);
    let mut s = 0x9E37_79B9_7F4A_7C15u64;
    for i in 0..n {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        let key = ((i * 7919) % 16) as u32;
        let u = ((i.wrapping_mul(2_654_435_761)) % (1 << 21)) as i64 - (1 << 20);
        let x = if spread > 0 {
            let mant = 1.0 + (s >> 11) as f64 / (1u64 << 53) as f64;
            let exp = (s % (2 * spread as u64 + 1)) as i32 - spread;
            mant * 2f64.powi(exp) * if s & 1 == 0 { 1.0 } else { -1.0 }
        } else {
            u as f64 * 2f64.powi(-20)
        };
        rows.push((key, x.to_bits()));
        units.push((key, u));
    }
    (rows, units)
}

// ------------------------------------------------------------------------------- the harness

type Rows<'a> = differential_dataflow::VecCollection<'a, u64, (u32, u64), isize>;
type Probe = timely::dataflow::ProbeHandle<u64>;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Exchange {
    Typed,
    Serialized,
}

/// One worker, the rows fed at epoch 0, timed from the first insert to the probe: on typed
/// exchange through `execute_directly` (the Thread allocator), on serialized through
/// `execute` with `ProcessBinary(1)`, joined.
fn timed<F>(rows: &[(u32, u64)], exchange: Exchange, build: F) -> Duration
where
    F: Fn(Rows<'_>) -> Probe + Send + Sync + 'static,
{
    let rows = rows.to_vec();
    let body = move |worker: &mut timely::worker::Worker| {
        let (mut input, probe) = worker.dataflow::<u64, _, _>(|scope| {
            let (input, rows) = scope.new_collection::<(u32, u64), isize>();
            (input, build(rows))
        });
        let start = Instant::now();
        for row in &rows {
            input.insert(*row);
        }
        input.advance_to(1);
        input.flush();
        worker.step_while(|| probe.less_than(input.time()));
        start.elapsed()
    };
    match exchange {
        Exchange::Typed => timely::execute_directly(body),
        Exchange::Serialized => {
            let config = Config {
                communication: CommunicationConfig::ProcessBinary(1),
                worker: WorkerConfig::default(),
            };
            let guards = timely::execute(config, body).expect("the worker starts");
            let mut out = guards.join();
            out.pop().unwrap().expect("the worker finishes")
        }
    }
}

fn best<F: Fn() -> Duration>(runs: usize, f: F) -> Duration {
    let mut ts: Vec<Duration> = (0..runs).map(|_| f()).collect();
    ts.sort();
    let n = ts.len();
    eprintln!("    runs: min {:.1} median {:.1} max {:.1} ms", ts[0].as_secs_f64() * 1e3, ts[n / 2].as_secs_f64() * 1e3, ts[n - 1].as_secs_f64() * 1e3);
    ts[0]
}

struct Cell {
    label: &'static str,
    band: usize,
    exchange: Exchange,
    ns: f64,
}

fn main() {
    let n: u64 = std::env::var("N").ok().and_then(|s| s.parse().ok()).unwrap_or(1_000_000);
    let runs: usize = std::env::var("RUNS").ok().and_then(|s| s.parse().ok()).unwrap_or(10);
    let bands = [("same-scale", 0), ("spread 2^±60", 60), ("mixed 2^±300", 300)];
    let mut cells: Vec<Cell> = Vec::new();
    let mut push = |label: &'static str, band: usize, exchange: Exchange, t: Duration| {
        let ns = t.as_nanos() as f64 / n as f64;
        let x = if exchange == Exchange::Typed { "typed" } else { "serialized" };
        println!("  {label:<16} {x:<11} {:>8.1} ms  {ns:>6.1} ns/update", t.as_secs_f64() * 1e3);
        cells.push(Cell { label, band, exchange, ns });
    };
    for (bi, (label, spread)) in bands.iter().enumerate() {
        let (rows, units) = fixture(n, *spread);
        println!("-- {n} updates, 16 keys, one epoch, one worker, {label}, best of {runs} --");
        for exchange in [Exchange::Typed, Exchange::Serialized] {
            if *spread == 0 {
                let t = best(runs, || {
                    let units = units.clone();
                    let body = move |worker: &mut timely::worker::Worker| {
                        let (mut input, probe) = worker.dataflow::<u64, _, _>(|scope| {
                            let (input, rows) = scope.new_collection::<(u32, i64), isize>();
                            let probe = rows
                                .explode(|(key, u)| Some(((key, ()), u as isize)))
                                .count_total()
                                .probe()
                                .0;
                            (input, probe)
                        });
                        let start = Instant::now();
                        for row in &units {
                            input.insert(*row);
                        }
                        input.advance_to(1);
                        input.flush();
                        worker.step_while(|| probe.less_than(input.time()));
                        start.elapsed()
                    };
                    match exchange {
                        Exchange::Typed => timely::execute_directly(body),
                        Exchange::Serialized => {
                            let config = Config {
                                communication: CommunicationConfig::ProcessBinary(1),
                                worker: WorkerConfig::default(),
                            };
                            let guards = timely::execute(config, body).expect("starts");
                            guards.join().pop().unwrap().expect("finishes")
                        }
                    }
                });
                push("isize", bi, exchange, t);
                let t = best(runs, || {
                    timed(&rows, exchange, |rows| {
                        rows.explode(|(key, bits)| {
                            Some(((key, ()), Exact::from(f64::from_bits(bits))))
                        })
                        .count_total()
                        .probe()
                        .0
                    })
                });
                push("Exact, plain", bi, exchange, t);
            }
            let t = best(runs, || {
                timed(&rows, exchange, |rows| {
                    rows.explode(|(key, bits)| {
                        let x = f64::from_bits(bits);
                        Some((((key, band(x)), ()), Exact::from(x)))
                    })
                    .count_total()
                    .map(|(((key, band), ()), total)| (key, (band, total)))
                    .reduce(
                        |_key, bands: &[(&(u8, Exact), isize)], output: &mut Vec<(u64, isize)>| {
                            let mut sum = Wide::default();
                            for ((_band, total), mult) in bands {
                                sum += Wide::try_from(total).expect("inside Wide") * (*mult as i64);
                            }
                            output.push((sum.to_f64().to_bits(), 1));
                        },
                    )
                    .probe()
                    .0
                })
            });
            push("Exact, banded", bi, exchange, t);
            let t = best(runs, || {
                timed(&rows, exchange, |rows| {
                    rows.explode(|(key, bits)| {
                        Some(((key, ()), H::C(Exact::from(f64::from_bits(bits)))))
                    })
                    .count_total()
                    .probe()
                    .0
                })
            });
            push("enum 40 B (H)", bi, exchange, t);
            let t = best(runs, || {
                timed(&rows, exchange, |rows| {
                    rows.explode(|(key, bits)| {
                        Some(((key, ()), N::<true, true>::from_exact(Exact::from(f64::from_bits(bits)))))
                    })
                    .count_total()
                    .probe()
                    .0
                })
            });
            push("niche A inline", bi, exchange, t);
            let t = best(runs, || {
                timed(&rows, exchange, |rows| {
                    rows.explode(|(key, bits)| {
                        Some(((key, ()), N::<false, true>::from_exact(Exact::from(f64::from_bits(bits)))))
                    })
                    .count_total()
                    .probe()
                    .0
                })
            });
            push("niche B header", bi, exchange, t);
            if *spread == 0 {
                // The diagnostic: the same word with an empty Drop. It leaks on promotion, so it
                // runs on the band that never promotes.
                let t = best(runs, || {
                    timed(&rows, exchange, |rows| {
                        rows.explode(|(key, bits)| {
                            Some(((key, ()), N::<true, false>::from_exact(Exact::from(f64::from_bits(bits)))))
                        })
                        .count_total()
                        .probe()
                        .0
                    })
                });
                push("niche A, no Drop", bi, exchange, t);
                let t = best(runs, || {
                    timed(&rows, exchange, |rows| {
                        rows.explode(|(key, bits)| Some(((key, ()), C(Exact::from(f64::from_bits(bits))))))
                            .count_total()
                            .probe()
                            .0
                    })
                });
                push("Copy + tag test", bi, exchange, t);
            }
            let t = best(runs, || {
                timed(&rows, exchange, |rows| {
                    rows.explode(|(key, bits)| Some(((key, ()), W(Wide::from(f64::from_bits(bits))))))
                        .count_total()
                        .probe()
                        .0
                })
            });
            push("Wide, plain", bi, exchange, t);
        }
    }
    println!();
    println!("| diff | {} |", bands.iter().map(|(l, _)| format!("{l} typed | {l} serialized")).collect::<Vec<_>>().join(" | "));
    println!("|---|{}|", "--:|".repeat(bands.len() * 2));
    for label in ["isize", "Exact, plain", "Exact, banded", "enum 40 B (H)", "niche A inline", "niche B header", "niche A, no Drop", "Copy + tag test", "Wide, plain"] {
        let mut line = format!("| {label} |");
        for bi in 0..bands.len() {
            for exchange in [Exchange::Typed, Exchange::Serialized] {
                let cell = cells.iter().find(|c| c.label == label && c.band == bi && c.exchange == exchange);
                match cell {
                    Some(c) => line.push_str(&format!(" {:.1} |", c.ns)),
                    None => line.push_str(" |"),
                }
            }
        }
        println!("{line}");
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;
    use std::mem;

    use differential_dataflow::difference::{Abelian, IsZero, Multiply, Semigroup};

    use super::*;

    fn n<const I: bool>(x: f64) -> N<I, true> {
        N::from_exact(Exact::from(x))
    }

    fn check<const I: bool>() {
        // 1e300 + 1e-300 promotes; subtracting 1e-300 demotes to Exact::from(1e300) structurally.
        let mut a = n::<I>(1e300);
        a.plus_equals(&n::<I>(1e-300));
        assert!(a.is_heap());
        let mut c = a.clone();
        assert_eq!(a, c);
        c.plus_equals(&n::<I>(-1e-300));
        assert!(!c.is_heap());
        assert_eq!(c, n::<I>(1e300));
        // 0.1 + 0.2 stays compact and equals the Exact sum.
        let mut d = n::<I>(0.1);
        d.plus_equals(&n::<I>(0.2));
        assert!(!d.is_heap());
        assert_eq!(d.as_exact(), &(Exact::from(0.1) + Exact::from(0.2)));
        // The five histories: each reads the small value exactly.
        for (big, small) in [(1e12, 1e-12), (2f64.powi(1000), 2f64.powi(-1000))] {
            let mut s = n::<I>(big);
            s.plus_equals(&n::<I>(small));
            s.plus_equals(&n::<I>(-big));
            assert_eq!(s, n::<I>(small));
            assert_eq!(s.to_f64(), small);
            let mut s = n::<I>(-big);
            s.plus_equals(&n::<I>(small));
            s.plus_equals(&n::<I>(big));
            assert_eq!(s, n::<I>(small));
        }
        // A 2^±300 fold with retractions equals a Wide oracle at every step and ends at zero.
        let (rows, _) = fixture(2000, 300);
        let mut acc = N::<I, true>::ZERO;
        let mut w = Wide::ZERO;
        for &(_, bits) in &rows {
            let x = f64::from_bits(bits);
            acc.plus_equals(&n::<I>(x));
            w.deposit(x);
            assert_eq!(acc.to_f64(), w.to_f64());
        }
        let heap_seen = acc.is_heap();
        assert!(heap_seen, "the mixed band promotes");
        // Serde round trip of a heap value.
        let bytes = bincode::serialize(&acc).unwrap();
        let back: N<I, true> = bincode::deserialize(&bytes).unwrap();
        assert_eq!(back, acc);
        let bytes = bincode::serialize(&n::<I>(0.1)).unwrap();
        assert_eq!(bytes.len(), 8 + 33);
        for &(_, bits) in rows.iter().rev() {
            acc.plus_equals(&n::<I>(-f64::from_bits(bits)));
        }
        assert!(acc.is_zero());
        assert_eq!(acc, N::<I, true>::ZERO);
        // Multiply by -3 on a heap value agrees with three negated adds.
        let mut h = n::<I>(1e300);
        h.plus_equals(&n::<I>(1e-300));
        let m = h.clone().multiply(&-3isize);
        let mut t = N::<I, true>::ZERO;
        for _ in 0..3 {
            let mut neg = h.clone();
            neg.negate();
            t.plus_equals(&neg);
        }
        assert_eq!(m, t);
        // Ord across forms.
        assert!(n::<I>(1e300) < h);
        assert!(h > n::<I>(-1e300));
        assert_eq!(h.cmp(&h.clone()), Ordering::Equal);
    }

    #[test]
    fn inline() {
        check::<true>();
    }

    /// A differential update `(data, time, diff)` with a `u64` key: the niche form keeps the
    /// tuple at the Copy word's 48 bytes where the enum proxy makes it 56.
    #[test]
    fn update_tuple_sizes() {
        assert_eq!(mem::size_of::<((u64, ()), u64, Exact)>(), 48);
        assert_eq!(mem::size_of::<((u64, ()), u64, N<true, true>)>(), 48);
        assert_eq!(mem::size_of::<((u64, ()), u64, N<false, true>)>(), 48);
        assert_eq!(mem::size_of::<((u64, ()), u64, H)>(), 56);
        assert_eq!(mem::size_of::<((u64, ()), u64, W)>(), 312);
        assert_eq!(mem::size_of::<H>(), 40);
        assert_eq!(mem::align_of::<N<true, true>>(), 8);
    }

    #[test]
    fn header() {
        check::<false>();
    }
}
