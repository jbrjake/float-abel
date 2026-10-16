//! The read surface of [`Exact`] over both forms: the predicates on the state, the special
//! projection, then the finite part rounded once in integer arithmetic, through the same `round`
//! kernel on the compact word's two limbs or the heap block's run, so a value reads the same bits
//! whichever form holds it.

use super::Exact;
use crate::ieee::{self, Special};

impl Exact {
    /// Whether the value is held in the compact form: a finite part of at most 127 bits and
    /// `i32` counts, in the word itself. Every value is, without `alloc`; with it, a value past
    /// that is in the heap form, and comes back as soon as an operation's result fits again.
    #[must_use]
    #[inline]
    pub fn is_compact(&self) -> bool {
        !self.is_heap()
    }

    /// Whether the state is the zero: no finite part and no amounts. The zero is the compact
    /// zero word, so this is a test of the bits.
    #[must_use]
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.lo == 0
            && self.hi == 0
            && self.exp == 0
            && self.nan == 0
            && self.pos_inf == 0
            && self.neg_inf == 0
    }

    /// Whether every amount is zero, so a read rounds the finite part. The finite part may still
    /// round to `±∞` when it lies past `f64::MAX`.
    #[must_use]
    #[inline]
    pub fn is_finite(&self) -> bool {
        #[cfg(feature = "alloc")]
        if self.is_heap() {
            let p = self.parts();
            return (0..3).all(|i| p.amount(i).is_zero());
        }
        self.nan == 0 && self.pos_inf == 0 && self.neg_inf == 0
    }

    /// The counts `(n, p, q)`: how many NaN, +∞ and −∞ inputs are in the multiset, each negative
    /// when more were retracted than inserted, if every amount is an integer an `i64` holds. With
    /// `alloc` a product can give a special a dyadic amount, as `2.5 · NaN` does, or an integer
    /// past `i64`, and then this is `None`; the projection a read applies needs only the sign.
    #[must_use]
    #[inline]
    pub fn specials(&self) -> Option<(i64, i64, i64)> {
        #[cfg(feature = "alloc")]
        if self.is_heap() {
            let p = self.parts();
            return Some((
                p.amount(0).to_i64()?,
                p.amount(1).to_i64()?,
                p.amount(2).to_i64()?,
            ));
        }
        Some((
            i64::from(self.nan),
            i64::from(self.pos_inf),
            i64::from(self.neg_inf),
        ))
    }

    pub(super) fn projection(&self) -> Option<Special> {
        #[cfg(feature = "alloc")]
        if self.is_heap() {
            let p = self.parts();
            let sign = |i: usize| {
                let a = p.amount(i);
                if a.is_zero() {
                    0
                } else if a.neg {
                    -1
                } else {
                    1
                }
            };
            return ieee::project(sign(0), sign(1), sign(2));
        }
        ieee::project(
            i64::from(self.nan),
            i64::from(self.pos_inf),
            i64::from(self.neg_inf),
        )
    }

    /// The value as a double: the special projection if the amounts say so, else the finite part
    /// rounded to nearest-even in integer arithmetic, with subnormals and overflow to ±∞. The zero
    /// reads `+0.0`.
    #[must_use]
    pub fn to_f64(&self) -> f64 {
        self.read().0
    }

    /// The value as a double if it is exactly one, else `None`. A special projection passes
    /// through: nothing was rounded to reach it.
    #[must_use]
    pub fn try_to_f64(&self) -> Option<f64> {
        match self.read() {
            (value, false) => Some(value),
            (_, true) => None,
        }
    }

    pub(super) fn read(&self) -> (f64, bool) {
        if let Some(special) = self.projection() {
            return (special.to_f64(), false);
        }
        #[cfg(feature = "alloc")]
        if self.is_heap() {
            return self.parts().finite().read();
        }
        self.finite().read()
    }

    /// The value as a single: the special projection if the amounts say so, else the finite part
    /// rounded once, to nearest-even in integer arithmetic, with subnormals and overflow to `±∞`.
    /// Never via `f64`: a finite part that `to_f64` rounds onto an `f32` tie would then round
    /// the wrong way. A nonzero finite part below half the smallest `f32` subnormal reads a zero
    /// of its sign.
    #[must_use]
    pub fn to_f32(&self) -> f32 {
        if let Some(special) = self.projection() {
            return special.to_f32();
        }
        #[cfg(feature = "alloc")]
        if self.is_heap() {
            return self.parts().finite().read_f32();
        }
        self.finite().read_f32()
    }

    /// The correctly rounded quotient of the exact value by an integer: what an `AVG` calls, so
    /// every engine divides the same exact numerator once. A special projection is returned as it
    /// is, as IEEE divides it; a finite part follows IEEE for a zero divisor too, `0/0` NaN and
    /// `x/0` `±∞`.
    #[must_use]
    pub fn div_f64(&self, n: u64) -> f64 {
        if let Some(special) = self.projection() {
            return special.to_f64();
        }
        #[cfg(feature = "alloc")]
        if self.is_heap() {
            return self.parts().finite().div_f64(n);
        }
        self.finite().div_f64(n)
    }
}
