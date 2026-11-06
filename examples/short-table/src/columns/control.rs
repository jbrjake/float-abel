//! Plain `f64`: the control, run through every probe outside the table so a reader sees what each
//! probe does to the type the engines use today.

use crate::column::{Column, Outcome, View};

/// `f64` with `+=` and `-=`.
pub(crate) struct Control;

impl Column for Control {
    const NAME: &'static str = "f64";
    type State = f64;

    fn empty() -> f64 {
        0.0
    }

    fn insert(s: &mut f64, x: f64) -> Outcome<()> {
        *s += x;
        Ok(())
    }

    fn retract(s: &mut f64, x: f64) -> Outcome<()> {
        *s -= x;
        Ok(())
    }

    fn read(s: &mut f64) -> Outcome<f64> {
        Ok(*s)
    }

    fn view(s: &f64) -> Outcome<View> {
        Ok(View::Bytes(s.to_bits().to_le_bytes().to_vec()))
    }

    fn batch(xs: &[f64]) -> Option<Outcome<f64>> {
        Some(Ok(xs.iter().sum()))
    }
}
