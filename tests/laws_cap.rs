//! The laws kit at the limb cap's edge. `tests/laws.rs`'s `exact_full()` never draws a value
//! within a bit of `EXACT_MAX_LIMBS`, so a closure defect at the edge is invisible to it: the law
//! is only as strong as its strategy. Here the strategy sits at the closure check's own budget
//! under the cap, where a case's fold climbs onto the cap's last bit and retracts across it and
//! the products land there too, and a control that refuses that bit fails the law on this
//! strategy and holds on every binade. Every test is named `laws_closure_exact_at_the_cap_*` so
//! the task verify can filter on it.

use core::ops::{Add, Mul, Neg, Sub};

use float_abel::laws::proptest::prelude::*;
use float_abel::laws::proptest::test_runner::TestRunner;
use float_abel::laws::{self, Weight, WEIGHTS};
use float_abel::{Exact, EXACT_MAX_LIMBS};

use crate::exact_heap::two_to;
use crate::laws::{exact_full, failure, runner, small_int, unshrunk_for, SEED};
use crate::wire_heap::{header, limbs};

/// Cases per sweep at the edge, an eighth of `tests/laws.rs`'s: every operation here moves
/// `EXACT_MAX_LIMBS` limbs, so one case's thousand-shuffle fold costs what a whole check of the
/// compact word does, and the control below proves the edge is reached inside this many.
const CASES: u32 = 32;

fn edge_runner() -> TestRunner {
    laws::runner(CASES, SEED)
}

/// Bits the heap form's finite part may span, as `src/exact/big.rs` counts them.
const CAP_BITS: i32 = 64 * EXACT_MAX_LIMBS as i32;

/// The bits the closure check needs under the cap: it folds up to `WEIGHTS` weights in a
/// thousand orders, and thirty-two integers under `2^k` sum under `2^(k + 5)` in every order and
/// every partial, so a weight under `2^(CAP_BITS − 5)` keeps every sum a case forms inside the
/// cap, and a weight one bit wider does not. The multipliers take the same five bits.
const HEADROOM: i32 = WEIGHTS.trailing_zeros() as i32;
const _: () = assert!(WEIGHTS == 1 << HEADROOM);

/// The widest weight the edge strategy draws spans this many bits.
const EDGE_BITS: i32 = CAP_BITS - HEADROOM;

fn e(x: f64) -> Exact {
    Exact::from(x)
}

/// Integers at the check's budget: three shapes spanning `EDGE_BITS`, every bit set, every bit
/// above the first limb set with the bottom one, and the top two with the bottom one, and three
/// small ones, `1`, `3` and `2^64 + 1`. A draw is small one time in sixteen and negative one time
/// in sixteen, so a case's fold climbs onto the cap's last bit within twenty-odd draws and then
/// retracts across it, instead of random-walking under it, and the pairwise sums and differences
/// meet a limb boundary and a carry chain the whole width of the cap.
fn exact_at_the_cap() -> impl Strategy<Value = Exact> {
    let wide = vec![
        two_to(EDGE_BITS) - e(1.0),
        two_to(EDGE_BITS) - two_to(64) + e(1.0),
        two_to(EDGE_BITS - 1) + two_to(EDGE_BITS - 2) + e(1.0),
    ];
    let small = vec![e(1.0), e(3.0), two_to(64) + e(1.0)];
    let magnitude = prop_oneof![15 => prop::sample::select(wide), 1 => prop::sample::select(small)];
    let negative = prop_oneof![15 => Just(false), 1 => Just(true)];
    (magnitude, negative).prop_map(|(v, neg)| if neg { -v } else { v })
}

/// A multiplier inside the headroom, `|k| ≤ 2^HEADROOM`: a weight at the edge scaled by it stays
/// inside the cap, and the odd `2^HEADROOM − 1` lands the wide shapes on the cap's last bit.
fn edge_int() -> impl Strategy<Value = i64> {
    -(1i64 << HEADROOM)..=(1i64 << HEADROOM)
}

/// The same multipliers as `Exact`, for `Mul<Exact>` at the edge.
fn edge_exact() -> impl Strategy<Value = Exact> {
    edge_int().prop_map(|k| e(k as f64))
}

/// Whether the finite part spans exactly `CAP_BITS`: the heap form's finite run holds
/// `EXACT_MAX_LIMBS` limbs and the top bit of the last is set, the mantissa being odd.
fn on_the_last_bit(v: &Exact) -> bool {
    if v.is_compact() {
        return false;
    }
    let bytes = v.encode();
    let len = header(&bytes)[0].2;
    len == EXACT_MAX_LIMBS && limbs(&bytes)[len - 1] >> 63 == 1
}

/// `Exact` refusing a result on the cap's last bit, as a cap decided on the operands' spans
/// rather than the result would. The law is only as strong as its strategy, so this control must
/// fail on one that lands there and hold on one that does not.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct UnderCap(Exact);

impl UnderCap {
    fn under(value: Exact, what: &str) -> Self {
        assert!(
            !on_the_last_bit(&value),
            "UnderCap: {what} lands on the cap's last bit"
        );
        UnderCap(value)
    }
}

impl Add for UnderCap {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::under(self.0 + other.0, "a sum")
    }
}

impl Sub for UnderCap {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::under(self.0 - other.0, "a difference")
    }
}

impl Neg for UnderCap {
    type Output = Self;
    fn neg(self) -> Self {
        UnderCap(-self.0)
    }
}

impl Mul<i64> for UnderCap {
    type Output = Self;
    fn mul(self, k: i64) -> Self {
        Self::under(self.0 * k, "a product")
    }
}

impl Weight for UnderCap {
    fn zero() -> Self {
        UnderCap(Exact::ZERO)
    }
    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

/// The heap form is closed at the cap's edge: the pairwise sums and differences, the products by
/// an integer inside the headroom, and the fold in a thousand orders, which climbs onto the cap's
/// last bit and retracts across it, all return.
#[test]
fn laws_closure_exact_at_the_cap_holds_by_an_integer() {
    assert_eq!(
        laws::check_closure(&mut edge_runner(), exact_at_the_cap(), edge_int()),
        Ok(())
    );
}

/// The same with the multiplier an `Exact`, so `Mul<Exact>` lands on the cap's last bit too.
#[test]
fn laws_closure_exact_at_the_cap_holds_by_an_exact() {
    assert_eq!(
        laws::check_closure(&mut edge_runner(), exact_at_the_cap(), edge_exact()),
        Ok(())
    );
}

/// The group laws on the same strategy: the fold that lands on the cap lands the same bits in
/// every order, and every weight there has its inverse and retracts.
#[test]
fn laws_closure_exact_at_the_cap_holds_the_group_laws() {
    assert_eq!(
        laws::check_abelian_group(&mut edge_runner(), exact_at_the_cap()),
        Ok(())
    );
}

/// The strategy reaches the edge inside the sweep's own cases: the control fails the closure law
/// on it, by name, and the witness is a fold or a product, since two weights at the budget sum
/// four bits under the cap.
#[test]
fn laws_closure_exact_at_the_cap_control_fails_on_the_last_bit() {
    let (why, case) = failure(
        laws::check_closure(
            &mut unshrunk_for(CASES),
            exact_at_the_cap().prop_map(UnderCap),
            edge_int(),
        ),
        "the closure check",
    );
    assert!(why.contains("closure"), "{why}");
    assert!(why.contains("the cap's last bit"), "{why}");
    assert!(why.contains("the fold of") || why.contains(" · "), "{why}");
    assert!(!case.weights.is_empty(), "{case:?}");
}

/// And holds on every binade: `exact_full()` never lands on the cap's last bit, which is why the
/// edge needs a strategy of its own.
#[test]
fn laws_closure_exact_at_the_cap_control_holds_on_every_binade() {
    assert_eq!(
        laws::check_closure(&mut runner(), exact_full().prop_map(UnderCap), small_int()),
        Ok(())
    );
}
