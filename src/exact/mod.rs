//! [`Exact`]: the engine weight, a 32-byte plain word. The finite part is the compact tier's
//! canonical dyadic, 127 bits of mantissa and an exponent; beside it sit three signed counts of
//! the NaN, +∞ and −∞ inputs. `Copy`, `repr(C)`, no padding, no pointer: a column of them is a
//! column of words, and a sum or a product is a fresh word, never an allocation.
//!
//! Past the cap, the operators panic with a named invariant, because an engine's `plus_equals`
//! cannot return an error; the checked methods return the same cap as [`PastCap`] and leave the
//! state as it was. The caps are the width (a magnitude at `2^127`), the exponent bound
//! ([`EXACT_MAX_EXP`]) and the counts (an `i32`, kept away from `i32::MIN` so negation is total).

mod compact;
mod convert;
#[cfg(feature = "differential")]
mod differential;
mod ops;
mod ring;
#[cfg(test)]
pub(crate) mod testing;
mod wire;

use core::cmp::Ordering;
use core::fmt;

use compact::{Compact, Overflow};

use crate::fixed::Hex;
use crate::ieee::{self, Special};

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
    /// The finite part's magnitude would reach `2^127`: the sum or product of the mantissas no
    /// longer fits the 127-bit width at the aligned exponent, or a `Wide` or `Fixed` state being
    /// converted spans more than 127 bits.
    Width,
    /// The exponent would leave `-EXACT_MAX_EXP..=EXACT_MAX_EXP`.
    Exponent,
    /// A special count would leave `i32` (or become `i32::MIN`), whether in an operation or in a
    /// `Wide` counter being converted, or a product would give a special a non-integer amount, as
    /// `2.5 × NaN` does.
    Count,
}

impl fmt::Display for PastCap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Width => "the mantissa needs more than the 127-bit width",
            Self::Exponent => "the exponent would leave ±EXACT_MAX_EXP, 2^30",
            Self::Count => "a NaN or ±∞ count would leave i32 or stop being an integer",
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

/// An exact abelian group of every `f64`, specials included, with `Mul<i64>` and `Mul<Exact>` on
/// top, in a 32-byte plain word: the engine weight.
///
/// The finite part is a dyadic `m · 2^exp` in canonical form (`m` odd, or the zero form), and the
/// three counts say how many NaN, +∞ and −∞ inputs are in the multiset, signed so that a
/// retraction subtracts. Structural `Eq` is numeric equality, and `Hash` and `Ord` agree with it.
/// Projection happens once, at read: a nonzero NaN count or both infinities read NaN, one infinity
/// reads by the sign of its count, and otherwise the finite part rounds to nearest-even.
///
/// The operators are exact or they panic with the named cap; the checked methods return the same
/// cap as a [`PastCap`] and leave the state as it was. No operation executes a floating-point
/// instruction, and no operation allocates. A [`Wide`](crate::Wide) or [`Fixed`](crate::Fixed)
/// state converts in through `TryFrom`, refused as [`PastCap::Width`] when its set bits span more
/// than the width, and an `Exact` converts out through `Wide::try_from`, refused where a product
/// would be.
///
/// The layout is `repr(C)` with no padding: two limbs, the exponent, three counts. The bytes are
/// the little-endian wire form on a little-endian machine, so an engine may cast a column of them
/// in place. A byte pattern with an even mantissa is not a value, so bytes from outside go through
/// the validating decoder, [`Exact::from_bytes`].
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Exact {
    lo: u64,
    hi: u64,
    exp: i32,
    nan: i32,
    pos_inf: i32,
    neg_inf: i32,
}

const _: () = assert!(
    core::mem::size_of::<Exact>() == 32 && core::mem::align_of::<Exact>() == 8,
    "Exact's layout moved"
);

impl Exact {
    /// The zero, which is the only zero.
    pub const ZERO: Self = Self {
        lo: 0,
        hi: 0,
        exp: 0,
        nan: 0,
        pos_inf: 0,
        neg_inf: 0,
    };

    /// The finite part in its working form.
    fn finite(self) -> Compact {
        Compact::from_parts(self.lo, self.hi, self.exp)
    }

    /// The counts as `[n, p, q]`.
    const fn counts(self) -> [i32; 3] {
        [self.nan, self.pos_inf, self.neg_inf]
    }

    fn assemble(finite: Compact, counts: [i32; 3]) -> Self {
        let (lo, hi, exp) = finite.parts();
        Self {
            lo,
            hi,
            exp,
            nan: counts[0],
            pos_inf: counts[1],
            neg_inf: counts[2],
        }
    }

    fn cap(overflow: Overflow) -> PastCap {
        match overflow {
            Overflow::NeedsWide => PastCap::Width,
            Overflow::Exponent => PastCap::Exponent,
            #[cfg(test)]
            Overflow::Special => unreachable!("Exact counts a special before the tier sees it"),
        }
    }

    /// A count from a wider intermediate: inside `i32`, and never `i32::MIN`, so that negation
    /// stays total.
    pub(crate) fn narrow(v: i128) -> Result<i32, PastCap> {
        match i32::try_from(v) {
            Ok(c) if c != i32::MIN => Ok(c),
            _ => Err(PastCap::Count),
        }
    }

    /// One count of a special.
    fn one(s: Special) -> Self {
        let mut v = Self::ZERO;
        match s {
            Special::Nan => v.nan = 1,
            Special::PosInf => v.pos_inf = 1,
            Special::NegInf => v.neg_inf = 1,
        }
        v
    }

    /// A fresh state holding `x`: a finite value exactly, or one count of its special.
    #[must_use]
    pub fn from_f64(x: f64) -> Self {
        match ieee::decompose(x) {
            Some(d) => Self::assemble(Compact::from_finite(d), [0; 3]),
            None => Self::one(ieee::special(x)),
        }
    }

    /// A fresh state holding the exact product of two doubles, the full 106-bit mantissa product,
    /// without a floating-point multiply: a product deposit on the zero. Total, and bit for bit
    /// what [`crate::Wide::deposit_product`] deposits.
    ///
    /// A special factor follows the NaN algebra of the design with a unit count, as on `Wide`: a
    /// NaN factor is one NaN; two infinities are one +∞ when their signs agree and one −∞ when
    /// they differ; an infinity times a nonzero finite factor is `±1` on that infinity's count by
    /// the finite factor's sign, so a negative factor flips it through the projection rule; and a
    /// zero factor makes the exact zero whatever the other factor is, as bilinearity requires. This
    /// is a row's contribution, counted once per row; the ring product of two weights,
    /// [`Exact::checked_mul`], is the one that scales a count by the other weight.
    #[must_use]
    pub fn product(a: f64, b: f64) -> Self {
        match (ieee::factor(a), ieee::factor(b)) {
            (ieee::Factor::Finite(da), ieee::Factor::Finite(db)) => {
                Self::assemble(Compact::from_finite_product(da, db), [0; 3])
            }
            (ieee::Factor::Finite(d), ieee::Factor::Special(_))
            | (ieee::Factor::Special(_), ieee::Factor::Finite(d))
                if d.mant == 0 =>
            {
                Self::ZERO
            }
            (ieee::Factor::Finite(d), ieee::Factor::Special(s))
            | (ieee::Factor::Special(s), ieee::Factor::Finite(d)) => {
                let (which, by) = ieee::special_times_finite(s, d.negative);
                let mut counts = [0i32; 3];
                counts[which] = by as i32;
                Self::assemble(Compact::ZERO, counts)
            }
            (ieee::Factor::Special(s), ieee::Factor::Special(t)) => {
                let mut counts = [0i32; 3];
                counts[ieee::special_times_special(s, t)] = 1;
                Self::assemble(Compact::ZERO, counts)
            }
        }
    }

    /// The sum, or the cap it would cross. Neither operand changes either way.
    ///
    /// # Errors
    ///
    /// [`PastCap::Width`] when the aligned sum of the finite parts needs more than the width,
    /// [`PastCap::Exponent`] when its exponent would leave the bound, and [`PastCap::Count`] when a
    /// count would leave `i32`.
    pub fn checked_add(&self, rhs: &Self) -> Result<Self, PastCap> {
        let finite = self.finite().checked_add(rhs.finite()).map_err(Self::cap)?;
        let mut counts = [0i32; 3];
        for (c, (a, b)) in counts
            .iter_mut()
            .zip(self.counts().into_iter().zip(rhs.counts()))
        {
            *c = Self::narrow(i128::from(a) + i128::from(b))?;
        }
        Ok(Self::assemble(finite, counts))
    }

    /// The difference, as [`Exact::checked_add`] of the negation.
    ///
    /// # Errors
    ///
    /// As [`Exact::checked_add`].
    pub fn checked_sub(&self, rhs: &Self) -> Result<Self, PastCap> {
        self.checked_add(&-*rhs)
    }

    /// The product, or the cap it would cross: the finite parts multiply exactly, and the counts
    /// follow the monoid algebra of DESIGN.md § NaN and ±∞ with integer amounts, so `0 · ∞` and
    /// `0 · NaN` are the zero, `(+∞)(−∞)` is one −∞, and `−1 · (+∞)` is a count of −1 on +∞.
    ///
    /// # Errors
    ///
    /// [`PastCap::Width`] or [`PastCap::Exponent`] from the finite product, and [`PastCap::Count`]
    /// when a special amount would leave `i32` or stop being an integer, as `2.5 · NaN` would.
    pub fn checked_mul(&self, rhs: &Self) -> Result<Self, PastCap> {
        ring::product(*self, *rhs)
    }

    /// Integer scaling of the finite part and the counts alike, as an engine applies a
    /// multiplicity or a retraction, or the cap it would cross.
    ///
    /// # Errors
    ///
    /// As [`Exact::checked_mul`] by the integer.
    pub fn checked_mul_i64(&self, k: i64) -> Result<Self, PastCap> {
        let finite = self
            .finite()
            .checked_mul(Compact::from_i64(k))
            .map_err(Self::cap)?;
        let mut counts = [0i32; 3];
        for (c, a) in counts.iter_mut().zip(self.counts()) {
            *c = Self::narrow(i128::from(a) * i128::from(k))?;
        }
        Ok(Self::assemble(finite, counts))
    }

    /// Adds a double, or leaves the state as it was.
    ///
    /// # Errors
    ///
    /// As [`Exact::checked_add`].
    pub fn checked_deposit(&mut self, x: f64) -> Result<(), PastCap> {
        *self = self.checked_add(&Self::from_f64(x))?;
        Ok(())
    }

    /// Adds the exact product of two doubles, or leaves the state as it was. This is the join-fold
    /// and dot-product shape.
    ///
    /// # Errors
    ///
    /// As [`Exact::checked_add`] of [`Exact::product`].
    pub fn checked_deposit_product(&mut self, a: f64, b: f64) -> Result<(), PastCap> {
        *self = self.checked_add(&Self::product(a, b))?;
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

    /// Whether the state is the zero: no finite part and no counts.
    #[must_use]
    pub fn is_zero(&self) -> bool {
        *self == Self::ZERO
    }

    /// Whether every count is zero, so a read rounds the finite part. The finite part may still
    /// round to `±∞` when it lies past `f64::MAX`.
    #[must_use]
    pub const fn is_finite(&self) -> bool {
        self.nan == 0 && self.pos_inf == 0 && self.neg_inf == 0
    }

    /// The counts `(n, p, q)`: how many NaN, +∞ and −∞ inputs are in the multiset, each negative
    /// when more were retracted than inserted.
    #[must_use]
    pub const fn specials(&self) -> (i32, i32, i32) {
        (self.nan, self.pos_inf, self.neg_inf)
    }

    fn projection(&self) -> Option<Special> {
        ieee::project(
            i64::from(self.nan),
            i64::from(self.pos_inf),
            i64::from(self.neg_inf),
        )
    }

    /// The value as a double: the special projection if the counts say so, else the finite part
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

    fn read(&self) -> (f64, bool) {
        match self.projection() {
            Some(special) => (special.to_f64(), false),
            None => self.finite().read(),
        }
    }

    /// The value as a single: the special projection if the counts say so, else the finite part
    /// rounded once, to nearest-even in integer arithmetic, with subnormals and overflow to `±∞`.
    /// Never via `f64`: a finite part that `to_f64` rounds onto an `f32` tie would then round
    /// the wrong way. A nonzero finite part below half the smallest `f32` subnormal reads a zero
    /// of its sign.
    #[must_use]
    pub fn to_f32(&self) -> f32 {
        match self.projection() {
            Some(special) => special.to_f32(),
            None => self.finite().read_f32(),
        }
    }

    /// The correctly rounded quotient of the exact value by an integer: what an `AVG` calls, so
    /// every engine divides the same exact numerator once. A special projection is returned as it
    /// is, as IEEE divides it; a finite part follows IEEE for a zero divisor too, `0/0` NaN and
    /// `x/0` `±∞`.
    #[must_use]
    pub fn div_f64(&self, n: u64) -> f64 {
        match self.projection() {
            Some(special) => special.to_f64(),
            None => self.finite().div_f64(n),
        }
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
        write!(
            f,
            "Exact({about}{value:e}; nan {}, +inf {}, -inf {}; {:?}:{:?} · 2^{})",
            self.nan,
            self.pos_inf,
            self.neg_inf,
            Hex(self.hi),
            Hex(self.lo),
            self.exp
        )
    }
}

impl PartialOrd for Exact {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Exact {
    /// Numeric on the finite part, then `(n, p, q)` lexicographic: a total order.
    fn cmp(&self, other: &Self) -> Ordering {
        self.finite()
            .cmp(&other.finite())
            .then_with(|| self.counts().cmp(&other.counts()))
    }
}
