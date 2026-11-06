//! fsum: Shewchuk's partials, as Python's `math.fsum` keeps them.

use std::ops::{Add, AddAssign, Neg};

use fsum::FSum;
use impls::impls;

use crate::column::{Column, Outcome, Shortfall, View};

// What the crate asserts absent, checked by the compiler.
const _: () = assert!(!impls!(FSum: Add<FSum>));
const _: () = assert!(!impls!(FSum: AddAssign<FSum>));
const _: () = assert!(!impls!(FSum: Neg));

/// `FSum`, through its `+=` and `-=` on `f64` and `with_all` for a slice.
pub(crate) struct Fsum;

impl Column for Fsum {
    const NAME: &'static str = "fsum";
    type State = FSum;

    fn empty() -> FSum {
        FSum::new()
    }

    fn insert(s: &mut FSum, x: f64) -> Outcome<()> {
        *s += x;
        Ok(())
    }

    fn retract(s: &mut FSum, x: f64) -> Outcome<()> {
        *s -= x;
        Ok(())
    }

    fn read(s: &mut FSum) -> Outcome<f64> {
        Ok(s.value())
    }

    /// `Debug` is the one window on the partials.
    fn view(s: &FSum) -> Outcome<View> {
        Ok(View::Debug(format!("{s:?}")))
    }

    fn batch(xs: &[f64]) -> Option<Outcome<f64>> {
        Some(Ok(FSum::with_all(xs).value()))
    }

    fn merge(_: FSum, _: FSum) -> Outcome<FSum> {
        Err(Shortfall::Absent(
            "no Add<FSum> or AddAssign<FSum>: FSum takes f64 alone".to_owned(),
        ))
    }

    fn negate(_: FSum) -> Outcome<FSum> {
        Err(Shortfall::Absent("no Neg on FSum".to_owned()))
    }
}
