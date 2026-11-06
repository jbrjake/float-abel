//! The row "`==` / `Hash` / `is_zero` mean the number".

use crate::column::{build, recipe, Column, Step};
use crate::guard::probe;
use crate::render::{wrong, Cell};
use crate::rows::Row;

/// One number by two histories.
const ONE: [Step; 1] = [Step::Insert(1.0)];
const HALVES: [Step; 2] = [Step::Insert(0.5), Step::Insert(0.5)];
/// A state cancelled to zero.
const CANCELLED: [Step; 2] = [Step::Insert(0.1), Step::Retract(0.1)];
/// Two different numbers.
const TENTH: [Step; 1] = [Step::Insert(0.1)];
const FIFTH: [Step; 1] = [Step::Insert(0.2)];

/// Builders dedup by `==`, `is_zero` deletes an update, and `count_total` makes the weight data.
pub(crate) struct Equality;

impl Row for Equality {
    const NAME: &'static str = "`==` / `Hash` / `is_zero` mean the number";

    fn intro() -> String {
        format!(
            "Builders dedup by `==`, `is_zero` deletes an update, and `count_total` makes the \
             weight data, so records merge by comparison and route to workers by `Hash`. Probes: \
             {} and {} are `==` and hash equal; {} is zero and `==` the empty state; {} and {} \
             are not `==`, so an `Eq` that says yes to everything fails. A crate with `==` but no \
             `is_zero` is asked whether the state `==` the empty one. Hashes are from the standard \
             library's default hasher and are compared inside the run, never printed, because \
             their values are not stable across Rust releases.",
            recipe(&ONE),
            recipe(&HALVES),
            recipe(&CANCELLED),
            recipe(&TENTH),
            recipe(&FIFTH)
        )
    }

    fn cell<C: Column>() -> Cell {
        let (one, halves, cancelled) = (recipe(&ONE), recipe(&HALVES), recipe(&CANCELLED));
        let (tenth, fifth) = (recipe(&TENTH), recipe(&FIFTH));
        let probes = [
            probe(|| {
                let (a, b) = (build::<C>(&ONE)?, build::<C>(&HALVES)?);
                if C::eq(&a, &b)? {
                    Ok(format!("{one} == {halves}"))
                } else {
                    wrong(format!("{one} != {halves} by =="))
                }
            }),
            probe(|| {
                let (a, b) = (build::<C>(&ONE)?, build::<C>(&HALVES)?);
                if C::hash_eq(&a, &b)? {
                    Ok("their hashes are equal".to_owned())
                } else {
                    wrong(format!("{one} and {halves} hash differently"))
                }
            }),
            probe(|| {
                let s = build::<C>(&CANCELLED)?;
                if C::is_zero(&s)? {
                    Ok(format!("{cancelled} is zero"))
                } else {
                    wrong(format!("{cancelled} is not zero"))
                }
            }),
            probe(|| {
                let (s, empty) = (build::<C>(&CANCELLED)?, C::empty());
                if C::eq(&s, &empty)? {
                    Ok(format!("{cancelled} == empty"))
                } else {
                    wrong(format!("{cancelled} != empty by =="))
                }
            }),
            probe(|| {
                let (a, b) = (build::<C>(&TENTH)?, build::<C>(&FIFTH)?);
                if C::eq(&a, &b)? {
                    wrong(format!("{tenth} == {fifth} by =="))
                } else {
                    Ok(format!("{tenth} != {fifth}"))
                }
            }),
        ];
        Cell::from_probes(&probes, |readings| readings.join("; "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::column::{hashes_equal, Outcome};
    use crate::columns::control::Control;
    use crate::columns::mutant::{Mutant, Mutated, Mutation};
    use crate::columns::FloatAbel;

    /// `Exact` whose `Hash` mixes in the insert count while its `==` stays numeric.
    struct HashesTheCount;

    impl Mutation for HashesTheCount {
        const NAME: &'static str = "hashes an insert count";

        fn hash_eq(a: &Mutated, b: &Mutated) -> Outcome<bool> {
            Ok(hashes_equal(&(&a.x, a.inserts), &(&b.x, b.inserts)))
        }
    }

    #[test]
    fn float_abel_passes_and_the_mutant_fails_the_hash_leg_alone() {
        let fa = Equality::cell::<FloatAbel>();
        assert_eq!(
            fa.evidence,
            "ran: +1.0 == +0.5 ×2; their hashes are equal; +0.1 −0.1 is zero; +0.1 −0.1 == \
             empty; +0.1 != +0.2"
        );
        assert!(fa.pass);
        let mutant = Equality::cell::<Mutant<HashesTheCount>>();
        assert!(!mutant.pass);
        assert_eq!(mutant.evidence, "ran: +1.0 and +0.5 ×2 hash differently");
    }

    #[test]
    fn the_control_has_no_hash_and_nothing_else_wrong() {
        let control = Equality::cell::<Control>();
        assert!(!control.pass);
        assert_eq!(control.evidence, "absent: no Hash on f64");
    }
}
