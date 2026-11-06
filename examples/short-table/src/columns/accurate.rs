//! accurate: `OnlineExactSum`, Zhu and Hayes's exponent-indexed accumulators.

use std::hash::Hash;
use std::ops::{Mul, Neg};

use accurate::sum::OnlineExactSum;
use accurate::traits::SumAccumulator;
use impls::impls;

use crate::column::{is_zero, Column, Outcome, Shortfall, View};

// What the crate asserts absent, checked by the compiler.
const _: () = assert!(!impls!(OnlineExactSum<f64>: Neg));
const _: () = assert!(!impls!(OnlineExactSum<f64>: PartialEq));
const _: () = assert!(!impls!(OnlineExactSum<f64>: Mul<i64>));
const _: () = assert!(!impls!(OnlineExactSum<f64>: Mul<OnlineExactSum<f64>>));
const _: () = assert!(!impls!(OnlineExactSum<f64>: Hash));

/// `OnlineExactSum<f64>` through `+=`, and `absorb` for a slice.
pub(crate) struct Accurate;

impl Column for Accurate {
    const NAME: &'static str = "accurate";
    type State = OnlineExactSum<f64>;

    fn empty() -> OnlineExactSum<f64> {
        OnlineExactSum::zero()
    }

    fn insert(s: &mut OnlineExactSum<f64>, x: f64) -> Outcome<()> {
        *s += x;
        Ok(())
    }

    /// There is no inverse; adding the negation is the nearest the crate offers.
    fn retract(s: &mut OnlineExactSum<f64>, x: f64) -> Outcome<()> {
        *s += -x;
        Ok(())
    }

    /// `sum` consumes the accumulator, so the read is of a clone.
    fn read(s: &mut OnlineExactSum<f64>) -> Outcome<f64> {
        Ok(s.clone().sum())
    }

    /// `Debug` is the one window on the state.
    fn view(s: &OnlineExactSum<f64>) -> Outcome<View> {
        Ok(View::Debug(format!("{s:?}")))
    }

    fn batch(xs: &[f64]) -> Option<Outcome<f64>> {
        Some(Ok(OnlineExactSum::<f64>::zero()
            .absorb(xs.iter().copied())
            .sum()))
    }

    fn merge(a: OnlineExactSum<f64>, b: OnlineExactSum<f64>) -> Outcome<OnlineExactSum<f64>> {
        Ok(a + b)
    }

    fn negate(_: OnlineExactSum<f64>) -> Outcome<OnlineExactSum<f64>> {
        Err(Shortfall::Absent(
            "no Neg on OnlineExactSum<f64>".to_owned(),
        ))
    }

    fn eq(_: &OnlineExactSum<f64>, _: &OnlineExactSum<f64>) -> Outcome<bool> {
        Err(Shortfall::Absent(
            "no PartialEq on OnlineExactSum<f64>".to_owned(),
        ))
    }

    fn hash_eq(_: &OnlineExactSum<f64>, _: &OnlineExactSum<f64>) -> Outcome<bool> {
        Err(Shortfall::Absent(
            "no Hash on OnlineExactSum<f64>".to_owned(),
        ))
    }

    /// No `PartialEq`; the one question the state answers is whether a clone's `sum` reads zero.
    fn is_zero(s: &OnlineExactSum<f64>) -> Outcome<bool> {
        Ok(is_zero(s.clone().sum()))
    }

    fn scale(_: OnlineExactSum<f64>, _: i64) -> Outcome<OnlineExactSum<f64>> {
        Err(Shortfall::Absent(
            "no Mul<i64> on OnlineExactSum<f64>".to_owned(),
        ))
    }

    fn mul(_: OnlineExactSum<f64>, _: OnlineExactSum<f64>) -> Outcome<OnlineExactSum<f64>> {
        Err(Shortfall::Absent(
            "no Mul<OnlineExactSum<f64>> on OnlineExactSum<f64>".to_owned(),
        ))
    }
}
