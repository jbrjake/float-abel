//! bitrep: `SumF64`, a fixed-point limb accumulator with a count and sticky special flags.

use bitrep::SumF64;

use crate::column::{num, Column, Outcome, Shortfall, View};

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
}
