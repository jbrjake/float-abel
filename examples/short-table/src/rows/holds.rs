//! The row "holds every f64 exactly".

use crate::column::{build, num, recipe, Column, Step};
use crate::guard::probe;
use crate::render::{wrong, Cell};
use crate::rows::Row;

/// The seven edge doubles.
pub(crate) const VALUES: [f64; 7] = [0.1, 1e-20, 1e30, 1e300, f64::MAX, f64::MIN_POSITIVE, 5e-324];

/// The double just below `x`.
fn below(x: f64) -> f64 {
    f64::from_bits(x.to_bits() - 1)
}

/// The weight carries the column's value, so a lossy carrier makes the `SUM` wrong.
pub(crate) struct Holds;

impl Row for Holds {
    const NAME: &'static str = "holds every f64 exactly";

    fn intro() -> String {
        let named: Vec<String> = VALUES.iter().map(|&x| num(x)).collect();
        format!(
            "The weight carries the column's value, so a carrier that loses any of it makes the \
             `SUM` wrong. Seven edge doubles, {}: each is read back alone, and each is entered \
             with the double just below it retracted, which must leave exactly their gap, one \
             ulp and itself a double. The second check catches a carrier that rounds on entry \
             and rounds back the same way on read, which the first alone cannot see. Every \
             comparison is of bits. Zero stays out, because the group's zero is +0.0 and the sign \
             of zero is not a value; NaN and ±∞ have their own row.",
            named.join(", ")
        )
    }

    fn cell<C: Column>() -> Cell {
        let mut probes = Vec::new();
        for x in VALUES {
            probes.push(probe(|| {
                let mut s = build::<C>(&[Step::Insert(x)])?;
                let got = C::read(&mut s)?;
                if got.to_bits() == x.to_bits() {
                    Ok(num(x))
                } else {
                    wrong(format!("+{} reads {}", num(x), num(got)))
                }
            }));
            let b = below(x);
            let gap = x - b;
            let steps = [Step::Insert(x), Step::Retract(b)];
            probes.push(probe(|| {
                let mut s = build::<C>(&steps)?;
                let got = C::read(&mut s)?;
                if got.to_bits() == gap.to_bits() {
                    Ok(num(gap))
                } else {
                    wrong(format!(
                        "{} reads {}, want {}",
                        recipe(&steps),
                        num(got),
                        num(gap)
                    ))
                }
            }));
        }
        Cell::from_probes(&probes, |_| {
            "all 7 values and their 7 one-ulp gaps read back bit for bit".to_owned()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::column::Outcome;
    use crate::columns::control::Control;
    use crate::columns::mutant::{Mutant, Mutated, Mutation};
    use crate::columns::FloatAbel;

    /// `Exact` that rounds every input through `f32` on the way in.
    struct RoundsOnEntry;

    /// The loss is the mutation: 29 bits of significand and most of the exponent range.
    #[allow(clippy::cast_possible_truncation)]
    fn through_f32(x: f64) -> f64 {
        f64::from(x as f32)
    }

    impl Mutation for RoundsOnEntry {
        const NAME: &'static str = "rounds through f32 on entry";

        fn insert(s: &mut Mutated, x: f64) -> Outcome<()> {
            s.inserts += 1;
            FloatAbel::insert(&mut s.x, through_f32(x))
        }

        fn retract(s: &mut Mutated, x: f64) -> Outcome<()> {
            FloatAbel::retract(&mut s.x, through_f32(x))
        }
    }

    #[test]
    fn float_abel_and_the_control_pass_and_the_mutant_fails() {
        assert!(Holds::cell::<FloatAbel>().pass);
        assert!(Holds::cell::<Control>().pass);
        let mutant = Holds::cell::<Mutant<RoundsOnEntry>>();
        assert!(!mutant.pass);
        assert!(
            mutant
                .evidence
                .contains("ran: +0.1 reads 0.10000000149011612"),
            "{}",
            mutant.evidence
        );
    }

    #[test]
    fn the_gap_is_one_ulp_and_itself_a_double() {
        for x in VALUES {
            let gap = x - below(x);
            assert!(gap > 0.0, "{}", num(x));
            assert_eq!(below(x) + gap, x, "{}", num(x));
        }
        assert_eq!(below(5e-324), 0.0);
    }
}
