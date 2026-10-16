//! differential-dataflow's weight traits on [`Wide`], DESIGN.md § Wide: the `Copy` diff for an
//! engine whose weight slot must be `Copy` or that casts over a mapping, every double admitted
//! with nothing to check at the door. Each impl is the group operator it names: `plus_equals` is
//! `+=`, `negate` is `Neg`, `zero` is [`Wide::ZERO`], `is_zero` is [`Wide::is_zero`], which is
//! what deletes an update, and `Multiply` is `Mul` by `i64` or `isize` on either side, so a
//! `join`'s `R × isize` and an `explode`'s `isize × R` reach the same integer scaling. Past the
//! headroom the operators panic with the named invariant, because `plus_equals` cannot return an
//! error to the engine.
//!
//! There is no `Multiply<Wide> for Wide`: the product of two 2176-bit words has no carrier, so a
//! `join` of two `Wide`-weighted collections does not compile, and a row's product enters the
//! diff through [`Wide::deposit_product`] at the door, refused by name past `2^1037`. A fold of
//! products inside an engine is [`Exact`](crate::Exact)'s.

use differential_dataflow::difference::{Abelian, IsZero, Monoid, Multiply, Semigroup};

use super::Wide;

impl IsZero for Wide {
    #[inline]
    fn is_zero(&self) -> bool {
        Wide::is_zero(self)
    }
}

impl Semigroup for Wide {
    #[inline]
    fn plus_equals(&mut self, rhs: &Self) {
        *self += *rhs;
    }
}

impl Monoid for Wide {
    #[inline]
    fn zero() -> Self {
        Self::ZERO
    }
}

impl Abelian for Wide {
    #[inline]
    fn negate(&mut self) {
        *self = -*self;
    }
}

impl Multiply<i64> for Wide {
    type Output = Wide;
    #[inline]
    fn multiply(self, rhs: &i64) -> Wide {
        self * *rhs
    }
}

impl Multiply<isize> for Wide {
    type Output = Wide;
    #[inline]
    fn multiply(self, rhs: &isize) -> Wide {
        self * *rhs
    }
}

impl Multiply<Wide> for i64 {
    type Output = Wide;
    #[inline]
    fn multiply(self, rhs: &Wide) -> Wide {
        *rhs * self
    }
}

impl Multiply<Wide> for isize {
    type Output = Wide;
    #[inline]
    fn multiply(self, rhs: &Wide) -> Wide {
        *rhs * self
    }
}
