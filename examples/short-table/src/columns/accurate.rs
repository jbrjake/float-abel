//! accurate: `OnlineExactSum`, Zhu and Hayes's exponent-indexed accumulators.

use accurate::sum::OnlineExactSum;
use accurate::traits::SumAccumulator;

use crate::column::{Column, Outcome, View};

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
}
