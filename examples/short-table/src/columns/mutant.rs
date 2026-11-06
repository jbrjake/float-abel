//! The mutants: `Exact` with exactly one property broken, so each row can show that its probes
//! tell the property apart on the column that passes them. Test-only.

use std::marker::PhantomData;

use float_abel::Exact;

use crate::column::{Column, Outcome, View};
use crate::columns::FloatAbel;

/// A mutant's state: the honest `Exact`, plus the two things a mutation may need to remember.
#[derive(Debug, Clone)]
pub(crate) struct Mutated {
    /// The sum.
    pub(crate) x: Exact,
    /// How many inserts reached this state, counted by the default `insert`.
    pub(crate) inserts: u64,
    /// A flag a mutation may set, false by default.
    pub(crate) flag: bool,
}

/// One broken property. Every method defaults to [`FloatAbel`]'s behaviour; a mutant overrides
/// the one it breaks.
pub(crate) trait Mutation {
    /// The mutant's name, which names what it breaks.
    const NAME: &'static str;

    fn insert(s: &mut Mutated, x: f64) -> Outcome<()> {
        s.inserts += 1;
        FloatAbel::insert(&mut s.x, x)
    }

    fn retract(s: &mut Mutated, x: f64) -> Outcome<()> {
        FloatAbel::retract(&mut s.x, x)
    }

    fn read(s: &mut Mutated) -> Outcome<f64> {
        FloatAbel::read(&mut s.x)
    }

    fn view(s: &Mutated) -> Outcome<View> {
        FloatAbel::view(&s.x)
    }

    fn batch(xs: &[f64]) -> Option<Outcome<f64>> {
        FloatAbel::batch(xs)
    }

    fn merge(a: Mutated, b: Mutated) -> Outcome<Mutated> {
        Ok(Mutated {
            x: FloatAbel::merge(a.x, b.x)?,
            inserts: a.inserts + b.inserts,
            flag: a.flag || b.flag,
        })
    }

    fn negate(s: Mutated) -> Outcome<Mutated> {
        Ok(Mutated {
            x: FloatAbel::negate(s.x)?,
            ..s
        })
    }

    fn eq(a: &Mutated, b: &Mutated) -> Outcome<bool> {
        FloatAbel::eq(&a.x, &b.x)
    }

    fn hash_eq(a: &Mutated, b: &Mutated) -> Outcome<bool> {
        FloatAbel::hash_eq(&a.x, &b.x)
    }

    fn is_zero(s: &Mutated) -> Outcome<bool> {
        FloatAbel::is_zero(&s.x)
    }
}

/// The column a [`Mutation`] makes.
pub(crate) struct Mutant<M>(PhantomData<M>);

impl<M: Mutation> Column for Mutant<M> {
    const NAME: &'static str = M::NAME;
    type State = Mutated;

    fn empty() -> Mutated {
        Mutated {
            x: Exact::default(),
            inserts: 0,
            flag: false,
        }
    }

    fn insert(s: &mut Mutated, x: f64) -> Outcome<()> {
        M::insert(s, x)
    }

    fn retract(s: &mut Mutated, x: f64) -> Outcome<()> {
        M::retract(s, x)
    }

    fn read(s: &mut Mutated) -> Outcome<f64> {
        M::read(s)
    }

    fn view(s: &Mutated) -> Outcome<View> {
        M::view(s)
    }

    fn batch(xs: &[f64]) -> Option<Outcome<f64>> {
        M::batch(xs)
    }

    fn merge(a: Mutated, b: Mutated) -> Outcome<Mutated> {
        M::merge(a, b)
    }

    fn negate(s: Mutated) -> Outcome<Mutated> {
        M::negate(s)
    }

    fn eq(a: &Mutated, b: &Mutated) -> Outcome<bool> {
        M::eq(a, b)
    }

    fn hash_eq(a: &Mutated, b: &Mutated) -> Outcome<bool> {
        M::hash_eq(a, b)
    }

    fn is_zero(s: &Mutated) -> Outcome<bool> {
        M::is_zero(s)
    }
}

/// No property broken: the control for the mutants themselves, which must read exactly as
/// [`FloatAbel`] does.
pub(crate) struct Honest;

impl Mutation for Honest {
    const NAME: &'static str = "float-abel, wrapped";
}
