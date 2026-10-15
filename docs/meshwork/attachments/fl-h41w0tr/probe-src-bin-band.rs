//! A candidate idiom: the key carries the double's band (its leading bit's exponent floored to a
//! multiple of 32), so every `Exact` diff differential forms is a sum of values inside one band,
//! whose span is at most 32 + 52 + log2(n) bits, inside the width for n < 2^43 rows. The band
//! totals come out of `count_total` as data with `isize` diffs, and a `reduce` folds a key's few
//! band totals into a `Wide` and emits the rounded double. No `Exact` diff then depends on what
//! else is in the key, so there is nothing to check at the door.
//!
//! MODE=histories runs the five histories of dd.rs on both pairs through the banded idiom.
//! MODE=random runs a seeded multiset over 2^±300, with a few specials on one key, inserts and
//! retractions over ten epochs, and checks every epoch's read per key against a `Wide` oracle
//! bit for bit. MODE=plain runs the same multiset through the plain `explode` + `count_total`
//! idiom, the negative control, which is expected to panic. `-w N` picks the workers.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use differential_dataflow::input::Input;
use differential_dataflow::operators::CountTotal;
use float_abel::{Exact, Wide};

/// The band of a double: the exponent of its leading bit, offset by 1074 so that subnormals
/// start at zero, floored to a multiple of 32. Finite values land in 0..=65; NaN and the
/// infinities take 255, where `Exact` counts them. The zero is the zero in any band.
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

type Epochs = Vec<Vec<(u32, f64, isize)>>;
/// Per (key, epoch): the output updates (bits of the read, diff).
type Out = Arc<Mutex<BTreeMap<(u32, u64), Vec<(u64, isize)>>>>;

fn run(epochs: Epochs, banded: bool) -> Out {
    let out: Out = Arc::default();
    let sink = out.clone();
    timely::execute_from_args(std::env::args(), move |worker| {
        let sink = sink.clone();
        let (mut rows, probe) = worker.dataflow::<u64, _, _>(|scope| {
            let (input, rows) = scope.new_collection::<(u32, u64), isize>();
            let reads = if banded {
                rows.explode(|(key, bits)| {
                    let x = f64::from_bits(bits);
                    Some((((key, band(x)), ()), Exact::from(x)))
                })
                // ((key, band), ()) -> ((key, band), band_total) with isize diffs
                .count_total()
                .map(|(((key, band), ()), total)| (key, (band, total)))
                .reduce(|_key, bands: &[(&(u8, Exact), isize)], output: &mut Vec<(u64, isize)>| {
                    let mut sum = Wide::default();
                    for ((_band, total), mult) in bands {
                        sum += Wide::try_from(total).expect("a band total is a sum of doubles")
                            * (*mult as i64);
                    }
                    output.push((sum.to_f64().to_bits(), 1));
                })
            } else {
                rows.explode(|(key, bits)| Some(((key, ()), Exact::from(f64::from_bits(bits)))))
                    .count_total()
                    .map(|((key, ()), total)| (key, total.to_f64().to_bits()))
            };
            let (probe, _) = reads
                .inspect(move |((key, bits), epoch, diff)| {
                    sink.lock()
                        .unwrap()
                        .entry((*key, *epoch))
                        .or_default()
                        .push((*bits, *diff));
                })
                .probe();
            (input, probe)
        });
        for (i, epoch) in epochs.iter().enumerate() {
            if worker.index() == 0 {
                for &(key, x, m) in epoch {
                    rows.update((key, x.to_bits()), m);
                }
            }
            rows.advance_to(i as u64 + 1);
            rows.flush();
            worker.step_while(|| probe.less_than(rows.time()));
        }
    })
    .expect("the workers start");
    out
}

/// The live read per key after each epoch: the output updates folded per bits in epoch order,
/// the one with multiplicity +1 at that point. A key with no live row reads +0.0, because a key
/// whose every band total is the zero has no input to the reduce.
fn reads_by_epoch(out: &Out, keys: u32, epochs: usize) -> BTreeMap<(u32, usize), f64> {
    let out = out.lock().unwrap();
    let mut result = BTreeMap::new();
    for key in 0..keys {
        let mut net: BTreeMap<u64, isize> = BTreeMap::new();
        for epoch in 0..epochs {
            if let Some(updates) = out.get(&(key, epoch as u64)) {
                for (bits, diff) in updates {
                    *net.entry(*bits).or_default() += diff;
                }
            }
            let live: Vec<u64> = net
                .iter()
                .filter(|(_, m)| **m != 0)
                .map(|(bits, m)| {
                    assert_eq!(*m, 1, "key {key} epoch {epoch}: multiplicity {m}");
                    *bits
                })
                .collect();
            assert!(live.len() <= 1, "key {key} epoch {epoch}: {} live reads", live.len());
            result.insert((key, epoch), live.first().map_or(0.0, |b| f64::from_bits(*b)));
        }
    }
    result
}

fn histories() {
    for (pair, big, small) in [
        ("everyday", 1e12, 1e-12),
        ("huge", 2f64.powi(1000), 2f64.powi(-1000)),
    ] {
        for (name, epochs) in [
            ("A", vec![vec![(0, big, 1)], vec![(0, big, -1), (0, small, 1)]]),
            ("B", vec![vec![(0, big, 1)], vec![(0, small, 1)], vec![(0, big, -1)]]),
            ("C", vec![vec![(0, big, 1), (0, small, 1), (0, big, -1)]]),
            ("D", vec![vec![(0, big, 1)], vec![(0, big, -1)], vec![(0, small, 1)]]),
            ("E", vec![vec![(0, big, 1), (0, big, -1), (0, small, 1)]]),
        ] {
            let n = epochs.len();
            let out = run(epochs, true);
            let reads = reads_by_epoch(&out, 1, n);
            let got = reads[&(0, n - 1)];
            println!(
                "{pair:<9} {name}: completes, reads {got:e} ({})",
                if got.to_bits() == small.to_bits() { "the small value exactly" } else { "WRONG" }
            );
            assert_eq!(got.to_bits(), small.to_bits());
        }
    }
}

/// A seeded xorshift, so the multiset is the same on every worker count.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

/// Ten epochs over five keys: each epoch inserts 400 rows spread over 2^±300 with full
/// mantissas, then retracts a third of what is live. One special in about 300 rows, on key 4
/// alone, so the other keys stay finite. Returns the epochs and the `Wide` oracle per key after
/// each epoch.
fn multiset() -> (Epochs, BTreeMap<(u32, usize), f64>) {
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    let keys = 5u32;
    let mut live: Vec<(u32, f64)> = Vec::new();
    let mut epochs: Epochs = Vec::new();
    let mut oracle = BTreeMap::new();
    for epoch_index in 0..10 {
        let mut epoch = Vec::new();
        for _ in 0..400 {
            let key = rng.below(keys as u64) as u32;
            let x = if key == 4 && rng.below(300) == 0 {
                [f64::NAN, f64::INFINITY, f64::NEG_INFINITY][rng.below(3) as usize]
            } else {
                let mant = 1.0 + (rng.next() >> 11) as f64 / (1u64 << 53) as f64;
                let exp = rng.below(601) as i32 - 300;
                let sign = if rng.below(2) == 0 { 1.0 } else { -1.0 };
                sign * mant * 2f64.powi(exp)
            };
            epoch.push((key, x, 1));
            live.push((key, x));
        }
        let mut i = 0;
        while i < live.len() {
            if rng.below(3) == 0 {
                let (key, x) = live.swap_remove(i);
                epoch.push((key, x, -1));
            } else {
                i += 1;
            }
        }
        epochs.push(epoch);
        let mut per_key: BTreeMap<u32, Wide> = (0..keys).map(|k| (k, Wide::default())).collect();
        for (key, x) in &live {
            *per_key.get_mut(key).unwrap() += Wide::from(*x);
        }
        for (key, w) in per_key {
            oracle.insert((key, epoch_index), w.to_f64());
        }
    }
    (epochs, oracle)
}

fn random(workers: &str, banded: bool) {
    let (epochs, oracle) = multiset();
    let total_updates: usize = epochs.iter().map(Vec::len).sum();
    let bands: std::collections::BTreeSet<u8> = epochs
        .iter()
        .flatten()
        .map(|(_, x, _)| band(*x))
        .collect();
    println!(
        "{workers} worker(s), {}: {total_updates} updates over 10 epochs across {} bands",
        if banded { "banded" } else { "plain" },
        bands.len()
    );
    let n = epochs.len();
    let out = run(epochs, banded);
    let reads = reads_by_epoch(&out, 5, n);
    let mut checked = 0;
    for ((key, epoch), want) in &oracle {
        let got = reads[&(*key, *epoch)];
        assert_eq!(
            got.to_bits(),
            want.to_bits(),
            "key {key} epoch {epoch}: dataflow {got:e}, oracle {want:e}"
        );
        checked += 1;
    }
    for key in 0..5 {
        println!(
            "  key {key}: last epoch reads {:e}, the Wide oracle's bits",
            reads[&(key, n - 1)]
        );
    }
    println!("  {checked} (key, epoch) reads equal the Wide oracle bit for bit");
}

fn main() {
    let mode = std::env::var("MODE").unwrap_or_else(|_| "histories".into());
    let workers = std::env::args()
        .collect::<Vec<_>>()
        .windows(2)
        .find(|w| w[0] == "-w")
        .map(|w| w[1].clone())
        .unwrap_or_else(|| "1".into());
    match mode.as_str() {
        "histories" => histories(),
        "random" => random(&workers, true),
        "plain" => random(&workers, false),
        other => panic!("unknown MODE {other}"),
    }
    println!("ok");
}
