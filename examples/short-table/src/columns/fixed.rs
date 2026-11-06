//! fixed: `I64F64`, a 128-bit fixed-point number with 64 fractional bits.

use fixed::types::I64F64;

use crate::column::{hashes_equal, num, Column, Outcome, Shortfall, View};

/// `I64F64` through its checked API, so a value outside the grid or the range is a refusal and
/// never a panic: `checked_from_num` on entry, `checked_add` and `checked_sub` to accumulate.
pub(crate) struct Fixed;

/// The fixed-point value for `x`, or the crate's refusal.
fn enter(x: f64) -> Outcome<I64F64> {
    I64F64::checked_from_num(x)
        .ok_or_else(|| Shortfall::Refused(format!("I64F64::checked_from_num({}) is None", num(x))))
}

impl Column for Fixed {
    const NAME: &'static str = "fixed";
    type State = I64F64;

    fn empty() -> I64F64 {
        I64F64::ZERO
    }

    fn insert(s: &mut I64F64, x: f64) -> Outcome<()> {
        let v = enter(x)?;
        *s = s
            .checked_add(v)
            .ok_or_else(|| Shortfall::Refused(format!("checked_add of {} overflows", num(x))))?;
        Ok(())
    }

    fn retract(s: &mut I64F64, x: f64) -> Outcome<()> {
        let v = enter(x)?;
        *s = s
            .checked_sub(v)
            .ok_or_else(|| Shortfall::Refused(format!("checked_sub of {} overflows", num(x))))?;
        Ok(())
    }

    fn read(s: &mut I64F64) -> Outcome<f64> {
        Ok(s.to_num::<f64>())
    }

    fn view(s: &I64F64) -> Outcome<View> {
        Ok(View::Bytes(s.to_bits().to_le_bytes().to_vec()))
    }

    fn batch(_: &[f64]) -> Option<Outcome<f64>> {
        None
    }

    fn merge(a: I64F64, b: I64F64) -> Outcome<I64F64> {
        a.checked_add(b)
            .ok_or_else(|| Shortfall::Refused("checked_add of the two states overflows".to_owned()))
    }

    fn negate(s: I64F64) -> Outcome<I64F64> {
        s.checked_neg()
            .ok_or_else(|| Shortfall::Refused("checked_neg overflows".to_owned()))
    }

    fn eq(a: &I64F64, b: &I64F64) -> Outcome<bool> {
        Ok(a == b)
    }

    fn hash_eq(a: &I64F64, b: &I64F64) -> Outcome<bool> {
        Ok(hashes_equal(a, b))
    }

    fn is_zero(s: &I64F64) -> Outcome<bool> {
        Ok(s.is_zero())
    }
}
