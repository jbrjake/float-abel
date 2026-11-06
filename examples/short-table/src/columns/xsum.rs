//! xsum: Radford Neal's exact summation, the small accumulator.

use xsum::{Xsum as _, XsumExt as _, XsumSmall};

use crate::column::{Column, Outcome, View};

/// `XsumSmall` one value at a time, and `[f64]::xsum()` for a slice, the entry the crate
/// recommends; its whole `Xsum` trait is `new`, `add_list`, `add`, `sum` and `clear`.
pub(crate) struct Xsum;

impl Column for Xsum {
    const NAME: &'static str = "xsum";
    type State = XsumSmall;

    fn empty() -> XsumSmall {
        XsumSmall::new()
    }

    fn insert(s: &mut XsumSmall, x: f64) -> Outcome<()> {
        s.add(x);
        Ok(())
    }

    /// There is no inverse; adding the negation is the nearest the crate offers.
    fn retract(s: &mut XsumSmall, x: f64) -> Outcome<()> {
        s.add(-x);
        Ok(())
    }

    fn read(s: &mut XsumSmall) -> Outcome<f64> {
        Ok(s.sum())
    }

    /// `Debug` is the one window on the state, and the crate derives it only under
    /// `cfg(debug_assertions)`, which this package turns on in every profile.
    fn view(s: &XsumSmall) -> Outcome<View> {
        Ok(View::Debug(format!("{s:?}")))
    }

    fn batch(xs: &[f64]) -> Option<Outcome<f64>> {
        Some(Ok(xs.xsum()))
    }
}
