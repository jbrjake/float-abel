//! The operator surface of [`Fixed`]: the group operators on the limbs, so an engine's
//! `plus_equals`, `negate` and `Multiply` all reach the same arithmetic, and the checked
//! constructor from a double. Every operator asserts the headroom in every build, because a
//! wrapped sum is silently wrong.

use core::iter::Sum;
use core::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

use super::{Fixed, OutOfWindow};
use crate::limbs;

impl<const LSB: i32, const LIMBS: usize> TryFrom<f64> for Fixed<LSB, LIMBS> {
    type Error = OutOfWindow;

    /// A fresh state holding `x`, or the refusal [`Fixed::try_deposit`] would give.
    fn try_from(x: f64) -> Result<Self, OutOfWindow> {
        let mut f = Self::ZERO;
        f.try_deposit(x)?;
        Ok(f)
    }
}

impl<const LSB: i32, const LIMBS: usize> AddAssign for Fixed<LSB, LIMBS> {
    fn add_assign(&mut self, rhs: Self) {
        let overflow = limbs::add_assign(&mut self.limbs, &rhs.limbs);
        assert!(!overflow, "Fixed: headroom exhausted in an add");
    }
}

impl<const LSB: i32, const LIMBS: usize> Add for Fixed<LSB, LIMBS> {
    type Output = Self;
    fn add(mut self, rhs: Self) -> Self {
        self += rhs;
        self
    }
}

impl<const LSB: i32, const LIMBS: usize> Neg for Fixed<LSB, LIMBS> {
    type Output = Self;
    fn neg(mut self) -> Self {
        let overflow = limbs::negate(&mut self.limbs);
        assert!(!overflow, "Fixed: headroom exhausted in a negation");
        self
    }
}

impl<const LSB: i32, const LIMBS: usize> SubAssign for Fixed<LSB, LIMBS> {
    fn sub_assign(&mut self, rhs: Self) {
        *self += -rhs;
    }
}

impl<const LSB: i32, const LIMBS: usize> Sub for Fixed<LSB, LIMBS> {
    type Output = Self;
    fn sub(mut self, rhs: Self) -> Self {
        self -= rhs;
        self
    }
}

impl<const LSB: i32, const LIMBS: usize> Mul<i64> for Fixed<LSB, LIMBS> {
    type Output = Self;

    /// Integer scaling, as an engine applies a multiplicity or a retraction.
    fn mul(self, k: i64) -> Self {
        let mut mag = [0u64; LIMBS];
        let negative = limbs::abs_into(&self.limbs, &mut mag);
        let carry = limbs::mul_assign_u64(&mut mag, k.unsigned_abs());
        assert!(
            carry == 0 && !limbs::is_negative(&mag),
            "Fixed: headroom exhausted in a scalar multiply"
        );
        if negative != (k < 0) {
            limbs::negate(&mut mag);
        }
        Self { limbs: mag }
    }
}

impl<const LSB: i32, const LIMBS: usize> Mul<isize> for Fixed<LSB, LIMBS> {
    type Output = Self;
    fn mul(self, k: isize) -> Self {
        self * (k as i64)
    }
}

impl<const LSB: i32, const LIMBS: usize> Sum for Fixed<LSB, LIMBS> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, Add::add)
    }
}

impl<'a, const LSB: i32, const LIMBS: usize> Sum<&'a Self> for Fixed<LSB, LIMBS> {
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, |acc, x| acc + *x)
    }
}
