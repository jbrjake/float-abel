//! The operator surface of [`Exact`]: every operator is its checked twin unwrapped through the
//! named cap panic, by value and by reference, so an engine's `plus_equals`, `negate` and
//! `Multiply` all reach the same arithmetic.

use core::iter::Sum;
use core::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

use super::Exact;

impl From<f64> for Exact {
    /// A fresh state holding `x`: its finite value, or one count of its special. Total.
    fn from(x: f64) -> Self {
        Self::from_f64(x)
    }
}

impl Add<&Exact> for &Exact {
    type Output = Exact;
    fn add(self, rhs: &Exact) -> Exact {
        self.checked_add(rhs).unwrap_or_else(|cap| cap.exceed())
    }
}

impl Add for Exact {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        self.checked_add(&rhs).unwrap_or_else(|cap| cap.exceed())
    }
}

impl AddAssign<&Exact> for Exact {
    fn add_assign(&mut self, rhs: &Exact) {
        *self = &*self + rhs;
    }
}

impl AddAssign for Exact {
    fn add_assign(&mut self, rhs: Self) {
        *self += &rhs;
    }
}

impl Neg for &Exact {
    type Output = Exact;
    /// Total: a canonical mantissa is never `i128::MIN`, and a count is never `i32::MIN`.
    fn neg(self) -> Exact {
        Exact::assemble(-self.finite(), self.counts().map(|c| -c))
    }
}

impl Neg for Exact {
    type Output = Self;
    fn neg(self) -> Self {
        Self::assemble(-self.finite(), self.counts().map(|c| -c))
    }
}

impl Sub<&Exact> for &Exact {
    type Output = Exact;
    fn sub(self, rhs: &Exact) -> Exact {
        self.checked_sub(rhs).unwrap_or_else(|cap| cap.exceed())
    }
}

impl Sub for Exact {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        self.checked_sub(&rhs).unwrap_or_else(|cap| cap.exceed())
    }
}

impl SubAssign<&Exact> for Exact {
    fn sub_assign(&mut self, rhs: &Exact) {
        *self = &*self - rhs;
    }
}

impl SubAssign for Exact {
    fn sub_assign(&mut self, rhs: Self) {
        *self -= &rhs;
    }
}

impl Mul<&Exact> for &Exact {
    type Output = Exact;
    fn mul(self, rhs: &Exact) -> Exact {
        self.checked_mul(rhs).unwrap_or_else(|cap| cap.exceed())
    }
}

impl Mul for Exact {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        self.checked_mul(&rhs).unwrap_or_else(|cap| cap.exceed())
    }
}

impl Mul<i64> for &Exact {
    type Output = Exact;
    /// Integer scaling, as an engine applies a multiplicity or a retraction.
    fn mul(self, k: i64) -> Exact {
        self.checked_mul_i64(k).unwrap_or_else(|cap| cap.exceed())
    }
}

impl Mul<i64> for Exact {
    type Output = Self;
    fn mul(self, k: i64) -> Self {
        self.checked_mul_i64(k).unwrap_or_else(|cap| cap.exceed())
    }
}

impl Mul<isize> for &Exact {
    type Output = Exact;
    fn mul(self, k: isize) -> Exact {
        self * (k as i64)
    }
}

impl Mul<isize> for Exact {
    type Output = Self;
    fn mul(self, k: isize) -> Self {
        self * (k as i64)
    }
}

impl Sum for Exact {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, Add::add)
    }
}

impl<'a> Sum<&'a Self> for Exact {
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, |acc, x| &acc + x)
    }
}
