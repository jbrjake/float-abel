//! bitrep: `SumF64`, a fixed-point limb accumulator with a count and sticky special flags.

use std::ops::Neg;

use bitrep::SumF64;
use impls::impls;

use crate::column::{num, Column, Outcome, Shortfall, View};

// What the crate asserts absent, checked by the compiler.
const _: () = assert!(!impls!(SumF64: Neg));

/// `SumF64` through `add`, `try_unmerge` for the inverse, `merge`, `to_bytes` and
/// `FromIterator<f64>`.
pub(crate) struct Bitrep;

impl Column for Bitrep {
    const NAME: &'static str = "bitrep";
    type State = SumF64;

    fn empty() -> SumF64 {
        SumF64::new()
    }

    fn insert(s: &mut SumF64, x: f64) -> Outcome<()> {
        s.add(x);
        Ok(())
    }

    /// The crate's inverse is `try_unmerge` of a state holding the value; it returns `false`,
    /// and leaves the state alone, when that state carries a special or a larger count.
    fn retract(s: &mut SumF64, x: f64) -> Outcome<()> {
        let mut one = SumF64::new();
        one.add(x);
        if s.try_unmerge(&one) {
            Ok(())
        } else {
            Err(Shortfall::Refused(format!(
                "try_unmerge of {} returned false",
                num(x)
            )))
        }
    }

    fn read(s: &mut SumF64) -> Outcome<f64> {
        Ok(s.value())
    }

    fn view(s: &SumF64) -> Outcome<View> {
        Ok(View::Bytes(s.to_bytes().to_vec()))
    }

    fn batch(xs: &[f64]) -> Option<Outcome<f64>> {
        Some(Ok(xs.iter().copied().collect::<SumF64>().value()))
    }

    fn merge(mut a: SumF64, b: SumF64) -> Outcome<SumF64> {
        a.merge(&b);
        Ok(a)
    }

    /// No `Neg`; the nearest operation is `try_unmerge` of the state from an empty accumulator,
    /// which refuses because the state's count exceeds the empty one's.
    fn negate(s: SumF64) -> Outcome<SumF64> {
        let mut empty = SumF64::new();
        if empty.try_unmerge(&s) {
            Ok(empty)
        } else {
            Err(Shortfall::Refused(
                "try_unmerge into an empty SumF64 returned false, and SumF64 has no Neg".to_owned(),
            ))
        }
    }
}
