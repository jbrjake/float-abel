//! [`Exact`]: the engine weight, a dyadic held in the smallest tier that represents it. The
//! compact tier is 127 bits of mantissa inline, in `compact`; the heap tier, behind the `alloc`
//! feature, is where a wider mantissa and the special counts go. This tree holds the compact tier,
//! so the inline width is the cap on every feature set.
//!
//! The tier enum sits behind a newtype because an enum's variant fields are as visible as the
//! enum, and a hand-built `Compact { lo: 2, .. }` would be a non-canonical value, which structural
//! `Eq` cannot afford. The layout is load-bearing: a struct variant lets the discriminant share the
//! word that `exp: i32` half fills, so the whole is 24 bytes where a nested struct or an `i128`
//! would make it 32.
//!
//! Past the cap, the operators panic with a named invariant, because an engine's `plus_equals`
//! cannot return an error; the checked methods return the same cap as [`PastCap`] and leave the
//! state as it was.

mod compact;
mod ops;
#[cfg(test)]
pub(crate) mod testing;

use core::cmp::Ordering;
use core::fmt;

use compact::{Compact, Overflow};

use crate::fixed::Hex;

/// The exponent bound: every held value is `m · 2^exp` with `|exp| ≤ EXACT_MAX_EXP`, checked
/// after every operation that moves an exponent, in every build. A sum's exponent stays inside its
/// inputs' range plus the mantissa width, and a product of two doubles lies within `±2148`, so only
/// a chain of nested products moves an exponent far: roughly a million products of the smallest
/// subnormal reach the bound. Past it is the cap. The number keeps every intermediate inside `i64`
/// and every stored exponent inside `i32`, with room to spare on both sides.
pub const EXACT_MAX_EXP: i32 = 1 << 30;

/// The one cap on [`Exact`]: which bound a result would cross. The operators panic with it, and
/// the checked methods return it with the state untouched.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PastCap {
    /// The mantissa would need more width than this build holds. With no heap tier, that is the
    /// inline 127 bits.
    Width,
    /// The exponent would leave `-EXACT_MAX_EXP..=EXACT_MAX_EXP`.
    Exponent,
    /// A NaN or ±∞ arrived, and this build has no heap tier, which is where specials are counted.
    Special,
}

impl fmt::Display for PastCap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Width => "the mantissa needs more width than this build holds",
            Self::Exponent => "the exponent would leave ±EXACT_MAX_EXP, 2^30",
            Self::Special => "a NaN or ±∞ needs the heap tier, which this build does not have",
        })
    }
}

#[cfg(feature = "std")]
impl std::error::Error for PastCap {}

impl PastCap {
    /// The named invariant every operator ends in past the cap.
    #[cold]
    #[inline(never)]
    pub(crate) fn exceed(self) -> ! {
        panic!("Exact: past the cap: {self}")
    }
}

/// An exact abelian group of `f64` sums with a ring on top, in 24 bytes: the engine weight.
///
/// A value is a dyadic `m · 2^exp` in canonical form, so structural `Eq` is numeric equality and
/// `Hash` and `Ord` agree with it. The operators are exact or they panic with the named cap; the
/// checked methods ([`Exact::checked_add`] and its kin) return the same cap as a [`PastCap`] and
/// leave the state as it was. On a build without the heap tier the cap is the inline 127-bit
/// width, and a NaN or ±∞ input is past it. No operation executes a floating-point instruction.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Exact(Tier);

#[derive(Clone, PartialEq, Eq, Hash)]
enum Tier {
    /// `m = (hi:lo)` as a signed 128-bit integer and the value `m · 2^exp`, with `m` odd, or the
    /// zero `(0, 0, 0)`.
    Compact { lo: u64, hi: u64, exp: i32 },
}

const _: () = assert!(
    core::mem::size_of::<Exact>() == 24,
    "Exact's layout moved: a tier moved"
);

impl Exact {
    /// The zero, which is the only zero.
    pub const ZERO: Self = Self::from_compact(Compact::ZERO);

    const fn from_compact(c: Compact) -> Self {
        let (lo, hi, exp) = c.parts();
        Self(Tier::Compact { lo, hi, exp })
    }

    /// The value in its working form.
    fn compact(&self) -> Compact {
        let Tier::Compact { lo, hi, exp } = self.0;
        Compact::from_parts(lo, hi, exp)
    }

    /// What the compact tier's refusal means on this build: with no heap tier, each is the cap.
    fn cap(overflow: Overflow) -> PastCap {
        match overflow {
            Overflow::NeedsWide => PastCap::Width,
            Overflow::Exponent => PastCap::Exponent,
            Overflow::Special => PastCap::Special,
        }
    }

    /// A fresh state holding `x`: the checked form of `From<f64>`.
    ///
    /// # Errors
    ///
    /// [`PastCap::Special`] for NaN or ±∞ on a build without the heap tier.
    pub fn try_from_f64(x: f64) -> Result<Self, PastCap> {
        Compact::from_f64(x)
            .map(Self::from_compact)
            .map_err(Self::cap)
    }

    /// A fresh state holding the exact product of two doubles, the full 106-bit mantissa product,
    /// without a floating-point multiply: the checked form of a product deposit on the zero.
    ///
    /// # Errors
    ///
    /// [`PastCap::Special`] for a non-finite factor on a build without the heap tier.
    pub fn checked_product(a: f64, b: f64) -> Result<Self, PastCap> {
        Compact::from_product(a, b)
            .map(Self::from_compact)
            .map_err(Self::cap)
    }

    /// The sum, or the cap it would cross. Neither operand changes either way.
    ///
    /// # Errors
    ///
    /// [`PastCap::Width`] when the aligned sum needs more than the inline width on a build without
    /// the heap tier, and [`PastCap::Exponent`] when its exponent would leave the bound.
    pub fn checked_add(&self, rhs: &Self) -> Result<Self, PastCap> {
        self.compact()
            .checked_add(rhs.compact())
            .map(Self::from_compact)
            .map_err(Self::cap)
    }

    /// The difference, as [`Exact::checked_add`] of the negation.
    ///
    /// # Errors
    ///
    /// As [`Exact::checked_add`].
    pub fn checked_sub(&self, rhs: &Self) -> Result<Self, PastCap> {
        self.compact()
            .checked_add(-rhs.compact())
            .map(Self::from_compact)
            .map_err(Self::cap)
    }

    /// The product, or the cap it would cross.
    ///
    /// # Errors
    ///
    /// [`PastCap::Width`] when the product's mantissa needs more than the inline width on a build
    /// without the heap tier, and [`PastCap::Exponent`] when its exponent would leave the bound.
    pub fn checked_mul(&self, rhs: &Self) -> Result<Self, PastCap> {
        self.compact()
            .checked_mul(rhs.compact())
            .map(Self::from_compact)
            .map_err(Self::cap)
    }

    /// Integer scaling, as an engine applies a multiplicity or a retraction, or the cap it would
    /// cross.
    ///
    /// # Errors
    ///
    /// As [`Exact::checked_mul`].
    pub fn checked_mul_i64(&self, k: i64) -> Result<Self, PastCap> {
        self.compact()
            .checked_mul(Compact::from_i64(k))
            .map(Self::from_compact)
            .map_err(Self::cap)
    }

    /// Adds a double, or leaves the state as it was.
    ///
    /// # Errors
    ///
    /// As [`Exact::try_from_f64`] then [`Exact::checked_add`].
    pub fn checked_deposit(&mut self, x: f64) -> Result<(), PastCap> {
        *self = self.checked_add(&Self::try_from_f64(x)?)?;
        Ok(())
    }

    /// Adds the exact product of two doubles, or leaves the state as it was. This is the join-fold
    /// and dot-product shape.
    ///
    /// # Errors
    ///
    /// As [`Exact::checked_product`] then [`Exact::checked_add`].
    pub fn checked_deposit_product(&mut self, a: f64, b: f64) -> Result<(), PastCap> {
        *self = self.checked_add(&Self::checked_product(a, b)?)?;
        Ok(())
    }

    /// Adds a double.
    ///
    /// # Panics
    ///
    /// With the named cap where [`Exact::checked_deposit`] would return it.
    pub fn deposit(&mut self, x: f64) {
        self.checked_deposit(x).unwrap_or_else(|cap| cap.exceed());
    }

    /// Adds the exact product of two doubles.
    ///
    /// # Panics
    ///
    /// With the named cap where [`Exact::checked_deposit_product`] would return it.
    pub fn deposit_product(&mut self, a: f64, b: f64) {
        self.checked_deposit_product(a, b)
            .unwrap_or_else(|cap| cap.exceed());
    }

    /// Whether the state is the zero.
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.compact().is_zero()
    }

    /// Whether the value sits on the compact tier: 127 bits of mantissa inline. On a build without
    /// the heap tier, every value does.
    #[must_use]
    pub fn is_compact(&self) -> bool {
        matches!(self.0, Tier::Compact { .. })
    }

    /// Whether no special is counted, so a read rounds the finite value. The compact tier holds
    /// no specials, so every compact value is finite.
    #[must_use]
    pub fn is_finite(&self) -> bool {
        self.is_compact()
    }

    /// The value rounded to nearest-even as a double, in integer arithmetic, with subnormals and
    /// overflow to ±∞. The zero reads `+0.0`.
    #[must_use]
    pub fn to_f64(&self) -> f64 {
        self.read().0
    }

    /// The value as a double if it is exactly one, else `None`.
    #[must_use]
    pub fn try_to_f64(&self) -> Option<f64> {
        match self.read() {
            (value, false) => Some(value),
            (_, true) => None,
        }
    }

    fn read(&self) -> (f64, bool) {
        self.compact().read()
    }

    /// The correctly rounded quotient of the exact value by an integer: what an `AVG` calls, so
    /// every engine divides the same exact numerator once. Division by zero follows IEEE: `0/0`
    /// is NaN and `x/0` is `±∞`.
    #[must_use]
    pub fn div_f64(&self, n: u64) -> f64 {
        self.compact().div_f64(n)
    }
}

impl Default for Exact {
    fn default() -> Self {
        Self::ZERO
    }
}

impl fmt::Debug for Exact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (value, inexact) = self.read();
        let about = if inexact { "≈" } else { "" };
        let Tier::Compact { lo, hi, exp } = self.0;
        write!(
            f,
            "Exact({about}{value:e}; compact {:?}:{:?} · 2^{exp})",
            Hex(hi),
            Hex(lo)
        )
    }
}

impl PartialOrd for Exact {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Exact {
    /// Numeric, and total.
    fn cmp(&self, other: &Self) -> Ordering {
        self.compact().cmp(&other.compact())
    }
}
