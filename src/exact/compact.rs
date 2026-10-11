//! The compact tier of `Exact`: a signed dyadic `m · 2^exp` whose magnitude fits in 127 bits,
//! held as an `i128` and an `i32`. Canonical form is `m` odd, or the zero `(0, 0)`, so one value
//! has one bit pattern and structural `Eq` and `Hash` are numeric. Nothing here executes a
//! floating-point instruction.
//!
//! Every operation that could leave the tier says so. [`Compact::checked_add`] and
//! [`Compact::checked_mul`] return [`NeedsWide`] when the aligned result would not fit, and a
//! non-finite double is `NeedsWide` at construction, because specials are counted on the heap
//! tier. The caller promotes. Negation is total, because `|m| < 2^127` keeps `i128::MIN` out.

use core::cmp::Ordering;
use core::ops::Neg;

use crate::ieee;
use crate::round;

/// The compact tier cannot hold the result: the caller promotes to the heap tier.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NeedsWide;

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

    /// `m · 2^exp` in canonical form: trailing zeros move into the exponent, and a zero mantissa
    /// is the zero form. `m` is never `i128::MIN`, which no caller produces.
    fn canonical(m: i128, exp: i32) -> Self {
        debug_assert_ne!(
            m,
            i128::MIN,
            "Exact: a 128-bit magnitude reached the compact tier"
        );
        if m == 0 {
            return Self::ZERO;
        }
        let tz = m.trailing_zeros();
        Self {
            m: m >> tz,
            exp: exp + tz as i32,
        }
    }

    /// `(-1)^negative · mant · 2^exp` for `mant < 2^127`.
    fn from_dyadic(negative: bool, mant: u128, exp: i32) -> Self {
        debug_assert_eq!(mant >> MAGNITUDE_BITS, 0);
        let m = mant as i128;
        Self::canonical(if negative { -m } else { m }, exp)
    }

    /// The finite double `x`, exactly. A zero of either sign is the zero.
    ///
    /// # Errors
    ///
    /// NaN and ±∞ are [`NeedsWide`]: the compact tier holds no special counts.
    pub(crate) fn from_f64(x: f64) -> Result<Self, NeedsWide> {
        let d = ieee::decompose(x).ok_or(NeedsWide)?;
        Ok(Self::from_dyadic(d.negative, u128::from(d.mant), d.exp))
    }

    /// The exact product of two finite doubles: the full 106-bit mantissa product, without a
    /// floating-point multiply. A zero factor makes the zero.
    ///
    /// # Errors
    ///
    /// A non-finite factor is [`NeedsWide`]: the special algebra lives on the heap tier.
    pub(crate) fn from_product(a: f64, b: f64) -> Result<Self, NeedsWide> {
        let da = ieee::decompose(a).ok_or(NeedsWide)?;
        let db = ieee::decompose(b).ok_or(NeedsWide)?;
        Ok(Self::from_dyadic(
            da.negative != db.negative,
            u128::from(da.mant) * u128::from(db.mant),
            da.exp + db.exp,
        ))
    }

    /// The integer `k`, canonicalised: an even `k` moves its trailing zeros into the exponent.
    pub(crate) fn from_i64(k: i64) -> Self {
        Self::canonical(i128::from(k), 0)
    }

    /// The storage form: `m` as two's-complement limbs, low first, and the exponent.
    pub(crate) fn parts(self) -> (u64, u64, i32) {
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

    pub(crate) fn is_zero(self) -> bool {
        self.m == 0
    }

    /// The sum, aligned to the lower exponent and canonicalised.
    ///
    /// # Errors
    ///
    /// [`NeedsWide`] when the aligned sum's magnitude would reach `2^127`: the raised operand
    /// alone, or the sum itself. The check is on the magnitude, so `a + b` and `-a + -b` refuse
    /// together.
    pub(crate) fn checked_add(self, rhs: Self) -> Result<Self, NeedsWide> {
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
        let m = low.m.checked_add(raised).ok_or(NeedsWide)?;
        if m == i128::MIN {
            return Err(NeedsWide);
        }
        Ok(Self::canonical(m, low.exp))
    }

    /// The product. Two odd mantissas multiply to an odd one, so only a zero factor
    /// canonicalises.
    ///
    /// # Errors
    ///
    /// [`NeedsWide`] when the product's magnitude would reach `2^127`.
    pub(crate) fn checked_mul(self, rhs: Self) -> Result<Self, NeedsWide> {
        let m = self.m.checked_mul(rhs.m).ok_or(NeedsWide)?;
        // Exponents add; `i32` holds some two million nested products of the smallest double,
        // past which they wrap in release and hit this assertion in debug builds.
        let (exp, overflow) = self.exp.overflowing_add(rhs.exp);
        debug_assert!(!overflow, "Exact: exponent past i32 in a product");
        Ok(Self::canonical(m, exp))
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
fn raise(m: i128, by: i64) -> Result<i128, NeedsWide> {
    let bits = i64::from(128 - m.unsigned_abs().leading_zeros());
    if bits + by > i64::from(MAGNITUDE_BITS) {
        return Err(NeedsWide);
    }
    Ok(m << by as u32)
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
mod tests;
