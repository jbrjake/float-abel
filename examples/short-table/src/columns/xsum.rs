//! xsum: Radford Neal's exact summation, the small accumulator.

use std::hash::Hash;
use std::ops::{Add, AddAssign, Neg};

use impls::impls;
use xsum::{Xsum as _, XsumExt as _, XsumSmall};

use crate::column::{Column, Outcome, Shortfall, View};

// What the crate asserts absent, checked by the compiler: a release that adds one of these
// traits fails this build rather than leaving a stale no.
const _: () = assert!(!impls!(XsumSmall: Add<XsumSmall>));
const _: () = assert!(!impls!(XsumSmall: AddAssign<XsumSmall>));
const _: () = assert!(!impls!(XsumSmall: Neg));
const _: () = assert!(!impls!(XsumSmall: PartialEq));
const _: () = assert!(!impls!(XsumSmall: Hash));
const _: () = assert!(!impls!(XsumSmall: Clone));

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

    fn merge(_: XsumSmall, _: XsumSmall) -> Outcome<XsumSmall> {
        Err(Shortfall::Absent(
            "no merge: the Xsum trait is new, add_list, add, sum and clear, and XsumSmall has no \
             Add<XsumSmall> or AddAssign<XsumSmall>"
                .to_owned(),
        ))
    }

    fn negate(_: XsumSmall) -> Outcome<XsumSmall> {
        Err(Shortfall::Absent("no Neg on XsumSmall".to_owned()))
    }

    fn eq(_: &XsumSmall, _: &XsumSmall) -> Outcome<bool> {
        Err(Shortfall::Absent("no PartialEq on XsumSmall".to_owned()))
    }

    fn hash_eq(_: &XsumSmall, _: &XsumSmall) -> Outcome<bool> {
        Err(Shortfall::Absent("no Hash on XsumSmall".to_owned()))
    }

    /// No `PartialEq` to compare with the empty state, no `Clone`, and `sum` wants `&mut self`,
    /// so nothing an `is_zero(&self)` could ask.
    fn is_zero(_: &XsumSmall) -> Outcome<bool> {
        Err(Shortfall::Absent(
            "no PartialEq or Clone on XsumSmall, and sum takes &mut self".to_owned(),
        ))
    }
}
