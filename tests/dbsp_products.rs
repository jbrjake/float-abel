//! The heap `Exact` inside DBSP, DESIGN.md § DBSP: `aggregate_linear` promotes an account's
//! accumulator past the width, scales it by the `ZWeight`, and demotes it when the large rows
//! are retracted; and the same operator folds a row's product, `Exact::product` of two columns,
//! to a `num-bigint` oracle, the dot-product shape a `CREATE LINEAR AGGREGATE` UDA takes. Every
//! test is named `dbsp_products_*` so the task verify can filter on it; the topic compiles only
//! under the `dbsp` feature, on the toolchain CLAUDE.md § Commands names.

use dbsp::utils::Tup2;
use dbsp::{IndexedZSetHandle, OrdIndexedZSet, OutputHandle, Runtime, ZWeight};
use float_abel::Exact;
use num_bigint::BigInt;

use crate::dbsp::{changes, e, push, sorted, sum_circuit};
use crate::exact_heap::units;
use crate::exact_heap_ring::product_units;
use crate::support::Rng;

/// A key whose rows span more than the width folds to a promoted accumulator, which
/// `aggregate_linear` emits as the group's value with the `ZWeight` scaling each row; retracting
/// the large rows demotes each total to the compact small one, and retracting the rest deletes
/// the groups. The output at each step is the old total retracted and the new one inserted, so a
/// heap value is what the two workers consolidate and emit.
#[test]
fn dbsp_products_aggregate_linear_promotes_scales_and_demotes_the_accumulator() {
    let (mut circuit, (ledger, output)) = sum_circuit();

    push(
        &ledger,
        &[(1, 1e300, 1), (1, 1e-300, 1), (2, 1e300, 2), (2, 1e-300, 3)],
    );
    circuit.transaction().expect("step 0");
    let held1 = e(1e300) + e(1e-300);
    let held2 = e(1e300) * 2i64 + e(1e-300) * 3i64;
    assert!(!held1.is_compact() && !held2.is_compact());
    assert_eq!(
        changes(&output),
        vec![(1, held1.clone(), 1), (2, held2.clone(), 1)]
    );

    push(&ledger, &[(1, 1e300, -1), (2, 1e300, -2)]);
    circuit.transaction().expect("step 1");
    let small2 = e(1e-300) * 3i64;
    assert!(small2.is_compact());
    assert_eq!(
        changes(&output),
        sorted(vec![
            (1, held1, -1),
            (1, e(1e-300), 1),
            (2, held2, -1),
            (2, small2.clone(), 1),
        ])
    );

    push(&ledger, &[(1, 1e-300, -1), (2, 1e-300, -3)]);
    circuit.transaction().expect("step 2");
    assert_eq!(
        changes(&output),
        sorted(vec![(1, e(1e-300), -1), (2, small2, -1)])
    );

    circuit.kill().expect("the workers join");
}

/// A row of two amounts, each as its bits.
type Pair = Tup2<u64, u64>;
/// The dot-product circuit's two ends.
type Ends = (
    IndexedZSetHandle<u32, Pair>,
    OutputHandle<OrdIndexedZSet<u32, Exact>>,
);

/// The dot-product circuit on two workers: `aggregate_linear` folds each key's rows into the
/// `Exact::product` of the row's two amounts, scaled by the `ZWeight`, which is the accumulator a
/// UDA over two columns would hold.
fn dot_circuit() -> (dbsp::DBSPHandle, Ends) {
    Runtime::init_circuit(2, |circuit| {
        let (rows, handle) = circuit.add_input_indexed_zset::<u32, Pair>();
        let totals = rows.aggregate_linear(|pair: &Pair| {
            Exact::product(f64::from_bits(pair.0), f64::from_bits(pair.1))
        });
        Ok((handle, totals.output()))
    })
    .expect("the circuit builds")
}

/// `(key, a, b, multiplicity)` rows.
type Dot = (u32, f64, f64, ZWeight);

fn push_pairs(ledger: &IndexedZSetHandle<u32, Pair>, rows: &[Dot]) {
    for &(key, a, b, multiplicity) in rows {
        ledger.push(key, (Tup2(a.to_bits(), b.to_bits()), multiplicity));
    }
}

/// Each key's total of `rows`, as the ring folds it and in the oracle's units of `2^-2148`.
fn dot_totals(rows: &[Dot], keys: u32) -> Vec<(Exact, BigInt)> {
    (0..keys)
        .map(|key| {
            let mut total = Exact::ZERO;
            let mut oracle = BigInt::from(0);
            for &(k, a, b, m) in rows.iter().filter(|row| row.0 == key) {
                debug_assert_eq!(k, key);
                total += Exact::product(a, b) * m;
                oracle += units(a) * units(b) * m;
            }
            (total, oracle)
        })
        .collect()
}

/// A fold of row products over 2^±300 with multiplicities promotes on every key, lands the
/// `num-bigint` oracle's bits, follows the retraction of half the rows, and deletes each group
/// when the rest go: what docs/OVERFLOW.md § The banded idiom said no 32-byte diff could hold.
#[test]
fn dbsp_products_aggregate_linear_folds_row_products_to_the_oracle() {
    const KEYS: u32 = 3;
    let mut rng = Rng(67);
    let rows: Vec<Dot> = (0..KEYS)
        .flat_map(|key| (0..40).map(move |_| key))
        .collect::<Vec<u32>>()
        .into_iter()
        .map(|key| {
            let a = rng.on_grid(-300, 300);
            let b = rng.on_grid(-300, 300);
            let m = (rng.next() % 3) as ZWeight + 1;
            (key, a, b, m)
        })
        .collect();
    let (mut circuit, (ledger, output)) = dot_circuit();

    push_pairs(&ledger, &rows);
    circuit.transaction().expect("step 0");
    let before = dot_totals(&rows, KEYS);
    for (key, (total, oracle)) in before.iter().enumerate() {
        assert!(!total.is_compact(), "key {key}");
        assert_eq!(product_units(total), *oracle, "key {key}");
    }
    assert_eq!(
        changes(&output),
        before
            .iter()
            .enumerate()
            .map(|(key, (total, _))| (key as u32, total.clone(), 1))
            .collect::<Vec<_>>()
    );

    let retracted: Vec<Dot> = rows
        .iter()
        .step_by(2)
        .map(|&(key, a, b, m)| (key, a, b, -m))
        .collect();
    let kept: Vec<Dot> = rows.iter().skip(1).step_by(2).copied().collect();
    push_pairs(&ledger, &retracted);
    circuit.transaction().expect("step 1");
    let after = dot_totals(&kept, KEYS);
    let mut want = Vec::new();
    for key in 0..KEYS as usize {
        assert_eq!(product_units(&after[key].0), after[key].1, "key {key}");
        want.push((key as u32, before[key].0.clone(), -1));
        want.push((key as u32, after[key].0.clone(), 1));
    }
    assert_eq!(changes(&output), sorted(want));

    let rest: Vec<Dot> = kept.iter().map(|&(key, a, b, m)| (key, a, b, -m)).collect();
    push_pairs(&ledger, &rest);
    circuit.transaction().expect("step 2");
    assert_eq!(
        changes(&output),
        after
            .iter()
            .enumerate()
            .map(|(key, (total, _))| (key as u32, total.clone(), -1))
            .collect::<Vec<_>>()
    );

    circuit.kill().expect("the workers join");
}
