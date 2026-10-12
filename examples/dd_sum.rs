//! The `explode` + `count_total` idiom, DESIGN.md § differential-dataflow: a `SUM` over a float
//! column, held exactly, with retractions that cancel to the one zero.
//!
//! ```bash
//! cargo run --example dd_sum -- -w 3
//! ```
//!
//! A row is `(account, amount)`, the amount as the double's bits because a collection's data must
//! be `Ord` and `f64` is not. `explode` turns the bits back into a double and the double into the
//! diff, so an account's total *is* its diff, and `count_total` reads it out as data with
//! multiplicity one. A delete is the row with multiplicity `-1`, which `Multiply<isize>` turns into
//! `-1 × Exact`; a total that cancels to zero is deleted by `is_zero`, and a total that reads out
//! is the correctly rounded double of the exact sum, so no order, worker count or timing of
//! consolidation moves its bits.

use differential_dataflow::input::Input;
use differential_dataflow::operators::CountTotal;
use float_abel::Exact;

/// Account 1 sums three tenths, which `f64` adds to `0.6000000000000001`; account 2 adds `1.0`
/// between `1e16` and `-1e16`, which `f64` loses whole.
const ROWS: [(u32, f64); 6] = [
    (1, 0.1),
    (1, 0.2),
    (1, 0.3),
    (2, 1e16),
    (2, 1.0),
    (2, -1e16),
];

fn main() {
    timely::execute_from_args(std::env::args(), |worker| {
        let (mut rows, probe) = worker.dataflow::<u64, _, _>(|scope| {
            let (input, rows) = scope.new_collection::<(u32, u64), isize>();
            let (probe, _totals) = rows
                .explode(|(account, bits)| Some((account, Exact::from(f64::from_bits(bits)))))
                .count_total()
                .inspect(|((account, total), epoch, change)| {
                    println!(
                        "epoch {epoch}: account {account} total {} ({change:+})",
                        total.to_f64()
                    );
                })
                .probe();
            (input, probe)
        });

        // Worker 0 feeds; exchange spreads the rows by account.
        if worker.index() == 0 {
            for (account, amount) in ROWS {
                rows.insert((account, amount.to_bits()));
            }
        }
        rows.advance_to(1);
        rows.flush();
        worker.step_while(|| probe.less_than(rows.time()));

        // Epoch 1 retracts every row: each total cancels to the one zero, and the key goes.
        if worker.index() == 0 {
            for (account, amount) in ROWS {
                rows.remove((account, amount.to_bits()));
            }
        }
        rows.advance_to(2);
        rows.flush();
        worker.step_while(|| probe.less_than(rows.time()));
    })
    .expect("the workers start");
}
