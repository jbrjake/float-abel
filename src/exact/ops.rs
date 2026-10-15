//! The operator surface of [`Exact`]: every operator is its checked twin unwrapped through the
//! named cap panic, by value and by reference, so an engine's `plus_equals`, `negate` and
//! `Multiply` all reach the same arithmetic. Each is `#[inline]`, as is the chain beneath it down
//! to the compact tier, so a consumer's `+=` on a compact word compiles to the integer ops in
//! place rather than a call across the crate boundary returning a `Result` through memory; the
//! heap form's paths are the out-of-line slow paths behind them.

use core::iter::Sum;
use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use super::compact::Compact;
use super::Exact;

impl From<f64> for Exact {
    /// A fresh state holding `x`: its finite value, or one count of its special. Total.
    #[inline]
    fn from(x: f64) -> Self {
        Self::from_f64(x)
    }
}

impl From<i8> for Exact {
    /// The integer, exactly, by integer arithmetic: the one that differential-dataflow's
    /// `count_core` and `distinct_core` put out.
    #[inline]
    fn from(k: i8) -> Self {
        Self::assemble(Compact::from_i64(i64::from(k)), [0, 0, 0])
    }
}

impl Add<&Exact> for &Exact {
    type Output = Exact;
    #[inline]
    fn add(self, rhs: &Exact) -> Exact {
        self.checked_add(rhs).unwrap_or_else(|cap| cap.exceed())
    }
}

impl Add for Exact {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        self.checked_add(&rhs).unwrap_or_else(|cap| cap.exceed())
    }
}

impl Add<&Exact> for Exact {
    type Output = Self;
    #[inline]
    fn add(self, rhs: &Exact) -> Self {
        self.checked_add(rhs).unwrap_or_else(|cap| cap.exceed())
    }
}

impl AddAssign<&Exact> for Exact {
    #[inline]
    fn add_assign(&mut self, rhs: &Exact) {
        let sum = &*self + rhs;
        self.replace(sum);
    }
}

impl AddAssign for Exact {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self += &rhs;
    }
}

impl Neg for &Exact {
    type Output = Exact;
    /// Total: a canonical mantissa is never `i128::MIN`, a count is never `i32::MIN`, and a heap
    /// run carries its sign.
    #[inline]
    fn neg(self) -> Exact {
        let mut v = self.clone();
        v.negate_assign();
        v
    }
}

impl Neg for Exact {
    type Output = Self;
    /// In place: a heap word keeps its block and flips its signs.
    #[inline]
    fn neg(mut self) -> Self {
        self.negate_assign();
        self
    }
}

impl Sub<&Exact> for &Exact {
    type Output = Exact;
    #[inline]
    fn sub(self, rhs: &Exact) -> Exact {
        self.checked_sub(rhs).unwrap_or_else(|cap| cap.exceed())
    }
}

impl Sub for Exact {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        self.checked_sub(&rhs).unwrap_or_else(|cap| cap.exceed())
    }
}

impl Sub<&Exact> for Exact {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: &Exact) -> Self {
        self.checked_sub(rhs).unwrap_or_else(|cap| cap.exceed())
    }
}

impl SubAssign<&Exact> for Exact {
    #[inline]
    fn sub_assign(&mut self, rhs: &Exact) {
        let difference = &*self - rhs;
        self.replace(difference);
    }
}

impl SubAssign for Exact {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self -= &rhs;
    }
}

impl Mul<&Exact> for &Exact {
    type Output = Exact;
    #[inline]
    fn mul(self, rhs: &Exact) -> Exact {
        self.checked_mul(rhs).unwrap_or_else(|cap| cap.exceed())
    }
}

impl Mul for Exact {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        self.checked_mul(&rhs).unwrap_or_else(|cap| cap.exceed())
    }
}

impl Mul<&Exact> for Exact {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: &Exact) -> Self {
        self.checked_mul(rhs).unwrap_or_else(|cap| cap.exceed())
    }
}

impl Mul<i64> for &Exact {
    type Output = Exact;
    /// Integer scaling, as an engine applies a multiplicity or a retraction.
    #[inline]
    fn mul(self, k: i64) -> Exact {
        self.checked_mul_i64(k).unwrap_or_else(|cap| cap.exceed())
    }
}

impl Mul<i64> for Exact {
    type Output = Self;
    #[inline]
    fn mul(self, k: i64) -> Self {
        self.checked_mul_i64(k).unwrap_or_else(|cap| cap.exceed())
    }
}

impl Mul<isize> for &Exact {
    type Output = Exact;
    #[inline]
    fn mul(self, k: isize) -> Exact {
        self * (k as i64)
    }
}

impl Mul<isize> for Exact {
    type Output = Self;
    #[inline]
    fn mul(self, k: isize) -> Self {
        self * (k as i64)
    }
}

impl MulAssign<i64> for Exact {
    #[inline]
    fn mul_assign(&mut self, k: i64) {
        let scaled = &*self * k;
        self.replace(scaled);
    }
}

impl MulAssign<isize> for Exact {
    #[inline]
    fn mul_assign(&mut self, k: isize) {
        *self *= k as i64;
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
