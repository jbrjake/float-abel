//! fsum: Shewchuk's partials, as Python's `math.fsum` keeps them.

use std::hash::Hash;
use std::ops::{Add, AddAssign, Mul, Neg};

use fsum::FSum;
use impls::impls;

use crate::column::{is_zero, Column, Outcome, Shortfall, View};

// What the crate asserts absent, checked by the compiler.
const _: () = assert!(!impls!(FSum: Add<FSum>));
const _: () = assert!(!impls!(FSum: AddAssign<FSum>));
const _: () = assert!(!impls!(FSum: Neg));
const _: () = assert!(!impls!(FSum: PartialEq));
const _: () = assert!(!impls!(FSum: Hash));
const _: () = assert!(!impls!(FSum: Mul<i64>));
const _: () = assert!(!impls!(FSum: Mul<FSum>));

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

    fn eq(_: &FSum, _: &FSum) -> Outcome<bool> {
        Err(Shortfall::Absent("no PartialEq on FSum".to_owned()))
    }

    fn hash_eq(_: &FSum, _: &FSum) -> Outcome<bool> {
        Err(Shortfall::Absent("no Hash on FSum".to_owned()))
    }

    /// No `PartialEq`; the one question the state answers is whether its value reads zero.
    fn is_zero(s: &FSum) -> Outcome<bool> {
        Ok(is_zero(s.value()))
    }

    fn scale(_: FSum, _: i64) -> Outcome<FSum> {
        Err(Shortfall::Absent("no Mul<i64> on FSum".to_owned()))
    }

    fn mul(_: FSum, _: FSum) -> Outcome<FSum> {
        Err(Shortfall::Absent("no Mul<FSum> on FSum".to_owned()))
    }
}
