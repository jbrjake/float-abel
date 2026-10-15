//! One key's SUM in differential-dataflow, explode + count_total, under three histories that
//! all end at the same multiset {2^-1000}. SCENARIO picks the history.
//!
//!   A: e0 +2^1000 | e1 -2^1000 +2^-1000
//!   B: e0 +2^1000 | e1 +2^-1000 | e2 -2^1000
//!   C: e0 +2^1000 +2^-1000 -2^1000
//!   D: e0 +2^1000 | e1 -2^1000 | e2 +2^-1000
//!   E: e0 +2^1000 -2^1000 +2^-1000   (C's updates, in another arrival order)

use differential_dataflow::input::Input;
use differential_dataflow::operators::CountTotal;
use float_abel::Exact;

fn main() {
    let scenario = std::env::var("SCENARIO").expect("SCENARIO=A|B|C|D");
    let big = 2f64.powi(1000);
    let tiny = 2f64.powi(-1000);
    // Each epoch: a list of (value, +1 insert / -1 remove).
    let epochs: Vec<Vec<(f64, isize)>> = match scenario.as_str() {
        "A" => vec![vec![(big, 1)], vec![(big, -1), (tiny, 1)]],
        "B" => vec![vec![(big, 1)], vec![(tiny, 1)], vec![(big, -1)]],
        "C" => vec![vec![(big, 1), (tiny, 1), (big, -1)]],
        "D" => vec![vec![(big, 1)], vec![(big, -1)], vec![(tiny, 1)]],
        "E" => vec![vec![(big, 1), (big, -1), (tiny, 1)]],
        other => panic!("unknown scenario {other}"),
    };

    timely::execute_from_args(std::env::args(), move |worker| {
        let (mut rows, probe) = worker.dataflow::<u64, _, _>(|scope| {
            let (input, rows) = scope.new_collection::<(u32, u64), isize>();
            let (probe, _totals) = rows
                .explode(|(key, bits)| Some((key, Exact::from(f64::from_bits(bits)))))
                .count_total()
                .inspect(|((_, total), epoch, change)| {
                    println!("  epoch {epoch}: total {:e} ({change:+})", total.to_f64());
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
