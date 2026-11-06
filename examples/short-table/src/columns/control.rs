//! Plain `f64`: the control, run through every probe outside the table so a reader sees what each
//! probe does to the type the engines use today.

use std::hash::Hash;

use impls::impls;

use crate::column::{is_zero, Column, Outcome, Shortfall, View};

// What the type asserts absent, checked by the compiler.
const _: () = assert!(!impls!(f64: Hash));

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

    fn merge(a: f64, b: f64) -> Outcome<f64> {
        Ok(a + b)
    }

    fn negate(s: f64) -> Outcome<f64> {
        Ok(-s)
    }

    /// The type's own `==`, which is the comparison under test here.
    #[allow(clippy::float_cmp)]
    fn eq(a: &f64, b: &f64) -> Outcome<bool> {
        Ok(a == b)
    }

    fn hash_eq(_: &f64, _: &f64) -> Outcome<bool> {
        Err(Shortfall::Absent("no Hash on f64".to_owned()))
    }

    fn is_zero(s: &f64) -> Outcome<bool> {
        Ok(is_zero(*s))
    }
}
