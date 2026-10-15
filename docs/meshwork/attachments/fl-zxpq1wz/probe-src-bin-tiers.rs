//! The five histories of dd.rs on one key, with the diff type chosen by TIER (exact | wide |
//! fixed) and the pair of values by PAIR (huge: 2^1000 and 2^-1000; everyday: 1e12 and 1e-12).
//! `Wide` and `Fixed256` get differential's weight traits through local newtypes, since the crate
//! implements them on `Exact` alone. A value `Fixed256` refuses is reported at the door and the
//! run does not start: that refusal is the per-value check.
//!
//!   A: e0 +big | e1 -big +small
//!   B: e0 +big | e1 +small | e2 -big
//!   C: e0 +big +small -big
//!   D: e0 +big | e1 -big | e2 +small
//!   E: e0 +big -big +small

use differential_dataflow::difference::{Abelian, IsZero, Monoid, Multiply, Semigroup};
use differential_dataflow::input::Input;
use differential_dataflow::operators::CountTotal;
use differential_dataflow::ExchangeData;
use float_abel::{Exact, Fixed256, Wide};
use serde::{Deserialize, Serialize};

macro_rules! weight {
    ($name:ident, $inner:ty) => {
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
        #[serde(transparent)]
        struct $name($inner);
        impl IsZero for $name {
            fn is_zero(&self) -> bool {
                self.0.is_zero()
            }
        }
        impl Semigroup for $name {
            fn plus_equals(&mut self, rhs: &Self) {
                self.0 += rhs.0;
            }
        }
        impl Monoid for $name {
            fn zero() -> Self {
                Self(<$inner>::default())
            }
        }
        impl Abelian for $name {
            fn negate(&mut self) {
                self.0 = -self.0;
            }
        }
        impl Multiply<isize> for $name {
            type Output = Self;
            fn multiply(self, rhs: &isize) -> Self {
                Self(self.0 * *rhs)
            }
        }
    };
}
weight!(W, Wide);
weight!(F, Fixed256);

fn run<R>(epochs: Vec<Vec<(f64, isize)>>, to: fn(f64) -> R, read: fn(&R) -> f64)
where
    R: Abelian + ExchangeData + Multiply<isize, Output = R> + Ord,
{
    timely::execute_from_args(std::env::args(), move |worker| {
        let (mut rows, probe) = worker.dataflow::<u64, _, _>(|scope| {
            let (input, rows) = scope.new_collection::<(u32, u64), isize>();
            let (probe, _totals) = rows
                .explode(move |(key, bits)| Some((key, to(f64::from_bits(bits)))))
                .count_total()
                .inspect(move |((_, total), epoch, change)| {
                    println!("  epoch {epoch}: total {:e} ({change:+})", read(total));
                })
                .probe();
            (input, probe)
        });
        for (i, epoch) in epochs.iter().enumerate() {
            if worker.index() == 0 {
                for &(x, m) in epoch {
                    rows.update((1, x.to_bits()), m);
                }
            }
            rows.advance_to(i as u64 + 1);
            rows.flush();
            worker.step_while(|| probe.less_than(rows.time()));
        }
    })
    .expect("the workers start");
}

fn main() {
    let tier = std::env::var("TIER").expect("TIER=exact|wide|fixed");
    let scenario = std::env::var("SCENARIO").expect("SCENARIO=A..E");
    let (big, small) = match std::env::var("PAIR").expect("PAIR=huge|everyday").as_str() {
        "huge" => (2f64.powi(1000), 2f64.powi(-1000)),
        "everyday" => (1e12, 1e-12),
        other => panic!("unknown pair {other}"),
    };
    let epochs: Vec<Vec<(f64, isize)>> = match scenario.as_str() {
        "A" => vec![vec![(big, 1)], vec![(big, -1), (small, 1)]],
        "B" => vec![vec![(big, 1)], vec![(small, 1)], vec![(big, -1)]],
        "C" => vec![vec![(big, 1), (small, 1), (big, -1)]],
        "D" => vec![vec![(big, 1)], vec![(big, -1)], vec![(small, 1)]],
        "E" => vec![vec![(big, 1), (big, -1), (small, 1)]],
        other => panic!("unknown scenario {other}"),
    };
    match tier.as_str() {
        "exact" => run(epochs, Exact::from, Exact::to_f64),
        "wide" => run(epochs, |x| W(Wide::from(x)), |w| w.0.to_f64()),
        "fixed" => {
            for x in [big, small] {
                if let Err(refused) = Fixed256::try_from(x) {
                    println!("  refused at the door: {x:e}: {refused:?}");
                    return;
                }
            }
            run(epochs, |x| F(Fixed256::try_from(x).expect("checked at the door")), |f| f.0.to_f64())
        }
        other => panic!("unknown tier {other}"),
    }
}
