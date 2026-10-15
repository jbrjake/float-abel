//! `Exact` as a DBSP weight, DESIGN.md § DBSP: a weighted batch consolidates and cancels exactly,
//! `aggregate_linear` scales each row by its `ZWeight` and deletes a group on the exact zero, the
//! archived form orders as `Ord` does and refuses a non-canonical word, and the integer scaling
//! is the ring's. Every test is named `dbsp_*` so the task verify can filter on it; the topic
//! compiles only under the `dbsp` feature, on the toolchain CLAUDE.md § Commands names.
//!
//! A row carries its amount as the double's bits where it is a value, because `f64` is not `Ord`;
//! dbsp's own `F64` would do too, and `examples/dbsp_uda` uses it.

use core::cmp::Ordering;
use std::panic::{catch_unwind, AssertUnwindSafe};

use dbsp::algebra::{HasOne, HasZero, MulByRef};
use dbsp::dynamic::{DowncastTrait, DynWeightTyped, OrdRepr};
use dbsp::trace::{BatchReader as _, Cursor as _};
use dbsp::typed_batch::BatchReader as _;
use dbsp::utils::Tup2;
use dbsp::{
    IndexedZSetHandle, IndexedZSetReader, OrdIndexedWSet, OrdIndexedZSet, OutputHandle, Runtime,
    ZWeight,
};
use float_abel::Exact;
use rkyv::{AlignedVec, Deserialize as _};

use crate::support::Rng;

/// A batch keyed and valued by `u32`, weighted by `Exact`.
type Weighted = OrdIndexedWSet<u32, u32, Exact, DynWeightTyped<Exact>>;
/// The SUM circuit's two ends: rows keyed by account with the amount's bits as the value, and
/// each account's `Exact` total as a value with a `ZWeight` multiplicity.
type Ends = (
    IndexedZSetHandle<u32, u64>,
    OutputHandle<OrdIndexedZSet<u32, Exact>>,
);

fn e(x: f64) -> Exact {
    Exact::from(x)
}

/// Every `(key, value, weight)` of `batch`, in its order. A custom-weight batch has no `iter`,
/// which `IndexedZSetReader` keeps for `ZWeight`, so the cursor reads it: the key and value
/// downcast with a check, and the typed weight derefs.
fn tuples(batch: &Weighted) -> Vec<(u32, u32, Exact)> {
    let mut out = Vec::new();
    let mut cursor = batch.inner().cursor();
    while cursor.key_valid() {
        while cursor.val_valid() {
            let key = *cursor.key().downcast_checked::<u32>();
            let val = *cursor.val().downcast_checked::<u32>();
            let weight: Exact = (**cursor.weight()).clone();
            out.push((key, val, weight));
            cursor.step_val();
        }
        cursor.step_key();
    }
    out
}

fn weighted(rows: &[((u32, u32), f64)]) -> Weighted {
    Weighted::from_tuples(
        (),
        rows.iter()
            .map(|&((k, v), x)| Tup2(Tup2(k, v), e(x)))
            .collect(),
    )
}

/// The SUM circuit on two workers, the idiom `examples/dbsp_uda` documents: `aggregate_linear`
/// folds each account's amounts into an `Exact` accumulator, scaling each row by its `ZWeight`.
fn sum_circuit() -> (dbsp::DBSPHandle, Ends) {
    Runtime::init_circuit(2, |circuit| {
        let (rows, handle) = circuit.add_input_indexed_zset::<u32, u64>();
        let totals = rows.aggregate_linear(|bits: &u64| e(f64::from_bits(*bits)));
        Ok((handle, totals.output()))
    })
    .expect("the circuit builds")
}

/// The output's change at the last step: `(account, total, multiplicity)`, consolidated over
/// the workers, in the batch's order.
fn changes(output: &OutputHandle<OrdIndexedZSet<u32, Exact>>) -> Vec<(u32, Exact, ZWeight)> {
    output.consolidate().iter().collect()
}

/// `expected` in a batch's order: by key, then by value, which for an `Exact` is numeric with
/// the counts breaking a tie.
fn sorted(mut expected: Vec<(u32, Exact, ZWeight)>) -> Vec<(u32, Exact, ZWeight)> {
    expected.sort_by(|a, b| (a.0, &a.1).cmp(&(b.0, &b.1)));
    expected
}

fn push(ledger: &IndexedZSetHandle<u32, u64>, rows: &[(u32, f64, ZWeight)]) {
    for &(account, amount, multiplicity) in rows {
        ledger.push(account, (amount.to_bits(), multiplicity));
    }
}

/// `n` compact values over the tiers, seeded: a double as drawn, with a NaN or an infinity one
/// time in sixteen; the exact sum of two such doubles, which spans far more than 53 bits when
/// their exponents differ; the exact product of two, which spans 106; and an integer multiple. A
/// result in the heap form, or past a cap, falls back to the first double alone, because the
/// archived form is the compact word's.
fn mixed(n: usize) -> Vec<Exact> {
    let mut rng = Rng(0x5EED_DB5B);
    (0..n)
        .map(|i| {
            let a = e(rng.any());
            let b = e(rng.any());
            let k = (rng.next() % 2001) as i64 - 1000;
            let result = match i % 4 {
                0 => Ok(a.clone()),
                1 => a.checked_add(&b),
                2 => a.checked_mul(&b),
                _ => a.checked_mul_i64(k),
            };
            result.ok().filter(Exact::is_compact).unwrap_or(a)
        })
        .collect()
}

/// `x` in rkyv's archived form, as dbsp's storage writes it.
fn archived(x: &Exact) -> AlignedVec {
    rkyv::to_bytes::<_, 64>(x).expect("a word serialises")
}

/// The archived root of `bytes`, which [`archived`] wrote.
fn root(bytes: &AlignedVec) -> &rkyv::Archived<Exact> {
    // SAFETY: `bytes` is what `rkyv::to_bytes` wrote for an `Exact`, or that with one bit
    // flipped, and every bit pattern of six integers is a structurally valid archived word.
    unsafe { rkyv::archived_root::<Exact>(bytes) }
}

/// Two tenths on one key consolidate to one tuple carrying their exact sum, and merging in their
/// negations leaves no tuple at all: each pair cancelled to the one zero, which `is_zero`
/// deleted. The f64 control, the same four adds, leaves residue.
#[test]
fn dbsp_weighted_batch_consolidates_exactly_and_cancels_on_merge() {
    assert_ne!(
        0.1 + 0.2 - 0.2 - 0.1,
        0.0,
        "the f64 control must leave residue"
    );
    let batch = weighted(&[((1, 10), 0.1), ((1, 10), 0.2), ((2, 20), -0.3)]);
    assert_eq!(
        tuples(&batch),
        vec![(1, 10, e(0.1) + e(0.2)), (2, 20, e(-0.3))]
    );
    let negated = weighted(&[((1, 10), -0.2), ((1, 10), -0.1), ((2, 20), 0.3)]);
    assert_eq!(tuples(&batch.merge(&negated)), vec![]);
    let half = batch.merge(&weighted(&[((1, 10), -0.2)]));
    assert_eq!(tuples(&half), vec![(1, 10, e(0.1)), (2, 20, e(-0.3))]);
}

/// A key whose weights span more than the width consolidates to a weight in the heap form, which
/// an in-memory batch holds as any other, and merging in the retraction of the large weight
/// demotes the key's weight to the compact small one.
#[test]
fn dbsp_weighted_batch_holds_a_promoted_weight_and_demotes_it_on_merge() {
    let batch = weighted(&[((1, 10), 1e300), ((1, 10), 1e-300), ((2, 20), 0.5)]);
    let promoted = e(1e300) + e(1e-300);
    assert!(!promoted.is_compact());
    assert_eq!(
        tuples(&batch),
        vec![(1, 10, promoted.clone()), (2, 20, e(0.5))]
    );
    let demoted = batch.merge(&weighted(&[((1, 10), -1e300)]));
    assert_eq!(tuples(&demoted), vec![(1, 10, e(1e-300)), (2, 20, e(0.5))]);
    let cancelled = batch.merge(&weighted(&[((1, 10), -1e-300), ((1, 10), -1e300)]));
    assert_eq!(tuples(&cancelled), vec![(2, 20, e(0.5))]);
}

/// Each row's contribution is scaled by its `ZWeight`, so three copies of `0.2` land `0.2 × 3`,
/// and a change is the retraction of the old total and the insertion of the new one. Retracting
/// every row of an account leaves its accumulator at the exact zero, which deletes the group:
/// the output is the retraction alone.
#[test]
fn dbsp_aggregate_linear_scales_by_zweight_and_deletes_the_zero_group() {
    let (mut circuit, (ledger, output)) = sum_circuit();

    push(&ledger, &[(1, 0.1, 1), (1, 0.2, 3), (2, 0.5, 1)]);
    circuit.transaction().expect("step 0");
    let total1 = e(0.1) + e(0.2) * 3i64;
    assert_eq!(
        changes(&output),
        vec![(1, total1.clone(), 1), (2, e(0.5), 1)]
    );

    push(
        &ledger,
        &[(1, 0.1, -1), (1, 0.2, -3), (2, 0.5, -1), (2, 0.25, 1)],
    );
    circuit.transaction().expect("step 1");
    assert_eq!(
        changes(&output),
        sorted(vec![
            (1, total1.clone(), -1),
            (2, e(0.5), -1),
            (2, e(0.25), 1)
        ])
    );

    push(&ledger, &[(2, 0.25, -1)]);
    circuit.transaction().expect("step 2");
    assert_eq!(changes(&output), vec![(2, e(0.25), -1)]);

    circuit.kill().expect("the workers join");
}

/// `From<f64>` is total, so a NaN or an infinity is a row like any other: counted into the
/// accumulator, scaled by its multiplicity, and retracted exactly, where an `f64` accumulator
/// could never retract a NaN.
#[test]
fn dbsp_specials_ride_the_accumulator_and_retract_exactly() {
    let (mut circuit, (ledger, output)) = sum_circuit();

    push(
        &ledger,
        &[
            (7, 1.5, 1),
            (7, f64::NAN, 1),
            (7, f64::INFINITY, 1),
            (7, 2.5, 2),
        ],
    );
    circuit.transaction().expect("step 0");
    let with_nan = e(1.5) + e(f64::NAN) + e(f64::INFINITY) + e(2.5) * 2i64;
    assert!(with_nan.to_f64().is_nan());
    assert_eq!(with_nan.specials(), Some((1, 1, 0)));
    assert_eq!(changes(&output), vec![(7, with_nan.clone(), 1)]);

    push(&ledger, &[(7, f64::NAN, -1)]);
    circuit.transaction().expect("step 1");
    let without = e(1.5) + e(f64::INFINITY) + e(2.5) * 2i64;
    assert_eq!(without.to_f64(), f64::INFINITY);
    assert_eq!(
        changes(&output),
        sorted(vec![(7, with_nan.clone(), -1), (7, without, 1)])
    );

    circuit.kill().expect("the workers join");
}

/// `OrdRepr`'s contract: the archived form of `a` compared with `b` is `a.cmp(&b)`, and two
/// archived forms order as their values do, on 10⁵ values over the tiers with the specials in,
/// each against itself, its neighbour and a random partner. The three orderings are each seen
/// at least a thousand times, so the check is not vacuous.
#[test]
fn dbsp_archived_order_agrees_with_ord_on_mixed_tiers_and_specials() {
    let values = mixed(100_000);
    let bytes: Vec<AlignedVec> = values.iter().map(archived).collect();
    let mut rng = Rng(7);
    let mut seen = [0usize; 3];
    for (i, a) in values.iter().enumerate() {
        let arch = root(&bytes[i]);
        for j in [
            i,
            (i + 1) % values.len(),
            rng.next() as usize % values.len(),
        ] {
            let b = &values[j];
            let expected = a.cmp(b);
            seen[(expected as i8 + 1) as usize] += 1;
            assert_eq!(arch.ord_cmp(b), expected, "{a:?} vs {b:?}");
            let arch_b = root(&bytes[j]);
            assert_eq!(arch.cmp(arch_b), expected, "{a:?} vs {b:?}");
            assert_eq!(
                arch == arch_b,
                expected == Ordering::Equal,
                "{a:?} vs {b:?}"
            );
        }
    }
    assert!(
        seen.iter().all(|&n| n >= 1_000),
        "orderings seen (less, equal, greater): {seen:?}"
    );
}

/// The archived form is the 32-byte word itself, the wire body without its version byte, and it
/// deserialises back to the value.
#[test]
fn dbsp_archived_word_is_the_wire_body_and_round_trips() {
    for x in mixed(1_000) {
        let bytes = archived(&x);
        assert_eq!(bytes.as_slice(), &x.to_bytes()[1..]);
        let back: Exact = root(&bytes)
            .deserialize(&mut rkyv::Infallible)
            .expect("infallible");
        assert_eq!(back, x);
    }
}

/// A word with an even mantissa is not a value, and no operation writes one; one that reaches
/// the archived form from outside is refused with the invariant named, by the deserialiser and
/// by the comparisons alike, because dbsp's deserialiser has no error channel.
#[test]
fn dbsp_a_non_canonical_archived_word_is_refused() {
    let mut raw = archived(&e(0.1)).to_vec();
    raw[0] ^= 1;
    let mut bytes = AlignedVec::new();
    bytes.extend_from_slice(&raw);
    let arch = root(&bytes);
    let refused = |run: &dyn Fn()| {
        let payload = catch_unwind(AssertUnwindSafe(run)).expect_err("refused");
        let message = payload
            .downcast_ref::<String>()
            .cloned()
            .unwrap_or_default();
        assert!(message.contains("canonical"), "{message}");
    };
    refused(&|| {
        let _: Exact = arch.deserialize(&mut rkyv::Infallible).expect("infallible");
    });
    refused(&|| {
        let _ = arch.ord_cmp(&e(0.1));
    });
    refused(&|| {
        let _ = arch.cmp(arch);
    });
}

/// `MulByRef<i64>` and `MulByRef<isize>`, what `aggregate_linear` scales a row's contribution by,
/// are the ring's `Mul` by the integer; `MulByRef<Exact>` is the ring product; and `HasZero` and
/// `HasOne` are the group's zero and the ring's one.
#[test]
fn dbsp_mul_by_ref_and_the_constants_are_the_ring_s() {
    let values = [0.1, -2.5, 1e300, 5e-324, f64::NAN, f64::INFINITY];
    for x in values.map(e) {
        for k in [-3i64, -1, 0, 1, 2, 1_000] {
            assert_eq!(x.mul_by_ref(&k), &x * k, "{x:?} × {k}");
            assert_eq!(x.mul_by_ref(&(k as isize)), &x * k, "{x:?} × {k}");
        }
        for y in [2.0, -4.0, 0.0].map(e) {
            assert_eq!(x.mul_by_ref(&y), &x * &y, "{x:?} × {y:?}");
        }
        assert_eq!(HasZero::is_zero(&x), x == Exact::ZERO);
    }
    assert_eq!(<Exact as HasZero>::zero(), Exact::ZERO);
    assert!(HasZero::is_zero(&<Exact as HasZero>::zero()));
    assert_eq!(<Exact as HasOne>::one(), e(1.0));
    assert_eq!(e(0.1).mul_by_ref(&<Exact as HasOne>::one()), e(0.1));
}
