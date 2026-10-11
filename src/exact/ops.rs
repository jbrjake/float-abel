//! The operator surface of [`Exact`]: every operator is its checked twin unwrapped through the
//! named cap panic, by value and by reference, so an engine's `plus_equals`, `negate` and
//! `Multiply` all reach the same arithmetic.

use core::iter::Sum;
use core::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

use super::Exact;

impl From<f64> for Exact {
    /// A fresh state holding `x`.
    ///
    /// # Panics
    ///
    /// With the named cap when `x` is NaN or ±∞ on a build without the heap tier, where
    /// [`Exact::try_from_f64`] returns [`PastCap::Special`](super::PastCap::Special) instead.
    fn from(x: f64) -> Self {
        Self::try_from_f64(x).unwrap_or_else(|cap| cap.exceed())
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
        &self + &rhs
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
    /// Total on every tier: a canonical mantissa is never the most negative value.
    fn neg(self) -> Exact {
        Exact::from_compact(-self.compact())
    }
}

impl Neg for Exact {
    type Output = Self;
    fn neg(self) -> Self {
        -&self
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
        &self - &rhs
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
        &self * &rhs
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
        &self * k
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
        &self * (k as i64)
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
