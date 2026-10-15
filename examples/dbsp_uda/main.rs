//! Feldera's `CREATE LINEAR AGGREGATE` route to an exact `SUM` over doubles, DESIGN.md § DBSP.
//!
//! ```bash
//! cargo +1.99.0 run --features dbsp --example dbsp_uda
//! ```
//!
//! SQL's own `SUM(DOUBLE)` is fixed to `f64` by Feldera's compiler and cannot be made linear from
//! outside, and a custom DBSP weight is only ever an accumulator or an output. So the sanctioned
//! route is a user-defined linear aggregate whose accumulator is [`Exact`]: the compiler wraps it
//! in `Tup3<acc, i64, i64>` for the NULL and row counts and folds it with `aggregate_linear`, where
//! a retraction arrives as `mul_by_ref(&-1)` and a group whose accumulator reaches the exact zero
//! is deleted by `is_zero`.
//!
//! The three items Feldera's udf.rs must hold for `program.sql`'s
//! `CREATE LINEAR AGGREGATE exact_sum(value DOUBLE) RETURNS DOUBLE` are below, named by the
//! aggregate with the suffixes udf.md fixes: `exact_sum_accumulator_type`, `exact_sum_map` and
//! `exact_sum_post`. udf.toml beside this file names the dependency. `main` drives the same three
//! items through `aggregate_linear` on two workers, so the mechanism runs here without Feldera.

use dbsp::algebra::F64;
use dbsp::{IndexedZSetReader, Runtime, ZWeight};
use float_abel::Exact;

/// The accumulator: the exact sum of every row the group holds, a 32-byte word.
#[allow(non_camel_case_types)]
pub type exact_sum_accumulator_type = Exact;

/// One row's contribution. `From<f64>` is total, so a NaN or an infinity is counted, never
/// refused, and projects at `exact_sum_post` the way IEEE would have.
#[must_use]
pub fn exact_sum_map(value: F64) -> exact_sum_accumulator_type {
    Exact::from(value.into_inner())
}

/// The aggregate's result: the exact total rounded once, to nearest-even.
#[must_use]
#[allow(clippy::needless_pass_by_value)] // Feldera's `_post` takes the accumulator by value
pub fn exact_sum_post(acc: exact_sum_accumulator_type) -> F64 {
    F64::new(acc.to_f64())
}

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

fn main() -> Result<(), dbsp::Error> {
    let (mut circuit, (ledger, totals)) = Runtime::init_circuit(2, |circuit| {
        let (rows, handle) = circuit.add_input_indexed_zset::<u32, F64>();
        let totals = rows.aggregate_linear(|amount| exact_sum_map(*amount));
        Ok((handle, totals.output()))
    })?;

    for (step, multiplicity) in [(0, 1), (1, -1)] {
        for (account, amount) in ROWS {
            ledger.push(account, (F64::new(amount), ZWeight::from(multiplicity)));
        }
        circuit.transaction()?;
        for (account, acc, change) in totals.consolidate().iter() {
            println!(
                "step {step}: account {account} total {} ({change:+})",
                exact_sum_post(acc).into_inner()
            );
        }
    }

    circuit.kill().expect("the workers join");
    Ok(())
}
