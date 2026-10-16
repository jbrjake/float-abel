//! The operator surface of [`Wide`]: the group operators on the limbs and the counters alike, so
//! an engine's `plus_equals`, `negate` and `Multiply` all reach the same arithmetic, and the
//! constructors from a double and from an integer. Every one asserts the headroom and the counter
//! budgets in every build, because a wrapped sum is silently wrong.

use core::iter::Sum;
use core::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

use super::{Wide, LIMBS};
use crate::limbs;

impl From<f64> for Wide {
    /// A fresh state holding `x`: its finite value, or one count of its special.
    fn from(x: f64) -> Self {
        let mut w = Self::ZERO;
        w.deposit(x);
        w
    }
}

impl AddAssign for Wide {
    fn add_assign(&mut self, rhs: Self) {
        let overflow = limbs::add_assign(&mut self.limbs, &rhs.limbs);
        assert!(!overflow, "Wide: headroom exhausted in an add");
        for (s, &r) in self.specials.iter_mut().zip(&rhs.specials) {
            let (v, overflow) = s.overflowing_add(r);
            assert!(!overflow, "Wide: a special counter overflowed in an add");
            *s = v;
        }
    }
}

impl Add for Wide {
    type Output = Self;
    fn add(mut self, rhs: Self) -> Self {
        self += rhs;
        self
    }
}

impl Neg for Wide {
    type Output = Self;
    fn neg(mut self) -> Self {
        let overflow = limbs::negate(&mut self.limbs);
        assert!(!overflow, "Wide: headroom exhausted in a negation");
        for s in &mut self.specials {
            let (v, overflow) = s.overflowing_neg();
            assert!(
                !overflow,
                "Wide: a special counter overflowed in a negation"
            );
            *s = v;
        }
        self
    }
}

impl SubAssign for Wide {
    fn sub_assign(&mut self, rhs: Self) {
        *self += -rhs;
    }
}

impl Sub for Wide {
    type Output = Self;
    fn sub(mut self, rhs: Self) -> Self {
        self -= rhs;
        self
    }
}

impl Mul<i64> for Wide {
    type Output = Self;

    /// Integer scaling of the finite part and the counters alike, as an engine applies a
    /// multiplicity or a retraction.
    fn mul(self, k: i64) -> Self {
        let mut mag = [0u64; LIMBS];
        let negative = limbs::abs_into(&self.limbs, &mut mag);
        let carry = limbs::mul_assign_u64(&mut mag, k.unsigned_abs());
        assert!(
            carry == 0 && !limbs::is_negative(&mag),
            "Wide: headroom exhausted in a scalar multiply"
        );
        if negative != (k < 0) {
            limbs::negate(&mut mag);
        }
        let mut specials = [0i64; 3];
        for (s, &c) in specials.iter_mut().zip(&self.specials) {
            let (v, overflow) = c.overflowing_mul(k);
            assert!(
                !overflow,
                "Wide: a special counter overflowed in a scalar multiply"
            );
            *s = v;
        }
        Self {
            limbs: mag,
            specials,
        }
    }
}

impl Mul<isize> for Wide {
    type Output = Self;
    fn mul(self, k: isize) -> Self {
        self * (k as i64)
    }
}

impl Sum for Wide {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, Add::add)
    }
}

impl<'a> Sum<&'a Self> for Wide {
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, |acc, x| acc + *x)
    }
}
