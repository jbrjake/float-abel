//! The compact tier of `Exact`: a signed dyadic `m · 2^exp` whose magnitude fits in 127 bits,
//! held as an `i128` and an `i32`. Canonical form is `m` odd, or the zero `(0, 0)`, so one value
//! has one bit pattern and structural `Eq` and `Hash` are numeric. Nothing here executes a
//! floating-point instruction.
//!
//! Every operation that could leave the tier says so. [`Compact::checked_add`] and
//! [`Compact::checked_mul`] return [`Overflow::NeedsWide`] when the aligned result would not fit
//! in the width; the special counts live beside the tier in `Exact`, which never hands it a
//! non-finite double. They return [`Overflow::Exponent`] when the
//! result's exponent would leave `±EXACT_MAX_EXP`, which no tier holds. Every exponent is computed
//! in `i64` and narrowed through that one check, so nothing here can wrap. Negation is total,
//! because `|m| < 2^127` keeps `i128::MIN` out.

use core::cmp::Ordering;
use core::ops::Neg;

use super::EXACT_MAX_EXP;
use crate::ieee::Dyadic;
use crate::round;

/// Why the compact tier could not hold a result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Overflow {
    /// The magnitude would reach `2^127`: the heap tier's job.
    NeedsWide,
    /// The exponent would leave `-EXACT_MAX_EXP..=EXACT_MAX_EXP`, which no tier holds.
    Exponent,
    /// The input is NaN or ±∞, which the tier does not hold: the counts live beside it.
    #[cfg(test)]
    Special,
}

/// A canonical dyadic `m · 2^exp` with `|m| < 2^127` and `m` odd, or the zero `(0, 0)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Compact {
    m: i128,
    exp: i32,
}

/// Bits a magnitude may occupy: the `i128` less its sign bit.
const MAGNITUDE_BITS: u32 = 127;

impl Compact {
    /// The zero, which is the only zero.
    pub(crate) const ZERO: Self = Self { m: 0, exp: 0 };

    /// `m · 2^exp` whose exponent is inside the bound by construction: a double, a product of two,
    /// or an integer, none of which lies more than `2148 + 127` from the unit.
    fn within(m: i128, exp: i64) -> Self {
        let (m, exp) = normalise(m, exp);
        debug_assert!(
            bounded(exp).is_ok(),
            "Exact: an input exponent past the bound"
        );
        Self { m, exp: exp as i32 }
    }

    /// `(-1)^negative · mant · 2^exp` for `mant < 2^127`.
    fn from_dyadic(negative: bool, mant: u128, exp: i32) -> Self {
        debug_assert_eq!(mant >> MAGNITUDE_BITS, 0);
        let m = mant as i128;
        Self::within(if negative { -m } else { m }, i64::from(exp))
    }

    /// A finite double, taken apart: exactly, with a zero of either sign the zero.
    pub(crate) fn from_finite(d: Dyadic) -> Self {
        Self::from_dyadic(d.negative, u128::from(d.mant), d.exp)
    }

    /// The exact product of two finite doubles, taken apart: the full 106-bit mantissa product,
    /// without a floating-point multiply. A zero factor makes the zero.
    pub(crate) fn from_finite_product(da: Dyadic, db: Dyadic) -> Self {
        Self::from_dyadic(
            da.negative != db.negative,
            u128::from(da.mant) * u128::from(db.mant),
            da.exp + db.exp,
        )
    }

    /// The finite double `x`, exactly.
    ///
    /// # Errors
    ///
    /// NaN and ±∞ are [`Overflow::Special`]: the compact tier holds no special counts.
    #[cfg(test)]
    pub(crate) fn from_f64(x: f64) -> Result<Self, Overflow> {
        crate::ieee::decompose(x)
            .map(Self::from_finite)
            .ok_or(Overflow::Special)
    }

    /// The exact product of two finite doubles.
    ///
    /// # Errors
    ///
    /// A non-finite factor is [`Overflow::Special`]: the counts live beside the tier, not in it.
    #[cfg(test)]
    pub(crate) fn from_product(a: f64, b: f64) -> Result<Self, Overflow> {
        match (crate::ieee::decompose(a), crate::ieee::decompose(b)) {
            (Some(da), Some(db)) => Ok(Self::from_finite_product(da, db)),
            _ => Err(Overflow::Special),
        }
    }

    /// The value as an integer, if it is one that fits `i128`: the mantissa shifted up by a
    /// non-negative exponent that leaves its magnitude below `2^127`.
    pub(crate) fn to_i128(self) -> Option<i128> {
        if self.m == 0 {
            return Some(0);
        }
        if self.exp < 0 {
            return None;
        }
        raise(self.m, i64::from(self.exp)).ok()
    }

    /// The integer `k`, canonicalised: an even `k` moves its trailing zeros into the exponent.
    pub(crate) fn from_i64(k: i64) -> Self {
        Self::within(i128::from(k), 0)
    }

    /// The storage form: `m` as two's-complement limbs, low first, and the exponent.
    pub(crate) const fn parts(self) -> (u64, u64, i32) {
        (self.m as u64, (self.m >> 64) as u64, self.exp)
    }

    /// From the storage form, which is canonical by construction.
    pub(crate) fn from_parts(lo: u64, hi: u64, exp: i32) -> Self {
        let m = ((u128::from(hi) << 64) | u128::from(lo)) as i128;
        debug_assert!(
            m & 1 == 1 || (m == 0 && exp == 0),
            "Exact: a non-canonical compact value in storage"
        );
        Self { m, exp }
    }

    #[cfg(test)]
    pub(crate) fn is_zero(self) -> bool {
        self.m == 0
    }

    /// The sum, aligned to the lower exponent and canonicalised.
    ///
    /// # Errors
    ///
    /// [`Overflow::NeedsWide`] when the aligned sum's magnitude would reach `2^127`: the raised
    /// operand alone, or the sum itself. The check is on the magnitude, so `a + b` and `-a + -b`
    /// refuse together. [`Overflow::Exponent`] when the canonical sum's exponent would leave the
    /// bound, which only a sum at the bound whose trailing zeros carry it past can do.
    pub(crate) fn checked_add(self, rhs: Self) -> Result<Self, Overflow> {
        if self.m == 0 {
            return Ok(rhs);
        }
        if rhs.m == 0 {
            return Ok(self);
        }
        let (low, high) = if self.exp <= rhs.exp {
            (self, rhs)
        } else {
            (rhs, self)
        };
        let raised = raise(high.m, i64::from(high.exp) - i64::from(low.exp))?;
        let m = low.m.checked_add(raised).ok_or(Overflow::NeedsWide)?;
        if m == i128::MIN {
            return Err(Overflow::NeedsWide);
        }
        let (m, exp) = normalise(m, i64::from(low.exp));
        Ok(Self {
            m,
            exp: bounded(exp)?,
        })
    }

    /// The product. Two odd mantissas multiply to an odd one, so only a zero factor
    /// canonicalises; the exponents add.
    ///
    /// # Errors
    ///
    /// [`Overflow::NeedsWide`] when the product's magnitude would reach `2^127`, and
    /// [`Overflow::Exponent`] when its exponent would leave the bound.
    pub(crate) fn checked_mul(self, rhs: Self) -> Result<Self, Overflow> {
        let m = self.m.checked_mul(rhs.m).ok_or(Overflow::NeedsWide)?;
        let (m, exp) = normalise(m, i64::from(self.exp) + i64::from(rhs.exp));
        Ok(Self {
            m,
            exp: bounded(exp)?,
        })
    }

    /// The sign and the magnitude as two limbs, low first, for the rounding routines.
    fn magnitude(self) -> (bool, [u64; 2]) {
        let mag = self.m.unsigned_abs();
        (self.m < 0, [mag as u64, (mag >> 64) as u64])
    }

    /// The value rounded to nearest-even as a double, in integer arithmetic, with subnormals and
    /// overflow to ±∞, and whether that rounding lost anything. The zero reads `+0.0`.
    pub(crate) fn read(self) -> (f64, bool) {
        let (negative, mag) = self.magnitude();
        let (bits, inexact) = round::round(negative, &mag, self.exp, false, round::F64);
        (f64::from_bits(bits), inexact)
    }

    /// The correctly rounded quotient of the exact value by an integer, with IEEE's answers for a
    /// zero divisor: `0/0` is NaN and `x/0` is `±∞`.
    pub(crate) fn div_f64(self, n: u64) -> f64 {
        let (negative, mag) = self.magnitude();
        round::div_f64(negative, &mag, self.exp, n)
    }
}

/// `m · 2^by` for a nonzero `m`, if its magnitude still fits in 127 bits.
fn raise(m: i128, by: i64) -> Result<i128, Overflow> {
    let bits = i64::from(128 - m.unsigned_abs().leading_zeros());
    if bits + by > i64::from(MAGNITUDE_BITS) {
        return Err(Overflow::NeedsWide);
    }
    Ok(m << by as u32)
}

/// Canonical form: trailing zeros move into the exponent, and a zero mantissa is the zero form.
/// `m` is never `i128::MIN`, which no caller produces. The exponent stays in `i64`, where a shift
/// of at most 126 cannot wrap, until [`bounded`] narrows it.
fn normalise(m: i128, exp: i64) -> (i128, i64) {
    debug_assert_ne!(
        m,
        i128::MIN,
        "Exact: a 128-bit magnitude reached the compact tier"
    );
    if m == 0 {
        return (0, 0);
    }
    let tz = m.trailing_zeros();
    (m >> tz, exp + i64::from(tz))
}

/// The exponent as stored, if it is inside `-EXACT_MAX_EXP..=EXACT_MAX_EXP`.
fn bounded(exp: i64) -> Result<i32, Overflow> {
    let max = i64::from(EXACT_MAX_EXP);
    if (-max..=max).contains(&exp) {
        Ok(exp as i32)
    } else {
        Err(Overflow::Exponent)
    }
}

impl Neg for Compact {
    type Output = Self;

    /// Total: a canonical mantissa is never `i128::MIN`, and the zero stays the zero.
    fn neg(self) -> Self {
        Self {
            m: -self.m,
            exp: self.exp,
        }
    }
}

impl PartialOrd for Compact {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Compact {
    /// Numeric, without overflow: the sign decides first, then the position of the leading bit,
    /// and only a tie aligns the operands, which raises the higher one by at most the difference
    /// of their bit lengths.
    fn cmp(&self, other: &Self) -> Ordering {
        let (sa, sb) = (self.m.signum(), other.m.signum());
        match sa.cmp(&sb) {
            Ordering::Equal if sa != 0 => {
                let by_magnitude = cmp_magnitude(*self, *other);
                if sa < 0 {
                    by_magnitude.reverse()
                } else {
                    by_magnitude
                }
            }
            decided => decided,
        }
    }
}

/// The magnitudes of two nonzero values. When their leading bits sit at the same position, the
/// raised operand's bit length becomes the other's, so it fits.
fn cmp_magnitude(a: Compact, b: Compact) -> Ordering {
    let (ma, mb) = (a.m.unsigned_abs(), b.m.unsigned_abs());
    let lead = |m: u128, exp: i32| i64::from(MAGNITUDE_BITS - m.leading_zeros()) + i64::from(exp);
    match lead(ma, a.exp).cmp(&lead(mb, b.exp)) {
        Ordering::Equal if a.exp >= b.exp => (ma << (a.exp - b.exp) as u32).cmp(&mb),
        Ordering::Equal => ma.cmp(&(mb << (b.exp - a.exp) as u32)),
        decided => decided,
    }
}

#[cfg(test)]
mod exponent_tests;
#[cfg(test)]
mod tests;
