//! A rough cost comparison of the candidate diffs, one worker, one epoch of N updates over 16
//! keys through `explode` + `count_total` (plus the banded idiom's `reduce`), timed from the
//! first insert to the probe passing. Three bands: same-scale amounts (units of 2^-20 below
//! 2^20), a spread of 2^±60, and the adversary 2^±300. `isize` and plain `Exact` run only where
//! they can. Three runs each, the fastest reported. Not criterion: a rough ratio for a ruling.
//!
//! `H` is a proxy for the design's heap arm: `Exact` inline, promoted to a `Box<Wide>` when the
//! compact add refuses, demoted back after every heap op so canonical form is the smallest tier
//! that fits (the original DESIGN.md § Exact rule). The real heap tier held a normalised
//! `Vec<u64>` mantissa (two allocations a promotion, a shorter limb add); the proxy allocates
//! once and adds 34 limbs. It is the cost shape, not the implementation.

use std::time::{Duration, Instant};

use differential_dataflow::difference::{Abelian, IsZero, Monoid, Multiply, Semigroup};
use differential_dataflow::input::Input;
use differential_dataflow::operators::CountTotal;
use float_abel::{Exact, Wide};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[serde(transparent)]
struct W(Wide);
impl IsZero for W {
    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}
impl Semigroup for W {
    fn plus_equals(&mut self, rhs: &Self) {
        self.0 += rhs.0;
    }
}
impl Monoid for W {
    fn zero() -> Self {
        Self(Wide::default())
    }
}
impl Abelian for W {
    fn negate(&mut self) {
        self.0 = -self.0;
    }
}
impl Multiply<isize> for W {
    type Output = Self;
    fn multiply(self, rhs: &isize) -> Self {
        Self(self.0 * *rhs)
    }
}

/// The heap-arm proxy.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
enum H {
    C(Exact),
    W(Box<Wide>),
}
fn widen(e: &Exact) -> Wide {
    Wide::try_from(e).expect("a sum of doubles is inside Wide")
}
fn demote(w: Wide) -> H {
    match Exact::try_from(&w) {
        Ok(e) => H::C(e),
        Err(_) => H::W(Box::new(w)),
    }
}
impl IsZero for H {
    fn is_zero(&self) -> bool {
        match self {
            H::C(e) => e.is_zero(),
            H::W(w) => w.is_zero(),
        }
    }
}
impl Semigroup for H {
    fn plus_equals(&mut self, rhs: &Self) {
        let next = match (&mut *self, rhs) {
            (H::C(a), H::C(b)) => match a.checked_add(b) {
                Ok(s) => {
                    *a = s;
                    return;
                }
                Err(_) => demote(widen(a) + widen(b)),
            },
            (H::W(a), H::C(b)) => demote(**a + widen(b)),
            (H::C(a), H::W(b)) => demote(widen(a) + **b),
            (H::W(a), H::W(b)) => demote(**a + **b),
        };
        *self = next;
    }
}
impl Monoid for H {
    fn zero() -> Self {
        H::C(Exact::ZERO)
    }
}
impl Abelian for H {
    fn negate(&mut self) {
        match self {
            H::C(e) => *e = -*e,
            H::W(w) => **w = -**w,
        }
    }
}
impl Multiply<isize> for H {
    type Output = Self;
    fn multiply(self, rhs: &isize) -> Self {
        match self {
            H::C(e) => match e.checked_mul_i64(*rhs as i64) {
                Ok(s) => H::C(s),
                Err(_) => demote(widen(&e) * *rhs),
            },
            H::W(w) => demote(*w * *rhs),
        }
    }
}

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

/// Runs `build` on one worker, feeds the rows at epoch 0, and times to the probe.
fn timed<F>(rows: Vec<(u32, u64)>, build: F) -> Duration
where
    F: Fn(differential_dataflow::VecCollection<'_, u64, (u32, u64), isize>) -> timely::dataflow::ProbeHandle<u64>
        + Send
        + Sync
        + 'static,
{
    timely::execute_directly(move |worker| {
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
    })
}

fn best<F: Fn() -> Duration>(f: F) -> Duration {
    (0..3).map(|_| f()).min().unwrap()
}

fn row(label: &str, t: Duration, n: u64) {
    println!("  {label:<16} {:>8.1} ms  {:>6.1} ns/update", t.as_secs_f64() * 1e3, t.as_nanos() as f64 / n as f64);
}

fn main() {
    let n: u64 = std::env::var("N").ok().and_then(|s| s.parse().ok()).unwrap_or(1_000_000);
    for (label, spread) in [("same-scale", 0), ("spread 2^±60", 60), ("mixed 2^±300", 300)] {
        let (rows, units) = fixture(n, spread);
        println!("-- {n} updates, 16 keys, one epoch, one worker, {label} --");
        if spread == 0 {
            let units = units.clone();
            let t = best(|| {
                let units = units.clone();
                timely::execute_directly(move |worker| {
                    let (mut input, probe) = worker.dataflow::<u64, _, _>(|scope| {
                        let (input, rows) = scope.new_collection::<(u32, i64), isize>();
                        let probe = rows
                            .explode(|(key, u)| Some(((key, ()), u as isize)))
                            .count_total()
                            .probe().0;
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
                })
            });
            row("isize", t, n);
            let r = rows.clone();
            let t = best(|| {
                timed(r.clone(), |rows| {
                    rows.explode(|(key, bits)| Some(((key, ()), Exact::from(f64::from_bits(bits)))))
                        .count_total()
                        .probe().0
                })
            });
            row("Exact, plain", t, n);
        } else {
            println!("  {:<16} (panics on this band)", "Exact, plain");
        }
        {
            let r = rows.clone();
            let t = best(|| {
                timed(r.clone(), |rows| {
                    rows.explode(|(key, bits)| {
                        let x = f64::from_bits(bits);
                        Some((((key, band(x)), ()), Exact::from(x)))
                    })
                    .count_total()
                    .map(|(((key, band), ()), total)| (key, (band, total)))
                    .reduce(|_key, bands: &[(&(u8, Exact), isize)], output: &mut Vec<(u64, isize)>| {
                        let mut sum = Wide::default();
                        for ((_band, total), mult) in bands {
                            sum += Wide::try_from(total).expect("inside Wide") * (*mult as i64);
                        }
                        output.push((sum.to_f64().to_bits(), 1));
                    })
                    .probe().0
                })
            });
            row("Exact, banded", t, n);
        }
        {
            let r = rows.clone();
            let t = best(|| {
                timed(r.clone(), |rows| {
                    rows.explode(|(key, bits)| Some(((key, ()), H::C(Exact::from(f64::from_bits(bits))))))
                        .count_total()
                        .probe().0
                })
            });
            row("heap-arm proxy", t, n);
        }
        {
            let r = rows.clone();
            let t = best(|| {
                timed(r.clone(), |rows| {
                    rows.explode(|(key, bits)| Some(((key, ()), W(Wide::from(f64::from_bits(bits))))))
                        .count_total()
                        .probe().0
                })
            });
            row("Wide, plain", t, n);
        }
    }
}
