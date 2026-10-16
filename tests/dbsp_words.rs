//! `Wide` and `Fixed` as DBSP weights, DESIGN.md § DBSP: the `Copy` words for a weighted batch
//! and for `aggregate_linear`'s accumulator, the `CREATE LINEAR AGGREGATE` shape, `Fixed` with the
//! door check inside the aggregate's closure. A weighted batch consolidates and cancels exactly on
//! either word, `aggregate_linear` scales each row by its `ZWeight` and deletes a group on the
//! exact zero, the archived form of either word is the word itself and orders as `Ord` does, and
//! `MulByRef` by `i64` and `isize` is the group's integer scaling. Every test is named
//! `dbsp_wide_*` or `dbsp_fixed_*` so the task verify can filter on it; the topic compiles only
//! under the `dbsp` feature, on the toolchain CLAUDE.md § Commands names.
//!
//! A row carries its amount as the double's bits where it is a value, because `f64` is not `Ord`.

use core::cmp::Ordering;
use core::ops::{Add, Mul};

use dbsp::algebra::{HasZero, MulByRef};
use dbsp::dynamic::{ArchivedDBData, DowncastTrait, DynWeightTyped, OrdRepr};
use dbsp::trace::{BatchReader as _, Cursor as _};
use dbsp::typed_batch::BatchReader as _;
use dbsp::utils::Tup2;
use dbsp::{
    DBWeight, IndexedZSetHandle, IndexedZSetReader, OrdIndexedWSet, OrdIndexedZSet, OutputHandle,
    Runtime, ZWeight,
};
use float_abel::{Fixed256, OutOfWindow, Wide};
use rkyv::ser::serializers::AllocSerializer;
use rkyv::{AlignedVec, Deserialize, Infallible, Serialize};

use crate::dbsp::push;
use crate::support::{pow2, Rng};

/// What the two words share here: a door that admits a double or refuses it by name, a draw
/// inside the word's window, and the wire body the archived form must equal. The supertraits are
/// what dbsp asks of a weight that is also `aggregate_linear`'s accumulator, and the claim the
/// topic pins: the word is its own archive, dbsp's `ArchivedDBData<Repr = Self>`, which is rkyv's
/// `Archived<Self> = Self` through dbsp's own supertrait, ordering as itself.
trait Word:
    DBWeight
    + Copy
    + MulByRef<i64, Output = Self>
    + MulByRef<isize, Output = Self>
    + Add<Output = Self>
    + Mul<i64, Output = Self>
    + ArchivedDBData<Repr = Self>
    + Serialize<AllocSerializer<64>>
    + Deserialize<Self, Infallible>
    + OrdRepr<Self>
{
    const NAME: &'static str;
    fn admit(x: f64) -> Result<Self, OutOfWindow>;
    /// A double the window holds, with room for a sum of two and a scaling by a thousand.
    fn draw(rng: &mut Rng) -> f64;
    /// The canonical wire body: the encoding behind its version byte.
    fn body(&self) -> Vec<u8>;
}

impl Word for Wide {
    const NAME: &'static str = "Wide";

    fn admit(x: f64) -> Result<Self, OutOfWindow> {
        Ok(Wide::from(x))
    }

    /// Any double, one in sixteen a special.
    fn draw(rng: &mut Rng) -> f64 {
        rng.any()
    }

    fn body(&self) -> Vec<u8> {
        self.to_bytes()[1..].to_vec()
    }
}

impl Word for Fixed256 {
    const NAME: &'static str = "Fixed256";

    fn admit(x: f64) -> Result<Self, OutOfWindow> {
        Fixed256::try_from(x)
    }

    /// On the grid, so the leading bit at or above `2^-40`, and below `2^80`, so a thousand
    /// times a sum of two stays under the ceiling of `2^95`.
    fn draw(rng: &mut Rng) -> f64 {
        rng.on_grid(-40, 80)
    }

    fn body(&self) -> Vec<u8> {
        self.to_bytes::<{ Fixed256::BYTES }>()[1..].to_vec()
    }
}

/// `x` through the door, which this topic's fixtures are all inside.
fn admit<R: Word>(x: f64) -> R {
    R::admit(x).unwrap_or_else(|e| panic!("{}: {x:e} refused at the door: {e}", R::NAME))
}

/// A batch keyed and valued by `u32`, weighted by `R`.
type Weighted<R> = OrdIndexedWSet<u32, u32, R, DynWeightTyped<R>>;

/// Every `(key, value, weight)` of `batch`, in its order, read through the cursor as
/// tests/dbsp.rs reads an `Exact`-weighted batch.
fn tuples<R: Word>(batch: &Weighted<R>) -> Vec<(u32, u32, R)> {
    let mut out = Vec::new();
    let mut cursor = batch.inner().cursor();
    while cursor.key_valid() {
        while cursor.val_valid() {
            let key = *cursor.key().downcast_checked::<u32>();
            let val = *cursor.val().downcast_checked::<u32>();
            let weight: R = **cursor.weight();
            out.push((key, val, weight));
            cursor.step_val();
        }
        cursor.step_key();
    }
    out
}

fn weighted<R: Word>(rows: &[((u32, u32), f64)]) -> Weighted<R> {
    Weighted::from_tuples(
        (),
        rows.iter()
            .map(|&((k, v), x)| Tup2(Tup2(k, v), admit::<R>(x)))
            .collect(),
    )
}

/// Two tenths on one key consolidate to one tuple carrying their exact sum, and merging in their
/// negations leaves no tuple at all: each pair cancelled to the one zero, which `is_zero`
/// deleted. The f64 control, the same four adds, leaves residue.
fn weighted_batch_consolidates_and_cancels<R: Word>() {
    assert_ne!(
        0.1 + 0.2 - 0.2 - 0.1,
        0.0,
        "the f64 control must leave residue"
    );
    let batch = weighted::<R>(&[((1, 10), 0.1), ((1, 10), 0.2), ((2, 20), -0.3)]);
    let got = tuples(&batch);
    assert_eq!(got.len(), 2, "{}", R::NAME);
    assert_eq!(
        got,
        vec![
            (1, 10, admit::<R>(0.1) + admit::<R>(0.2)),
            (2, 20, admit::<R>(-0.3))
        ]
    );
    let negated = weighted::<R>(&[((1, 10), -0.2), ((1, 10), -0.1), ((2, 20), 0.3)]);
    assert_eq!(tuples(&batch.merge(&negated)), vec![], "{}", R::NAME);
    let half = batch.merge(&weighted(&[((1, 10), -0.2)]));
    assert_eq!(
        tuples(&half),
        vec![(1, 10, admit::<R>(0.1)), (2, 20, admit::<R>(-0.3))],
        "{}",
        R::NAME
    );
}

#[test]
fn dbsp_wide_weighted_batch_consolidates_exactly_and_cancels_on_merge() {
    weighted_batch_consolidates_and_cancels::<Wide>();
}

#[test]
fn dbsp_fixed_weighted_batch_consolidates_exactly_and_cancels_on_merge() {
    weighted_batch_consolidates_and_cancels::<Fixed256>();
}

/// The SUM circuit's two ends: rows keyed by account with the amount's bits as the value, and
/// each account's `R` total as a value with a `ZWeight` multiplicity.
type Ends<R> = (
    IndexedZSetHandle<u32, u64>,
    OutputHandle<OrdIndexedZSet<u32, R>>,
);

/// The SUM circuit on two workers with `R` as the accumulator: `aggregate_linear` folds each
/// account's amounts, the door check inside its closure, scaling each row by its `ZWeight`.
fn sum_circuit<R: Word>() -> (dbsp::DBSPHandle, Ends<R>) {
    Runtime::init_circuit(2, |circuit| {
        let (rows, handle) = circuit.add_input_indexed_zset::<u32, u64>();
        let totals = rows.aggregate_linear(|bits: &u64| admit::<R>(f64::from_bits(*bits)));
        Ok((handle, totals.output()))
    })
    .expect("the circuit builds")
}

/// The output's change at the last step: `(account, total, multiplicity)`, consolidated over
/// the workers, in the batch's order.
fn changes<R: Word>(output: &OutputHandle<OrdIndexedZSet<u32, R>>) -> Vec<(u32, R, ZWeight)> {
    output.consolidate().iter().collect()
}

/// `expected` in a batch's order: by key, then by value, which for a word is numeric.
fn sorted<R: Word>(mut expected: Vec<(u32, R, ZWeight)>) -> Vec<(u32, R, ZWeight)> {
    expected.sort_by_key(|a| (a.0, a.1));
    expected
}

/// Each row's contribution is scaled by its `ZWeight`, so three copies of `0.2` land `0.2 × 3`,
/// and a change is the retraction of the old total and the insertion of the new one. Retracting
/// every row of an account leaves its accumulator at the exact zero, which deletes the group:
/// the output is the retraction alone.
fn aggregate_linear_scales_and_deletes_the_zero_group<R: Word>() {
    let (mut circuit, (ledger, output)) = sum_circuit::<R>();

    push(&ledger, &[(1, 0.1, 1), (1, 0.2, 3), (2, 0.5, 1)]);
    circuit.transaction().expect("step 0");
    let total1 = admit::<R>(0.1) + admit::<R>(0.2) * 3i64;
    assert_eq!(
        changes(&output),
        vec![(1, total1, 1), (2, admit::<R>(0.5), 1)],
        "{}",
        R::NAME
    );

    push(
        &ledger,
        &[(1, 0.1, -1), (1, 0.2, -3), (2, 0.5, -1), (2, 0.25, 1)],
    );
    circuit.transaction().expect("step 1");
    assert_eq!(
        changes(&output),
        sorted(vec![
            (1, total1, -1),
            (2, admit::<R>(0.5), -1),
            (2, admit::<R>(0.25), 1)
        ]),
        "{}",
        R::NAME
    );

    push(&ledger, &[(2, 0.25, -1)]);
    circuit.transaction().expect("step 2");
    assert_eq!(
        changes(&output),
        vec![(2, admit::<R>(0.25), -1)],
        "{}",
        R::NAME
    );

    circuit.kill().expect("the workers join");
}

#[test]
fn dbsp_wide_aggregate_linear_scales_by_zweight_and_deletes_the_zero_group() {
    aggregate_linear_scales_and_deletes_the_zero_group::<Wide>();
}

/// The door check runs inside the aggregate's closure, so a value the window cannot hold is
/// refused by name where the row becomes a weight; the values the window holds fold as `Wide`'s.
#[test]
fn dbsp_fixed_aggregate_linear_scales_by_zweight_and_deletes_the_zero_group() {
    assert_eq!(
        Fixed256::try_from(pow2(1000)),
        Err(OutOfWindow::AboveCeiling)
    );
    assert_eq!(Fixed256::try_from(1e-300), Err(OutOfWindow::OffGrid));
    assert_eq!(Fixed256::try_from(f64::NAN), Err(OutOfWindow::NotFinite));
    aggregate_linear_scales_and_deletes_the_zero_group::<Fixed256>();
}

/// `From<f64>` is total on `Wide`, so a NaN or an infinity is a row like any other: counted into
/// the accumulator, scaled by its multiplicity, and retracted exactly, where an `f64` accumulator
/// could never retract a NaN.
#[test]
fn dbsp_wide_specials_ride_the_accumulator_and_retract_exactly() {
    let (mut circuit, (ledger, output)) = sum_circuit::<Wide>();
    let w = Wide::from;

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
    let with_nan = w(1.5) + w(f64::NAN) + w(f64::INFINITY) + w(2.5) * 2i64;
    assert!(with_nan.to_f64().is_nan());
    assert_eq!(with_nan.specials(), (1, 1, 0));
    assert_eq!(changes(&output), vec![(7, with_nan, 1)]);

    push(&ledger, &[(7, f64::NAN, -1)]);
    circuit.transaction().expect("step 1");
    let without = w(1.5) + w(f64::INFINITY) + w(2.5) * 2i64;
    assert_eq!(without.to_f64(), f64::INFINITY);
    assert_eq!(
        changes(&output),
        sorted(vec![(7, with_nan, -1), (7, without, 1)])
    );

    circuit.kill().expect("the workers join");
}

/// `n` values of `R` over its window, seeded: a double as drawn, the sum of two, and an integer
/// multiple, so the three orderings each show.
fn mixed<R: Word>(n: usize) -> Vec<R> {
    let mut rng = Rng(0x5EED_C0DE);
    (0..n)
        .map(|i| {
            let a = admit::<R>(R::draw(&mut rng));
            let b = admit::<R>(R::draw(&mut rng));
            let k = (rng.next() % 2001) as i64 - 1000;
            match i % 3 {
                0 => a,
                1 => a + b,
                _ => a * k,
            }
        })
        .collect()
}

/// `x` in rkyv's archived form, as dbsp's storage writes it.
fn archived<R: Word>(x: &R) -> AlignedVec {
    rkyv::to_bytes::<_, 64>(x).expect("a word serialises")
}

/// The archived root of `bytes`, which [`archived`] wrote: the word itself, as `Word`'s
/// `ArchivedDBData<Repr = Self>` bound says.
fn root<R: Word>(bytes: &AlignedVec) -> &R {
    // SAFETY: `bytes` is what `rkyv::to_bytes` wrote for an `R`, whose archived form is the word
    // itself, and every bit pattern of the word's limbs is a value.
    unsafe { rkyv::archived_root::<R>(bytes) }
}

/// The archived form of a word is the word: its bytes are the wire body behind the version byte
/// (on this little-endian host, where the in-memory bytes are the wire's), it deserialises back
/// to the value, and `OrdRepr`'s contract holds, the archived form of `a` against `b` being
/// `a.cmp(&b)` and two archived forms ordering as their values do, on 2 × 10⁴ values over the
/// word's window, each against itself, its neighbour and a random partner. The three orderings
/// are each seen at least a thousand times, so the check is not vacuous.
fn archived_form_is_the_word_and_orders_as_ord<R: Word>() {
    let values = mixed::<R>(20_000);
    let bytes: Vec<AlignedVec> = values.iter().map(archived).collect();
    let mut rng = Rng(7);
    let mut seen = [0usize; 3];
    for (i, a) in values.iter().enumerate() {
        assert_eq!(
            bytes[i].as_slice(),
            a.body().as_slice(),
            "{}: the archive is the wire body",
            R::NAME
        );
        let arch = root::<R>(&bytes[i]);
        assert_eq!(arch, a);
        let back: R = arch.deserialize(&mut Infallible).expect("infallible");
        assert_eq!(back, *a);
        for j in [
            i,
            (i + 1) % values.len(),
            rng.next() as usize % values.len(),
        ] {
            let b = &values[j];
            let expected = a.cmp(b);
            seen[(expected as i8 + 1) as usize] += 1;
            assert_eq!(arch.ord_cmp(b), expected, "{a:?} vs {b:?}");
            assert_eq!(arch.cmp(root::<R>(&bytes[j])), expected, "{a:?} vs {b:?}");
        }
    }
    assert!(
        seen.iter().all(|&n| n >= 1_000),
        "{}: orderings seen (less, equal, greater): {seen:?}",
        R::NAME
    );
}

#[test]
fn dbsp_wide_archived_form_is_the_word_and_orders_as_ord() {
    archived_form_is_the_word_and_orders_as_ord::<Wide>();
}

#[test]
fn dbsp_fixed_archived_form_is_the_word_and_orders_as_ord() {
    archived_form_is_the_word_and_orders_as_ord::<Fixed256>();
}

/// `MulByRef<i64>` and `MulByRef<isize>`, what `aggregate_linear` scales a row's contribution
/// by, are the group's `Mul` by the integer, and `HasZero` is the group's zero.
fn mul_by_ref_and_zero_are_the_group_s<R: Word>() {
    let mut rng = Rng(69);
    let zero = <R as HasZero>::zero();
    assert!(HasZero::is_zero(&zero));
    assert_eq!(zero + zero, zero);
    let mut nonzero = 0;
    for _ in 0..64 {
        let x = admit::<R>(R::draw(&mut rng));
        for k in [-3i64, -1, 0, 1, 2, 1_000] {
            assert_eq!(x.mul_by_ref(&k), x * k, "{}: {x:?} × {k}", R::NAME);
            assert_eq!(
                x.mul_by_ref(&(k as isize)),
                x * k,
                "{}: {x:?} × {k}",
                R::NAME
            );
        }
        assert_eq!(HasZero::is_zero(&x), x == zero);
        assert!(HasZero::is_zero(&(x + x.mul_by_ref(&-1i64))));
        nonzero += usize::from(!HasZero::is_zero(&x));
    }
    assert!(nonzero >= 60, "{}: {nonzero} of 64 draws nonzero", R::NAME);
}

#[test]
fn dbsp_wide_mul_by_ref_and_zero_are_the_group_s() {
    mul_by_ref_and_zero_are_the_group_s::<Wide>();
}

#[test]
fn dbsp_fixed_mul_by_ref_and_zero_are_the_group_s() {
    mul_by_ref_and_zero_are_the_group_s::<Fixed256>();
}

/// The orderings the archived check counts are `Ordering`'s three, and the index arithmetic
/// above maps them to `0..3`.
#[test]
fn dbsp_wide_ordering_index_maps_the_three_orderings() {
    assert_eq!(
        [Ordering::Less, Ordering::Equal, Ordering::Greater].map(|o| (o as i8 + 1) as usize),
        [0, 1, 2]
    );
}
