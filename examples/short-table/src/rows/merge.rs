//! The row "merges partial sums".

use crate::column::{build, num, recipe, Column, Step};
use crate::guard::probe;
use crate::render::{wrong, Cell};
use crate::rows::Row;

/// What each of three workers holds. The split puts 0.125 beside 1e16 in one worker and the
/// matching −1e16 in another, so losing the small part under the large one anywhere shows.
pub(crate) const WORKERS: [&[Step]; 3] = [
    &[Step::Insert(0.5), Step::Insert(0.25)],
    &[Step::Insert(0.125), Step::Insert(1e16)],
    &[Step::Insert(1.0), Step::Insert(-1e16)],
];

/// The exact total, 15/8, itself a double.
pub(crate) const TOTAL: f64 = 1.875;

/// Workers sum their shares, and the engine combines the partial states.
pub(crate) struct Merge;

/// One merge order: the three workers' states combined as `(w1 ⊕ w2) ⊕ w3`, or as
/// `w3 ⊕ (w2 ⊕ w1)`.
fn merged<C: Column>(reversed: bool) -> Result<f64, crate::column::Shortfall> {
    let w1 = build::<C>(WORKERS[0])?;
    let w2 = build::<C>(WORKERS[1])?;
    let w3 = build::<C>(WORKERS[2])?;
    let mut all = if reversed {
        C::merge(w3, C::merge(w2, w1)?)?
    } else {
        C::merge(C::merge(w1, w2)?, w3)?
    };
    C::read(&mut all)
}

impl Row for Merge {
    const NAME: &'static str = "merges partial sums";

    fn intro() -> String {
        format!(
            "Workers sum their shares and the engine combines the partial states. Three workers \
             hold {}, {} and {}; merged as (w1 ⊕ w2) ⊕ w3 and as w3 ⊕ (w2 ⊕ w1), both must read \
             {}, the exact total and itself a double. The split is chosen so that losing the \
             0.125 under 1e16 anywhere shows: plain f64 reads 0.0 one-shot and merged, and a \
             merge that passes through a rounded double reads 0.75 in one order and 1.0 in the \
             other. Whether the merged states have equal bits is the previous row's question.",
            recipe(WORKERS[0]),
            recipe(WORKERS[1]),
            recipe(WORKERS[2]),
            num(TOTAL)
        )
    }

    fn cell<C: Column>() -> Cell {
        let probes: Vec<_> = [(false, "(w1 ⊕ w2) ⊕ w3"), (true, "w3 ⊕ (w2 ⊕ w1)")]
            .into_iter()
            .map(|(reversed, order)| {
                probe(|| {
                    let got = merged::<C>(reversed)?;
                    if got.to_bits() == TOTAL.to_bits() {
                        Ok(format!("{order} reads {}", num(got)))
                    } else {
                        wrong(format!("{order} reads {}, want {}", num(got), num(TOTAL)))
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
    use float_abel::Exact;

    /// `Exact` that merges the other worker's state through its rounded double.
    struct MergesRounded;

    impl Mutation for MergesRounded {
        const NAME: &'static str = "merges through a rounded f64";

        fn merge(a: Mutated, b: Mutated) -> Outcome<Mutated> {
            Ok(Mutated {
                x: a.x + Exact::from(b.x.to_f64()),
                inserts: a.inserts + b.inserts,
                flag: a.flag || b.flag,
            })
        }
    }

    #[test]
    fn float_abel_passes_and_the_mutant_and_the_control_fail() {
        let fa = Merge::cell::<FloatAbel>();
        assert!(fa.pass, "{}", fa.evidence);
        assert_eq!(
            fa.evidence,
            "ran: (w1 ⊕ w2) ⊕ w3 reads 1.875; w3 ⊕ (w2 ⊕ w1) reads 1.875"
        );
        let mutant = Merge::cell::<Mutant<MergesRounded>>();
        assert!(!mutant.pass);
        assert!(
            mutant.evidence.contains("reads 0.75, want 1.875")
                || mutant.evidence.contains("reads 1.0, want 1.875"),
            "{}",
            mutant.evidence
        );
        let control = Merge::cell::<Control>();
        assert!(!control.pass);
        assert_eq!(
            control.evidence,
            "ran: (w1 ⊕ w2) ⊕ w3 reads 0.0, want 1.875; ran: w3 ⊕ (w2 ⊕ w1) reads 0.0, want 1.875"
        );
    }

    #[test]
    fn the_total_is_exact_and_one_shot_f64_loses_it() {
        let all: Vec<f64> = WORKERS
            .iter()
            .flat_map(|w| w.iter())
            .map(|s| match s {
                Step::Insert(x) => *x,
                Step::Retract(x) => -*x,
            })
            .collect();
        let mut exact = Exact::default();
        for x in &all {
            exact += Exact::from(*x);
        }
        assert_eq!(exact.to_f64(), TOTAL);
        assert_eq!(all.iter().sum::<f64>(), 0.0);
    }
}
