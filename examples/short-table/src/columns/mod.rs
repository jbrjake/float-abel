//! One file per column: the eight crates the table compares, the `f64` control, and the mutants
//! the tests run.

pub(crate) mod accurate;
pub(crate) mod bigdecimal;
pub(crate) mod bitrep;
pub(crate) mod control;
pub(crate) mod fixed;
pub(crate) mod float_abel;
pub(crate) mod fsum;
#[cfg(test)]
pub(crate) mod mutant;
pub(crate) mod num_rational;
pub(crate) mod xsum;

pub(crate) use self::accurate::Accurate;
pub(crate) use self::bigdecimal::BigDec;
pub(crate) use self::bitrep::Bitrep;
pub(crate) use self::fixed::Fixed;
pub(crate) use self::float_abel::FloatAbel;
pub(crate) use self::fsum::Fsum;
pub(crate) use self::num_rational::NumRational;
pub(crate) use self::xsum::Xsum;

use crate::column::Column;

/// The eight columns' names, in table order.
pub(crate) const NAMES: [&str; 8] = [
    FloatAbel::NAME,
    Bitrep::NAME,
    Xsum::NAME,
    Fsum::NAME,
    Accurate::NAME,
    NumRational::NAME,
    BigDec::NAME,
    Fixed::NAME,
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::column::{build, Shortfall, Step, View};
    use crate::columns::control::Control;
    use crate::guard::probe;

    /// Every adapter's one-value path: `+0.5 +0.25` reads 0.75, and the same recipe built twice
    /// views equal.
    fn round_trip<C: Column>() {
        let recipe = [Step::Insert(0.5), Step::Insert(0.25)];
        let out = probe(|| {
            let mut s = build::<C>(&recipe)?;
            C::read(&mut s)
        });
        assert_eq!(out, Ok(0.75), "{}", C::NAME);
        let a = probe(|| C::view(&build::<C>(&recipe)?));
        let b = probe(|| C::view(&build::<C>(&recipe)?));
        assert!(a.is_ok(), "{}: {a:?}", C::NAME);
        assert_eq!(a, b, "{}", C::NAME);
        if let Some(batch) = C::batch(&[0.5, 0.25]) {
            assert_eq!(batch, Ok(0.75), "{} batch", C::NAME);
        }
    }

    #[test]
    fn every_column_reads_a_small_sum_and_views_one_recipe_the_same_twice() {
        round_trip::<FloatAbel>();
        round_trip::<Bitrep>();
        round_trip::<Xsum>();
        round_trip::<Fsum>();
        round_trip::<Accurate>();
        round_trip::<NumRational>();
        round_trip::<BigDec>();
        round_trip::<Fixed>();
        round_trip::<Control>();
        round_trip::<mutant::Mutant<mutant::Honest>>();
    }

    #[test]
    fn the_guard_turns_accurates_zero_state_read_into_panicked_at_its_line() {
        let out = probe(|| {
            let mut s = build::<Accurate>(&[Step::Insert(1.0), Step::Retract(1.0)])?;
            Accurate::read(&mut s)
        });
        assert_eq!(
            out,
            Err(Shortfall::Panicked {
                message: "assertion failed: count < xs.len()".to_owned(),
                at: "accurate-0.4.1/src/sum/ifastsum.rs:89".to_owned(),
            })
        );
    }

    #[test]
    fn a_refusal_names_the_value_and_stops_the_recipe() {
        let out = probe(|| {
            let mut s =
                build::<Fixed>(&[Step::Insert(1.0), Step::Insert(1e30), Step::Insert(1.0)])?;
            Fixed::read(&mut s)
        });
        assert_eq!(
            out,
            Err(Shortfall::Refused(
                "I64F64::checked_from_num(1e30) is None".to_owned()
            ))
        );
        let out = probe(|| build::<NumRational>(&[Step::Insert(f64::NAN)]).map(|_| ()));
        assert_eq!(
            out,
            Err(Shortfall::Refused(
                "BigRational::from_float(NaN) is None".to_owned()
            ))
        );
    }

    #[test]
    fn views_are_bytes_where_the_crate_has_them_and_debug_elsewhere() {
        let one = [Step::Insert(1.0)];
        assert!(matches!(
            FloatAbel::view(&build::<FloatAbel>(&one).unwrap()),
            Ok(View::Bytes(_))
        ));
        assert!(matches!(
            Bitrep::view(&build::<Bitrep>(&one).unwrap()),
            Ok(View::Bytes(_))
        ));
        assert!(matches!(
            Fixed::view(&build::<Fixed>(&one).unwrap()),
            Ok(View::Bytes(_))
        ));
        assert!(matches!(
            Control::view(&build::<Control>(&one).unwrap()),
            Ok(View::Bytes(_))
        ));
        assert!(matches!(
            Xsum::view(&build::<Xsum>(&one).unwrap()),
            Ok(View::Debug(_))
        ));
        assert_eq!(
            Fsum::view(&build::<Fsum>(&one).unwrap()),
            Ok(View::Debug("FSum { partials: [1.0] }".to_owned()))
        );
        assert!(matches!(
            Accurate::view(&build::<Accurate>(&one).unwrap()),
            Ok(View::Debug(_))
        ));
        assert_eq!(
            NumRational::view(&build::<NumRational>(&one).unwrap()),
            Ok(View::Debug("1/1".to_owned()))
        );
        assert_eq!(
            BigDec::view(&build::<BigDec>(&one).unwrap()),
            Ok(View::Debug("digits 1 at scale 0".to_owned()))
        );
    }
}
