//! The row "exact products (explode, join)".

use crate::column::{build, is_zero, num, recipe, Column, Step};
use crate::guard::probe;
use crate::render::{wrong, Cell};
use crate::rows::Row;

/// The state scaled by counts.
const S: [Step; 2] = [Step::Insert(0.1), Step::Insert(0.2)];

/// `1 + 2⁻⁴⁰`, by its bits: one, with the significand bit forty places down set.
const ONE_PLUS_2_NEG_40: f64 = f64::from_bits(0x3FF0_0000_0000_1000);
/// `1 + 2⁻³⁹`, the double part of the square.
const ONE_PLUS_2_NEG_39: f64 = f64::from_bits(0x3FF0_0000_0000_2000);
/// `2⁻⁸⁰`, the part of the square a 53-bit or a 64.64 product drops.
const TWO_NEG_80: f64 = f64::from_bits(0x3AF0_0000_0000_0000);

/// `explode` multiplies each value by its ±1 diff, and `join` evaluates `(a + b) × c` as
/// `a × c + b × c`.
pub(crate) struct Products;

impl Row for Products {
    const NAME: &'static str = "exact products (explode, join)";

    fn intro() -> String {
        let thrice: Vec<Step> = S.iter().copied().cycle().take(6).collect();
        format!(
            "`explode` multiplies each value by its ±1 diff, and `join` evaluates (a + b) × c as \
             a × c + b × c, so a weight multiplies exactly, by counts and by other weights. By \
             count, with S = {}: scale(S, 3) must read what {} reads, and scale(S, −1) then {} \
             must read zero. By weight: the square of +{}, less {}, must read {}. The exact square \
             is 1 + 2⁻³⁹ + 2⁻⁸⁰; a product rounded to 53 bits, or truncated at 64 fractional \
             bits, drops the 2⁻⁸⁰ and reads 0.",
            recipe(&S),
            recipe(&thrice),
            recipe(&S),
            num(ONE_PLUS_2_NEG_40),
            num(ONE_PLUS_2_NEG_39),
            num(TWO_NEG_80)
        )
    }

    fn cell<C: Column>() -> Cell {
        let s = recipe(&S);
        let probes = [
            probe(|| {
                let thrice: Vec<Step> = S.iter().copied().cycle().take(6).collect();
                let mut scaled = C::scale(build::<C>(&S)?, 3)?;
                let mut summed = build::<C>(&thrice)?;
                let (a, b) = (C::read(&mut scaled)?, C::read(&mut summed)?);
                if a.to_bits() == b.to_bits() {
                    Ok(format!(
                        "scale({s}, 3) reads {}, as {} does",
                        num(a),
                        recipe(&thrice)
                    ))
                } else {
                    wrong(format!(
                        "scale({s}, 3) reads {}, but {} reads {}",
                        num(a),
                        recipe(&thrice),
                        num(b)
                    ))
                }
            }),
            probe(|| {
                let mut st = C::scale(build::<C>(&S)?, -1)?;
                for step in S {
                    match step {
                        Step::Insert(x) => C::insert(&mut st, x)?,
                        Step::Retract(x) => C::retract(&mut st, x)?,
                    }
                }
                let got = C::read(&mut st)?;
                if is_zero(got) {
                    Ok(format!("scale({s}, -1), then {s}, reads {}", num(got)))
                } else {
                    wrong(format!(
                        "scale({s}, -1), then {s}, reads {}, want 0.0",
                        num(got)
                    ))
                }
            }),
            probe(|| {
                let factor = [Step::Insert(ONE_PLUS_2_NEG_40)];
                let mut square = C::mul(build::<C>(&factor)?, build::<C>(&factor)?)?;
                C::retract(&mut square, ONE_PLUS_2_NEG_39)?;
                let got = C::read(&mut square)?;
                let asked = format!(
                    "(+{})², less {},",
                    num(ONE_PLUS_2_NEG_40),
                    num(ONE_PLUS_2_NEG_39)
                );
                if got.to_bits() == TWO_NEG_80.to_bits() {
                    Ok(format!("{asked} reads {}", num(got)))
                } else {
                    wrong(format!(
                        "{asked} reads {}, want {}",
                        num(got),
                        num(TWO_NEG_80)
                    ))
                }
            }),
        ];
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

    /// `Exact` whose product goes through the two factors' rounded doubles.
    struct MultipliesRounded;

    impl Mutation for MultipliesRounded {
        const NAME: &'static str = "multiplies through rounded f64s";

        fn mul(a: Mutated, b: Mutated) -> Outcome<Mutated> {
            Ok(Mutated {
                x: Exact::from(a.x.to_f64() * b.x.to_f64()),
                inserts: a.inserts + b.inserts,
                flag: a.flag || b.flag,
            })
        }
    }

    #[test]
    fn the_constants_are_the_doubles_the_prose_names() {
        assert_eq!(ONE_PLUS_2_NEG_40, 1.0 + 2f64.powi(-40));
        assert_eq!(ONE_PLUS_2_NEG_39, 1.0 + 2f64.powi(-39));
        assert_eq!(TWO_NEG_80, 2f64.powi(-80));
        assert_eq!(num(TWO_NEG_80), "8.271806125530277e-25");
        // The square in f64 loses the 2⁻⁸⁰ outright.
        assert_eq!(
            ONE_PLUS_2_NEG_40 * ONE_PLUS_2_NEG_40 - ONE_PLUS_2_NEG_39,
            0.0
        );
    }

    #[test]
    fn float_abel_passes_and_the_mutant_and_the_control_fail_by_weight() {
        let fa = Products::cell::<FloatAbel>();
        assert!(fa.pass, "{}", fa.evidence);
        assert!(
            fa.evidence.ends_with("reads 8.271806125530277e-25"),
            "{}",
            fa.evidence
        );
        let mutant = Products::cell::<Mutant<MultipliesRounded>>();
        assert!(!mutant.pass);
        assert!(
            mutant
                .evidence
                .ends_with("reads 0.0, want 8.271806125530277e-25"),
            "{}",
            mutant.evidence
        );
        assert!(!mutant.evidence.contains("scale"), "{}", mutant.evidence);
        let control = Products::cell::<Control>();
        assert!(!control.pass);
        assert!(
            control
                .evidence
                .contains("reads 0.0, want 8.271806125530277e-25"),
            "{}",
            control.evidence
        );
    }
}
