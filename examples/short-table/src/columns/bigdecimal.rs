//! bigdecimal: `BigDecimal`, a big integer of digits and a decimal scale.

use bigdecimal::{BigDecimal, FromPrimitive, ToPrimitive, Zero};

use crate::column::{num, Column, Outcome, Shortfall, View};

/// `BigDecimal` entered through `from_f64`.
pub(crate) struct BigDec;

/// The decimal equal to `x`, or the crate's refusal.
fn enter(x: f64) -> Outcome<BigDecimal> {
    BigDecimal::from_f64(x)
        .ok_or_else(|| Shortfall::Refused(format!("BigDecimal::from_f64({}) is None", num(x))))
}

impl Column for BigDec {
    const NAME: &'static str = "bigdecimal";
    type State = BigDecimal;

    fn empty() -> BigDecimal {
        BigDecimal::zero()
    }

    fn insert(s: &mut BigDecimal, x: f64) -> Outcome<()> {
        *s += enter(x)?;
        Ok(())
    }

    fn retract(s: &mut BigDecimal, x: f64) -> Outcome<()> {
        *s -= enter(x)?;
        Ok(())
    }

    fn read(s: &mut BigDecimal) -> Outcome<f64> {
        s.to_f64()
            .ok_or_else(|| Shortfall::Refused("BigDecimal::to_f64 is None".to_owned()))
    }

    /// The digits and the scale the crate keeps, which it does not normalise after arithmetic.
    fn view(s: &BigDecimal) -> Outcome<View> {
        let (digits, scale) = s.as_bigint_and_exponent();
        Ok(View::Debug(format!("digits {digits} at scale {scale}")))
    }

    fn batch(_: &[f64]) -> Option<Outcome<f64>> {
        None
    }
}
