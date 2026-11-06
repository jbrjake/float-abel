//! The crate under test: `Exact`, the group.

use float_abel::Exact;

use crate::column::{Column, Outcome, View};

/// `Exact`, driven exactly as the differential and DBSP adapters drive it.
pub(crate) struct FloatAbel;

impl Column for FloatAbel {
    const NAME: &'static str = "float-abel";
    type State = Exact;

    fn empty() -> Exact {
        Exact::default()
    }

    fn insert(s: &mut Exact, x: f64) -> Outcome<()> {
        *s += Exact::from(x);
        Ok(())
    }

    fn retract(s: &mut Exact, x: f64) -> Outcome<()> {
        *s -= Exact::from(x);
        Ok(())
    }

    fn read(s: &mut Exact) -> Outcome<f64> {
        Ok(s.to_f64())
    }

    /// `encode`, not `to_bytes`: the canonical encoding of either form, where `to_bytes` is the
    /// compact form's alone and panics on the heap form.
    fn view(s: &Exact) -> Outcome<View> {
        Ok(View::Bytes(s.encode()))
    }

    fn batch(xs: &[f64]) -> Option<Outcome<f64>> {
        Some(Ok(xs
            .iter()
            .copied()
            .map(Exact::from)
            .sum::<Exact>()
            .to_f64()))
    }
}
