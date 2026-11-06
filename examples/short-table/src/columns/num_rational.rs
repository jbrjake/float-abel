//! num-rational: `BigRational`, an exact fraction of big integers.

use num_rational::BigRational;
use num_traits::{ToPrimitive, Zero};

use crate::column::{num, Column, Outcome, Shortfall, View};

/// `BigRational` entered through `from_float`, which is exact for every finite double.
pub(crate) struct NumRational;

/// The fraction equal to `x`, or the crate's refusal.
fn enter(x: f64) -> Outcome<BigRational> {
    BigRational::from_float(x)
        .ok_or_else(|| Shortfall::Refused(format!("BigRational::from_float({}) is None", num(x))))
}

impl Column for NumRational {
    const NAME: &'static str = "num-rational";
    type State = BigRational;

    fn empty() -> BigRational {
        BigRational::zero()
    }

    fn insert(s: &mut BigRational, x: f64) -> Outcome<()> {
        *s += enter(x)?;
        Ok(())
    }

    fn retract(s: &mut BigRational, x: f64) -> Outcome<()> {
        *s -= enter(x)?;
        Ok(())
    }

    fn read(s: &mut BigRational) -> Outcome<f64> {
        s.to_f64()
            .ok_or_else(|| Shortfall::Refused("BigRational::to_f64 is None".to_owned()))
    }

    /// The numerator and denominator the crate keeps, which it holds reduced.
    fn view(s: &BigRational) -> Outcome<View> {
        Ok(View::Debug(format!("{}/{}", s.numer(), s.denom())))
    }

    fn batch(_: &[f64]) -> Option<Outcome<f64>> {
        None
    }

    fn merge(a: BigRational, b: BigRational) -> Outcome<BigRational> {
        Ok(a + b)
    }

    fn negate(s: BigRational) -> Outcome<BigRational> {
        Ok(-s)
    }
}
