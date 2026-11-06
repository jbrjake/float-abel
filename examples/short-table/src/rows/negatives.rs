//! The row "standalone negatives".

use crate::column::{build, is_zero, num, recipe, Column, Step};
use crate::guard::probe;
use crate::render::{wrong, Cell};
use crate::rows::Row;

/// The state whose negative is asked for.
pub(crate) const S: [Step; 2] = [Step::Insert(0.1), Step::Insert(0.2)];

/// `negate` runs inside `iterate`, `antijoin` and `reduce_abelian`, and consolidation can meet
/// a −x diff before its +x.
pub(crate) struct Negatives;

impl Row for Negatives {
    const NAME: &'static str = "standalone negatives";

    fn intro() -> String {
        format!(
            "`negate` runs inside `iterate`, `antijoin` and `reduce_abelian`, and consolidation \
             can meet a −x diff before its +x, so every state needs a negative that exists on its \
             own. S is {}. negate(S) must exist without knowing S's inputs; then the history where \
             the negative comes first: start from negate(S), insert 0.1, insert 0.2, and read \
             exactly zero. Equality with the empty state is the `==` row's question; this row \
             asks only that the negative exists and cancels exactly.",
            recipe(&S)
        )
    }

    fn cell<C: Column>() -> Cell {
        let probes = [probe(|| {
            let mut s = C::negate(build::<C>(&S)?)?;
            for step in S {
                match step {
                    Step::Insert(x) => C::insert(&mut s, x)?,
                    Step::Retract(x) => C::retract(&mut s, x)?,
                }
            }
            let got = C::read(&mut s)?;
            let history = format!("negate({}), then {}", recipe(&S), recipe(&S));
            if is_zero(got) {
                Ok(format!("{history}, reads {}", num(got)))
            } else {
                wrong(format!("{history}, reads {}, want 0.0", num(got)))
            }
        })];
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
    use float_abel::Exact;

    /// `Exact` whose negative is the negation of its rounded double.
    struct NegatesRounded;

    impl Mutation for NegatesRounded {
        const NAME: &'static str = "negates through a rounded f64";

        fn negate(s: Mutated) -> Outcome<Mutated> {
            Ok(Mutated {
                x: Exact::from(-s.x.to_f64()),
                ..s
            })
        }
    }

    #[test]
    fn float_abel_passes_and_the_mutant_and_the_control_fail() {
        let fa = Negatives::cell::<FloatAbel>();
        assert_eq!(
            fa.evidence,
            "ran: negate(+0.1 +0.2), then +0.1 +0.2, reads 0.0"
        );
        assert!(fa.pass);
        let mutant = Negatives::cell::<Mutant<NegatesRounded>>();
        assert!(!mutant.pass);
        assert_eq!(
            mutant.evidence,
            "ran: negate(+0.1 +0.2), then +0.1 +0.2, reads -2.7755575615628914e-17, want 0.0"
        );
        let control = Negatives::cell::<Control>();
        assert!(!control.pass);
        assert_eq!(control.evidence, mutant.evidence);
    }

    #[test]
    fn a_zero_of_either_sign_is_zero_and_nothing_else_is() {
        assert!(is_zero(0.0));
        assert!(is_zero(-0.0));
        assert!(!is_zero(5e-324));
        assert!(!is_zero(-5e-324));
        assert!(!is_zero(f64::NAN));
    }
}
