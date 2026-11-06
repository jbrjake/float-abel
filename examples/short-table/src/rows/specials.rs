//! The row "retractable NaN and ∞".

use crate::column::{build, num, recipe, Column, Step};
use crate::guard::probe;
use crate::render::{wrong, Cell};
use crate::rows::Row;

/// What a history reads before its specials are retracted.
#[derive(Clone, Copy)]
enum Reads {
    Nan,
    Inf,
}

/// One history: the steps up to a reading, what that reading must be, and the steps that take
/// the specials back out, after which the finite 1.5 must be all that is left.
struct History {
    first: &'static [Step],
    then: Option<Reads>,
    rest: &'static [Step],
}

const FINITE: f64 = 1.5;

/// A retraction here is the column's own retraction of that same value.
const HISTORIES: [History; 4] = [
    History {
        first: &[Step::Insert(FINITE), Step::Insert(f64::NAN)],
        then: Some(Reads::Nan),
        rest: &[Step::Retract(f64::NAN)],
    },
    History {
        first: &[Step::Insert(FINITE), Step::Insert(f64::INFINITY)],
        then: Some(Reads::Inf),
        rest: &[Step::Retract(f64::INFINITY)],
    },
    History {
        first: &[
            Step::Insert(FINITE),
            Step::Insert(f64::INFINITY),
            Step::Insert(f64::NEG_INFINITY),
        ],
        then: Some(Reads::Nan),
        rest: &[
            Step::Retract(f64::INFINITY),
            Step::Retract(f64::NEG_INFINITY),
        ],
    },
    // A negative NaN count with no NaN yet: a state a group holds and IEEE cannot.
    History {
        first: &[
            Step::Retract(f64::NAN),
            Step::Insert(f64::NAN),
            Step::Insert(FINITE),
        ],
        then: None,
        rest: &[],
    },
];

/// `f64` includes NaN and ±∞, and a `SUM` over a changing column has to recover when the NaN row
/// is deleted.
pub(crate) struct Specials;

/// Runs the steps on a state, through the column's own insert and retract.
fn apply<C: Column>(s: &mut C::State, steps: &[Step]) -> Result<(), crate::column::Shortfall> {
    for step in steps {
        match *step {
            Step::Insert(x) => C::insert(s, x)?,
            Step::Retract(x) => C::retract(s, x)?,
        }
    }
    Ok(())
}

impl Row for Specials {
    const NAME: &'static str = "retractable NaN and ∞";

    fn intro() -> String {
        let told: Vec<String> = HISTORIES
            .iter()
            .map(|h| match h.then {
                Some(Reads::Nan) => format!(
                    "{} reads NaN, then {} reads {}",
                    recipe(h.first),
                    recipe(h.rest),
                    num(FINITE)
                ),
                Some(Reads::Inf) => format!(
                    "{} reads ∞, then {} reads {}",
                    recipe(h.first),
                    recipe(h.rest),
                    num(FINITE)
                ),
                None => format!("{} reads {}", recipe(h.first), num(FINITE)),
            })
            .collect();
        format!(
            "f64 includes NaN and ±∞, and a `SUM` over a changing column has to recover when the \
             NaN row is deleted. Four histories, where a retraction is the column's own \
             retraction of that same value: {}. The last starts with a NaN retracted before any \
             arrived, a state a group holds and IEEE cannot.",
            told.join("; ")
        )
    }

    fn cell<C: Column>() -> Cell {
        let probes: Vec<_> = HISTORIES
            .iter()
            .map(|h| {
                probe(|| {
                    let mut s = build::<C>(h.first)?;
                    let first = recipe(h.first);
                    let mid = C::read(&mut s)?;
                    let seen = match h.then {
                        Some(Reads::Nan) if !mid.is_nan() => {
                            return wrong(format!("{first} reads {}, want NaN", num(mid)));
                        }
                        Some(Reads::Inf) if mid.to_bits() != f64::INFINITY.to_bits() => {
                            return wrong(format!("{first} reads {}, want ∞", num(mid)));
                        }
                        Some(_) => format!("{first} reads {}, then {}", num(mid), recipe(h.rest)),
                        None => first,
                    };
                    apply::<C>(&mut s, h.rest)?;
                    let got = C::read(&mut s)?;
                    if got.to_bits() == FINITE.to_bits() {
                        Ok(format!("{seen} reads {}", num(got)))
                    } else {
                        wrong(format!("{seen} reads {}, want {}", num(got), num(FINITE)))
                    }
                })
            })
            .collect();
        Cell::from_probes(&probes, |readings| readings.join("; "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::column::Outcome;
    use crate::columns::control::Control;
    use crate::columns::mutant::{Mutant, Mutated, Mutation};
    use crate::columns::FloatAbel;

    /// `Exact` with a sticky NaN flag that retraction never clears.
    struct StickyNan;

    impl Mutation for StickyNan {
        const NAME: &'static str = "sticky NaN";

        fn insert(s: &mut Mutated, x: f64) -> Outcome<()> {
            s.inserts += 1;
            s.flag |= x.is_nan();
            FloatAbel::insert(&mut s.x, x)
        }

        fn read(s: &mut Mutated) -> Outcome<f64> {
            if s.flag {
                Ok(f64::NAN)
            } else {
                FloatAbel::read(&mut s.x)
            }
        }
    }

    #[test]
    fn float_abel_passes_and_the_mutant_and_the_control_fail() {
        let fa = Specials::cell::<FloatAbel>();
        assert_eq!(
            fa.evidence,
            "ran: +1.5 +NaN reads NaN, then −NaN reads 1.5; +1.5 +∞ reads ∞, then −∞ reads 1.5; \
             +1.5 +∞ +(−∞) reads NaN, then −∞ −(−∞) reads 1.5; −NaN +NaN +1.5 reads 1.5"
        );
        assert!(fa.pass);
        let mutant = Specials::cell::<Mutant<StickyNan>>();
        assert!(!mutant.pass);
        assert!(
            mutant
                .evidence
                .starts_with("ran: +1.5 +NaN reads NaN, then −NaN reads NaN, want 1.5"),
            "{}",
            mutant.evidence
        );
        // Only the NaN histories fail it: the two with ∞ alone pass, and a failing cell lists
        // its shortfalls alone.
        assert_eq!(
            mutant.evidence.matches("ran:").count(),
            2,
            "{}",
            mutant.evidence
        );
        assert!(!mutant.evidence.contains('∞'), "{}", mutant.evidence);
        let control = Specials::cell::<Control>();
        assert!(!control.pass);
        assert!(
            control
                .evidence
                .contains("+1.5 +∞ reads ∞, then −∞ reads NaN, want 1.5"),
            "{}",
            control.evidence
        );
    }
}
