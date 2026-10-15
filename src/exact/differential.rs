//! differential-dataflow's weight traits on [`Exact`], DESIGN.md § differential-dataflow, so a
//! collection takes it as a diff the way it takes `isize`. Each impl is the group operator it
//! names: `plus_equals` is `+=`, `negate` is `Neg`, `zero` is [`Exact::ZERO`], `is_zero` is
//! [`Exact::is_zero`], which is what deletes an update, and `Multiply` is `Mul` by `Exact`, `i64`
//! or `isize` on either side, so a `join`'s `R × R2` and an `explode`'s `R2 × R` reach the same
//! integer arithmetic. Past the cap the operators panic with the named invariant, because
//! `plus_equals` cannot return an error to the engine.

use differential_dataflow::difference::{Abelian, IsZero, Monoid, Multiply, Semigroup};

use super::Exact;

impl IsZero for Exact {
    #[inline]
    fn is_zero(&self) -> bool {
        Exact::is_zero(self)
    }
}

impl Semigroup for Exact {
    #[inline]
    fn plus_equals(&mut self, rhs: &Self) {
        *self += rhs;
    }
}

impl Monoid for Exact {
    #[inline]
    fn zero() -> Self {
        Self::ZERO
    }
}

impl Abelian for Exact {
    #[inline]
    fn negate(&mut self) {
        self.negate_assign();
    }
}

impl Multiply<Exact> for Exact {
    type Output = Exact;
    #[inline]
    fn multiply(self, rhs: &Exact) -> Exact {
        self * rhs
    }
}

impl Multiply<i64> for Exact {
    type Output = Exact;
    #[inline]
    fn multiply(self, rhs: &i64) -> Exact {
        self * *rhs
    }
}

impl Multiply<isize> for Exact {
    type Output = Exact;
    #[inline]
    fn multiply(self, rhs: &isize) -> Exact {
        self * *rhs
    }
}

impl Multiply<Exact> for i64 {
    type Output = Exact;
    #[inline]
    fn multiply(self, rhs: &Exact) -> Exact {
        rhs * self
    }
}

impl Multiply<Exact> for isize {
    type Output = Exact;
    #[inline]
    fn multiply(self, rhs: &Exact) -> Exact {
        rhs * self
    }
}
