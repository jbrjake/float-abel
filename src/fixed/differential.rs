//! differential-dataflow's weight traits on [`Fixed`], DESIGN.md § Fixed: the declared window as
//! a `Copy` diff, with the door check per value where the row becomes a weight, `Fixed::try_from`
//! inside the `explode`, so nothing downstream depends on what else is in the key. Each impl is
//! the group operator it names: `plus_equals` is `+=`, `negate` is `Neg`, `zero` is
//! [`Fixed::ZERO`], `is_zero` is [`Fixed::is_zero`], which is what deletes an update, and
//! `Multiply` is `Mul` by `i64` or `isize` on either side. Past the headroom the operators panic
//! with the named invariant, because `plus_equals` cannot return an error to the engine.
//!
//! There is no `Multiply<Fixed> for Fixed`: the product of two windows is not a value in either,
//! so a `join` of two `Fixed`-weighted collections does not compile, and a row's product enters
//! the diff through [`Fixed::deposit_product`] at the door, judged on the exact product. A fold of
//! products inside an engine is [`Exact`](crate::Exact)'s. `count_core` and `distinct_core` put
//! `From<i8>` out, which is pinned to the windows that hold every `i8`.

use differential_dataflow::difference::{Abelian, IsZero, Monoid, Multiply, Semigroup};

use super::Fixed;

impl<const LSB: i32, const LIMBS: usize> IsZero for Fixed<LSB, LIMBS> {
    #[inline]
    fn is_zero(&self) -> bool {
        Fixed::is_zero(self)
    }
}

impl<const LSB: i32, const LIMBS: usize> Semigroup for Fixed<LSB, LIMBS> {
    #[inline]
    fn plus_equals(&mut self, rhs: &Self) {
        *self += *rhs;
    }
}

impl<const LSB: i32, const LIMBS: usize> Monoid for Fixed<LSB, LIMBS> {
    #[inline]
    fn zero() -> Self {
        Self::ZERO
    }
}

impl<const LSB: i32, const LIMBS: usize> Abelian for Fixed<LSB, LIMBS> {
    #[inline]
    fn negate(&mut self) {
        *self = -*self;
    }
}

impl<const LSB: i32, const LIMBS: usize> Multiply<i64> for Fixed<LSB, LIMBS> {
    type Output = Self;
    #[inline]
    fn multiply(self, rhs: &i64) -> Self {
        self * *rhs
    }
}

impl<const LSB: i32, const LIMBS: usize> Multiply<isize> for Fixed<LSB, LIMBS> {
    type Output = Self;
    #[inline]
    fn multiply(self, rhs: &isize) -> Self {
        self * *rhs
    }
}

impl<const LSB: i32, const LIMBS: usize> Multiply<Fixed<LSB, LIMBS>> for i64 {
    type Output = Fixed<LSB, LIMBS>;
    #[inline]
    fn multiply(self, rhs: &Fixed<LSB, LIMBS>) -> Fixed<LSB, LIMBS> {
        *rhs * self
    }
}

impl<const LSB: i32, const LIMBS: usize> Multiply<Fixed<LSB, LIMBS>> for isize {
    type Output = Fixed<LSB, LIMBS>;
    #[inline]
    fn multiply(self, rhs: &Fixed<LSB, LIMBS>) -> Fixed<LSB, LIMBS> {
        *rhs * self
    }
}
