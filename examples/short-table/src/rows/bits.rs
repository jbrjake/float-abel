//! The row "same bits for the same number, however reached".

use crate::column::{build, recipe, Column, Step};
use crate::guard::probe;
use crate::render::{wrong, Cell};
use crate::rows::Row;

/// `1 + ε`, the double just above one.
const ABOVE_ONE: f64 = 1.0 + f64::EPSILON;

/// Five routes, each two recipes that hold one number.
pub(crate) const ROUTES: [(&str, &[Step], &[Step]); 5] = [
    (
        "split",
        &[Step::Insert(1.0)],
        &[Step::Insert(0.5), Step::Insert(0.5)],
    ),
    (
        "order",
        &[
            Step::Insert(ABOVE_ONE),
            Step::Insert(ABOVE_ONE),
            Step::Insert(1.0),
        ],
        &[
            Step::Insert(ABOVE_ONE),
            Step::Insert(1.0),
            Step::Insert(ABOVE_ONE),
        ],
    ),
    ("cancel", &[], &[Step::Insert(0.1), Step::Retract(0.1)]),
    (
        "return",
        &[Step::Insert(1.0)],
        &[Step::Insert(1.0), Step::Insert(0.5), Step::Retract(0.5)],
    ),
    (
        "excursion",
        &[Step::Insert(1e-12)],
        &[Step::Insert(1e-12), Step::Insert(1e12), Step::Retract(1e12)],
    ),
];

/// A recipe as the evidence names it, the empty one by name.
fn named(steps: &[Step]) -> String {
    if steps.is_empty() {
        "empty".to_owned()
    } else {
        recipe(steps)
    }
}

/// Consolidation adds equal-key diffs in an unspecified order and matches them structurally.
pub(crate) struct Bits;

impl Row for Bits {
    const NAME: &'static str = "same bits for the same number, however reached";

    fn intro() -> String {
        let routes: Vec<String> = ROUTES
            .iter()
            .map(|(name, l, r)| format!("{name} ({} against {})", named(l), named(r)))
            .collect();
        format!(
            "Consolidation adds equal-key diffs in an unspecified order and matches them \
             structurally, so one number has to be one bit pattern whatever history produced \
             it. Five routes, each two histories of the same number, compared through the \
             crate's own view of its state, its bytes or its `Debug`: {}. The excursion takes \
             float-abel through its heap form and back. A column passes if every route's two \
             views are equal.",
            routes.join("; ")
        )
    }

    fn cell<C: Column>() -> Cell {
        let probes: Vec<_> = ROUTES
            .iter()
            .map(|(name, left, right)| {
                probe(|| {
                    let a = C::view(&build::<C>(left)?)?;
                    let b = C::view(&build::<C>(right)?)?;
                    if a == b {
                        Ok((*name).to_owned())
                    } else {
                        wrong(format!(
                            "{name}: {} against {}, {}",
                            named(left),
                            named(right),
                            a.difference(&b)
                        ))
                    }
                })
            })
            .collect();
        Cell::from_probes(&probes, |_| "all 5 routes view equal".to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::column::{Outcome, View};
    use crate::columns::control::Control;
    use crate::columns::mutant::{Mutant, Mutated, Mutation};
    use crate::columns::FloatAbel;

    /// `Exact` whose view carries the number of inserts beside the sum.
    struct CountsInserts;

    impl Mutation for CountsInserts {
        const NAME: &'static str = "view carries an insert count";

        fn view(s: &Mutated) -> Outcome<View> {
            let mut bytes = s.x.encode();
            bytes.extend_from_slice(&s.inserts.to_le_bytes());
            Ok(View::Bytes(bytes))
        }
    }

    #[test]
    fn float_abel_passes_and_the_mutant_and_the_control_fail() {
        assert!(Bits::cell::<FloatAbel>().pass);
        let mutant = Bits::cell::<Mutant<CountsInserts>>();
        assert!(!mutant.pass);
        assert!(
            mutant
                .evidence
                .starts_with("ran: split: +1.0 against +0.5 ×2, bytes differ from offset"),
            "{}",
            mutant.evidence
        );
        let control = Bits::cell::<Control>();
        assert!(!control.pass);
        assert!(control.evidence.contains("order:"), "{}", control.evidence);
        assert!(
            control.evidence.contains("excursion:"),
            "{}",
            control.evidence
        );
        assert!(!control.evidence.contains("split:"), "{}", control.evidence);
    }

    #[test]
    fn the_excursion_leaves_and_returns_to_the_compact_form() {
        let (_, left, right) = ROUTES[4];
        let mut s = build::<FloatAbel>(&right[..2]).unwrap();
        assert!(!s.is_compact(), "1e-12 + 1e12 needs the heap form");
        FloatAbel::retract(&mut s, 1e12).unwrap();
        assert!(s.is_compact());
        assert_eq!(s, build::<FloatAbel>(left).unwrap());
    }
}
