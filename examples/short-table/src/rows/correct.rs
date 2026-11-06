//! The row "correct on every input tested".

use crate::column::{build, num, recipe, same_number, Column, Step};
use crate::guard::probe;
use crate::render::{wrong, Cell};
use crate::rows::Row;

/// One finite sum whose exact answer is itself a double.
pub(crate) struct Input {
    /// The values, in the order they arrive; every one is an insert.
    pub(crate) values: Vec<f64>,
    /// The exact sum.
    pub(crate) answer: f64,
    /// Why it is here.
    pub(crate) why: &'static str,
}

/// The eleven inputs.
pub(crate) fn inputs() -> Vec<Input> {
    let input = |values: Vec<f64>, answer: f64, why: &'static str| Input {
        values,
        answer,
        why,
    };
    let mut tenths = vec![0.1; 10];
    tenths.push(-1.0);
    vec![
        input(
            vec![0.1, 0.2, -0.1, -0.2],
            0.0,
            "the README's opening example",
        ),
        input(vec![1e100, 1.0, -1e100], 1.0, "insert, insert, delete"),
        input(
            vec![1e200, 1e100, 1.0, -1e100, -1e200],
            1.0,
            "three scales defeat one correction term",
        ),
        input(
            tenths,
            2f64.powi(-54),
            "0.1 is 3602879701896397 / 2⁵⁵, so ten of them less one is 2 / 2⁵⁵; a carrier holding \
             decimal 0.1 reads 0",
        ),
        input(vec![1e308, 1e308, -1e308], 1e308, "a partial past f64::MAX"),
        input(
            vec![f64::MAX, f64::MAX, -f64::MAX],
            f64::MAX,
            "the same at the top of the range",
        ),
        input(vec![1.0, -1.0], 0.0, "a state cancelled to exactly zero"),
        input(vec![5e-324, 5e-324], 1e-323, "subnormals"),
        input(vec![1e-20, 1.0, -1.0], 1e-20, "below a fixed grid"),
        input(vec![1e30, 2.0, -1e30], 2.0, "above a fixed range"),
        input(vec![1.0; 65536], 65536.0, "a long stream of one magnitude"),
    ]
}

/// The values as a recipe: every one an insert.
fn steps(values: &[f64]) -> Vec<Step> {
    values.iter().map(|&x| Step::Insert(x)).collect()
}

/// Every other row assumes the arithmetic underneath is right.
pub(crate) struct Correct;

impl Row for Correct {
    const NAME: &'static str = "correct on every input tested";

    fn intro() -> String {
        let listed: Vec<String> = inputs()
            .iter()
            .map(|i| {
                format!(
                    "{} is {} ({})",
                    recipe(&steps(&i.values)),
                    num(i.answer),
                    i.why
                )
            })
            .collect();
        format!(
            "Every other row assumes the arithmetic underneath is right. Eleven finite sums whose \
             exact answers are themselves doubles, so no oracle is needed, each run one value at \
             a time through the column's insert and, where the crate has one, through its entry \
             for a whole slice; both paths must read the answer. The specials have their own \
             row. The inputs: {}.",
            listed.join("; ")
        )
    }

    fn cell<C: Column>() -> Cell {
        let mut probes = Vec::new();
        let mut batched = false;
        for i in inputs() {
            let st = steps(&i.values);
            let shown = recipe(&st);
            probes.push(probe(|| {
                let mut s = build::<C>(&st)?;
                let got = C::read(&mut s)?;
                if same_number(got, i.answer) {
                    Ok(num(got))
                } else {
                    wrong(format!(
                        "{shown} reads {}, want {}",
                        num(got),
                        num(i.answer)
                    ))
                }
            }));
            // The slice entry runs under the guard too: whether the crate has one is known only
            // once the call returns, and the call itself can panic.
            let slice = match probe(|| Ok(C::batch(&i.values))) {
                Ok(None) => None,
                Ok(Some(outcome)) => Some(outcome),
                Err(shortfall) => Some(Err(shortfall)),
            };
            if let Some(outcome) = slice {
                batched = true;
                probes.push(outcome.and_then(|got| {
                    if same_number(got, i.answer) {
                        Ok(num(got))
                    } else {
                        wrong(format!(
                            "the slice entry on {shown} reads {}, want {}",
                            num(got),
                            num(i.answer)
                        ))
                    }
                }));
            }
        }
        Cell::from_probes(&probes, |_| {
            if batched {
                "all 11 inputs read their answer, one value at a time and through the slice entry"
                    .to_owned()
            } else {
                "all 11 inputs read their answer, one value at a time; no slice entry".to_owned()
            }
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

    /// `Exact` that drops every insert after the 65,535th.
    struct StopsCounting;

    impl Mutation for StopsCounting {
        const NAME: &'static str = "ignores inserts past the 65,535th";

        fn insert(s: &mut Mutated, x: f64) -> Outcome<()> {
            s.inserts += 1;
            if s.inserts > 65535 {
                return Ok(());
            }
            FloatAbel::insert(&mut s.x, x)
        }
    }

    #[test]
    fn every_answer_is_the_exact_sum_and_a_double() {
        // Each answer, re-derived: the inputs are chosen so the exact sum is a double, which
        // Exact's correctly rounded read then returns unchanged.
        for i in inputs() {
            let mut exact = float_abel::Exact::default();
            for &x in &i.values {
                exact += float_abel::Exact::from(x);
            }
            assert_eq!(exact.to_f64().to_bits(), i.answer.to_bits(), "{}", i.why);
        }
        assert_eq!(num(2f64.powi(-54)), "5.551115123125783e-17");
        assert_eq!(inputs().len(), 11);
    }

    #[test]
    fn float_abel_passes_and_the_mutant_and_the_control_fail() {
        let fa = Correct::cell::<FloatAbel>();
        assert_eq!(
            fa.evidence,
            "ran: all 11 inputs read their answer, one value at a time and through the slice entry"
        );
        assert!(fa.pass);
        let mutant = Correct::cell::<Mutant<StopsCounting>>();
        assert!(!mutant.pass);
        assert_eq!(
            mutant.evidence,
            "ran: +1.0 ×65536 reads 65535.0, want 65536.0"
        );
        let control = Correct::cell::<Control>();
        assert!(!control.pass);
        assert!(
            control
                .evidence
                .contains("ran: +1e100 +1.0 +(-1e100) reads 0.0, want 1.0"),
            "{}",
            control.evidence
        );
    }
}
