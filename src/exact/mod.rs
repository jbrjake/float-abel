//! [`Exact`]: the engine weight, a 32-byte word in two forms. The compact form is the tier's
//! canonical dyadic, 127 bits of mantissa and an exponent, beside three signed `i32` counts of
//! the NaN, +∞ and −∞ inputs: `repr(C)`, no padding, no pointer. Behind the `alloc` feature a
//! result past that width promotes to the heap form, the same word tagged in the mantissa's
//! odd-only niche and holding one allocation of normalised limbs with dyadic special amounts, and
//! demotes after every operation that fits again, so the canonical form is the smallest form that
//! holds the value and structural `Eq` stays numeric.
//!
//! Past a cap, the operators panic with a named invariant, because an engine's `plus_equals`
//! cannot return an error; the checked methods return the same cap as [`PastCap`] and leave the
//! state as it was. With `alloc` the caps are the limb cap ([`EXACT_MAX_LIMBS`]) and the exponent
//! bound ([`EXACT_MAX_EXP`]); without it the width (a magnitude at `2^127`) and the counts (an
//! `i32`, kept away from `i32::MIN` so negation is total) cap the compact form instead.

#[cfg(feature = "alloc")]
mod big;
mod compact;
mod convert;
#[cfg(feature = "dbsp")]
mod dbsp;
#[cfg(feature = "differential")]
mod differential;
mod form;
#[cfg(feature = "alloc")]
mod heap;
mod ops;
mod read;
mod ring;
#[cfg(test)]
pub(crate) mod testing;
#[cfg(feature = "alloc")]
mod value;
mod wire;

use core::fmt;

use compact::{Compact, Overflow};
#[cfg(all(feature = "alloc", feature = "serde"))]
pub(crate) use wire::HEAP_TAG;

use crate::ieee::{self, Special};

/// The exponent bound: every held value is `m · 2^exp` with `|exp| ≤ EXACT_MAX_EXP`, checked
/// after every operation that moves an exponent, in every build. A sum's exponent stays inside its
/// inputs' range plus the mantissa width, and a product of two doubles lies within `±2148`, so only
/// a chain of nested products moves an exponent far: roughly a million products of the smallest
/// subnormal reach the bound. Past it is the cap. The number keeps every intermediate inside `i64`
/// and every stored exponent inside `i32`, with room to spare on both sides.
pub const EXACT_MAX_EXP: i32 = 1 << 30;

/// The limb cap of the heap form: the finite part and each special amount span at most this many
/// 64-bit limbs, 16 × 2148 bits, about 4.3 KB. A product of two doubles spans 2148 bits and a
/// product of sums grows by that per factor, so the cap is reachable only through sixteen nested
/// `float × float` joins on extreme values; a sum of doubles never nears it. Past it is the cap,
/// [`PastCap::Limbs`], decided on the operands' spans before anything is allocated.
pub const EXACT_MAX_LIMBS: usize = 16 * 2148 / 64;

/// The one cap on [`Exact`]: which bound a result would cross. The operators panic with it, and
/// the checked methods return it with the state untouched. With the `alloc` feature only
/// [`PastCap::Limbs`] and [`PastCap::Exponent`] are ever returned; without it the compact form
/// stands alone and [`PastCap::Width`] and [`PastCap::Count`] cap it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PastCap {
    /// Without `alloc`: the finite part's magnitude would reach `2^127`, so the sum or product of
    /// the mantissas no longer fits the 127-bit width at the aligned exponent, or a `Wide` or
    /// `Fixed` state being converted spans more than 127 bits. With `alloc` the value promotes
    /// instead.
    Width,
    /// The exponent would leave `-EXACT_MAX_EXP..=EXACT_MAX_EXP`.
    Exponent,
    /// Without `alloc`: a special count would leave `i32` (or become `i32::MIN`), whether in an
    /// operation or in a `Wide` counter being converted, or a product would give a special a
    /// non-integer amount, as `2.5 × NaN` does. With `alloc` the value promotes instead.
    Count,
    /// With `alloc`: the finite part or a special amount would span more than
    /// [`EXACT_MAX_LIMBS`] limbs.
    Limbs,
}

impl fmt::Display for PastCap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Width => "the mantissa needs more than the 127-bit width",
            Self::Exponent => "the exponent would leave ±EXACT_MAX_EXP, 2^30",
            Self::Count => "a NaN or ±∞ count would leave i32 or stop being an integer",
            Self::Limbs => "the value would span more than EXACT_MAX_LIMBS limbs",
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
/// top, in a 32-byte word: the engine weight.
///
/// The finite part is a dyadic `m · 2^exp` in canonical form (`m` odd, or the zero form), and the
/// three amounts say how many NaN, +∞ and −∞ inputs are in the multiset, signed so that a
/// retraction subtracts. Structural `Eq` is numeric equality, and `Hash` and `Ord` agree with it.
/// Projection happens once, at read: a nonzero NaN amount or both infinities read NaN, one infinity
/// reads by the sign of its amount, and otherwise the finite part rounds to nearest-even.
///
/// The word has two forms. The compact form holds a finite part of at most 127 bits and `i32`
/// counts inline, and is every value on a build without the `alloc` feature. With `alloc` a
/// result past the width, or with a count past `i32` or a non-integer amount such as `2.5 · NaN`,
/// promotes to the heap form: the same 32 bytes, tagged by an even nonzero low limb (no compact
/// mantissa is one), holding one allocation of normalised limbs for the finite part and the three
/// amounts as dyadics. Every operation demotes a result that fits the compact form again, so one
/// value has one form, which [`Exact::is_compact`] reports. `Clone` copies the compact form and
/// allocates for the heap form; nothing else allocates, and no operation executes a
/// floating-point instruction.
///
/// The operators are exact or they panic with the named cap; the checked methods return the same
/// cap as a [`PastCap`] and leave the state as it was. A [`Wide`](crate::Wide) or
/// [`Fixed`](crate::Fixed) state converts in through `TryFrom`, and an `Exact` converts out
/// through `Wide::try_from`, refused where a product would be.
///
/// The layout is `repr(C)` with no padding: two limbs, the exponent, three counts. In the compact
/// form the bytes are the little-endian wire form on a little-endian machine. A byte pattern with
/// an even mantissa is not a compact value, so bytes from outside go through the validating
/// decoder, [`Exact::from_bytes`], and `to_bytes` encodes the compact form; a heap value crosses
/// an engine's exchange as its serde bytes.
///
/// Behind the `dbsp` feature the compact word is its own rkyv archive, so DBSP's storage writes it
/// as these bytes; the archived form's `Deserialize` and comparisons refuse a non-canonical word,
/// and a heap word, by name.
#[cfg_attr(feature = "dbsp", derive(rkyv::Archive, rkyv::Serialize))]
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

    /// Whether the word is in the heap form. Without `alloc` no word is.
    #[cfg(not(feature = "alloc"))]
    #[inline]
    #[allow(clippy::unused_self)] // the same call as the `alloc` build's, which reads the word
    const fn is_heap(&self) -> bool {
        false
    }

    /// The finite part of a compact word in its working form.
    #[inline]
    fn finite(&self) -> Compact {
        debug_assert!(!self.is_heap(), "the compact finite part of a heap word");
        Compact::from_parts(self.lo, self.hi, self.exp)
    }

    /// The counts of a compact word as `[n, p, q]`.
    #[inline]
    fn counts(&self) -> [i32; 3] {
        debug_assert!(!self.is_heap(), "the counts of a heap word");
        [self.nan, self.pos_inf, self.neg_inf]
    }

    #[inline]
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
            Overflow::Count => PastCap::Count,
            Overflow::Exponent => PastCap::Exponent,
            #[cfg(feature = "alloc")]
            Overflow::Limbs => PastCap::Limbs,
            #[cfg(test)]
            Overflow::Special => unreachable!("Exact counts a special before the tier sees it"),
        }
    }

    /// A count from a wider intermediate: inside `i32`, and never `i32::MIN`, so that negation
    /// stays total; `None` is the heap form's to hold.
    #[inline]
    pub(crate) fn narrow(v: i128) -> Option<i32> {
        match i32::try_from(v) {
            Ok(c) if c != i32::MIN => Some(c),
            _ => None,
        }
    }

    /// One count of a special.
    #[inline]
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
    #[inline]
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
    /// [`Exact::checked_mul`], is the one that scales an amount by the other weight.
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

    /// The sum of two compact words in the compact form, or why it does not fit there.
    #[inline]
    fn compact_add(&self, rhs: &Self) -> Result<Self, Overflow> {
        let finite = self.finite().checked_add(rhs.finite())?;
        let mut counts = [0i32; 3];
        for (c, (a, b)) in counts
            .iter_mut()
            .zip(self.counts().into_iter().zip(rhs.counts()))
        {
            *c = Self::narrow(i128::from(a) + i128::from(b)).ok_or(Overflow::Count)?;
        }
        Ok(Self::assemble(finite, counts))
    }

    /// Integer scaling of a compact word in the compact form, or why it does not fit there.
    #[inline]
    fn compact_mul_i64(&self, k: i64) -> Result<Self, Overflow> {
        let finite = self.finite().checked_mul(Compact::from_i64(k))?;
        let mut counts = [0i32; 3];
        for (c, a) in counts.iter_mut().zip(self.counts()) {
            *c = Self::narrow(i128::from(a) * i128::from(k)).ok_or(Overflow::Count)?;
        }
        Ok(Self::assemble(finite, counts))
    }

    /// A compact result, or the promotion the heap form takes over.
    #[cfg(feature = "alloc")]
    #[inline]
    fn or_promote(
        compact: Result<Self, Overflow>,
        promote: impl FnOnce() -> Result<Self, Overflow>,
    ) -> Result<Self, PastCap> {
        match compact {
            Ok(v) => Ok(v),
            Err(Overflow::NeedsWide | Overflow::Count) => promote().map_err(Self::cap),
            Err(e) => Err(Self::cap(e)),
        }
    }

    /// The sum, or the cap it would cross. Neither operand changes either way.
    ///
    /// # Errors
    ///
    /// With `alloc`, [`PastCap::Limbs`] when the sum would span more than the cap's limbs and
    /// [`PastCap::Exponent`] when its exponent would leave the bound. Without it, also
    /// [`PastCap::Width`] when the aligned sum of the finite parts needs more than the width and
    /// [`PastCap::Count`] when a count would leave `i32`.
    #[inline]
    pub fn checked_add(&self, rhs: &Self) -> Result<Self, PastCap> {
        #[cfg(feature = "alloc")]
        {
            let compact = if self.is_heap() || rhs.is_heap() {
                Err(Overflow::NeedsWide)
            } else {
                self.compact_add(rhs)
            };
            Self::or_promote(compact, || {
                Self::land(value::Value::add(&self.operand(), &rhs.operand())?)
            })
        }
        #[cfg(not(feature = "alloc"))]
        self.compact_add(rhs).map_err(Self::cap)
    }

    /// The difference, as [`Exact::checked_add`] of the negation.
    ///
    /// # Errors
    ///
    /// As [`Exact::checked_add`].
    #[inline]
    pub fn checked_sub(&self, rhs: &Self) -> Result<Self, PastCap> {
        self.checked_add(&-rhs)
    }

    /// The product, or the cap it would cross: the finite parts multiply exactly, and the amounts
    /// follow the monoid algebra of DESIGN.md § NaN and ±∞, so `0 · ∞` and `0 · NaN` are the
    /// zero, `(+∞)(−∞)` is one −∞, and `−1 · (+∞)` is an amount of −1 on +∞. With `alloc` the
    /// amounts are dyadic, so `2.5 · NaN` is a heap value with that amount; without it the
    /// amounts stay integer counts.
    ///
    /// # Errors
    ///
    /// With `alloc`, [`PastCap::Limbs`] and [`PastCap::Exponent`] from the finite product or an
    /// amount. Without it, also [`PastCap::Width`] from the finite product and [`PastCap::Count`]
    /// when a special amount would leave `i32` or stop being an integer, as `2.5 · NaN` would.
    pub fn checked_mul(&self, rhs: &Self) -> Result<Self, PastCap> {
        #[cfg(feature = "alloc")]
        {
            let compact = if self.is_heap() || rhs.is_heap() {
                Err(Overflow::NeedsWide)
            } else {
                ring::product(self, rhs)
            };
            Self::or_promote(compact, || {
                Self::land(value::Value::mul(&self.operand(), &rhs.operand())?)
            })
        }
        #[cfg(not(feature = "alloc"))]
        ring::product(self, rhs).map_err(Self::cap)
    }

    /// Integer scaling of the finite part and the amounts alike, as an engine applies a
    /// multiplicity or a retraction, or the cap it would cross.
    ///
    /// # Errors
    ///
    /// As [`Exact::checked_mul`] by the integer.
    #[inline]
    pub fn checked_mul_i64(&self, k: i64) -> Result<Self, PastCap> {
        #[cfg(feature = "alloc")]
        {
            let compact = if self.is_heap() {
                Err(Overflow::NeedsWide)
            } else {
                self.compact_mul_i64(k)
            };
            Self::or_promote(compact, || {
                Self::land(value::Value::mul_i64(&self.operand(), k)?)
            })
        }
        #[cfg(not(feature = "alloc"))]
        self.compact_mul_i64(k).map_err(Self::cap)
    }

    /// Adds a double, or leaves the state as it was.
    ///
    /// # Errors
    ///
    /// As [`Exact::checked_add`].
    pub fn checked_deposit(&mut self, x: f64) -> Result<(), PastCap> {
        let sum = self.checked_add(&Self::from_f64(x))?;
        self.replace(sum);
        Ok(())
    }

    /// Adds the exact product of two doubles, or leaves the state as it was. This is the join-fold
    /// and dot-product shape.
    ///
    /// # Errors
    ///
    /// As [`Exact::checked_add`] of [`Exact::product`].
    pub fn checked_deposit_product(&mut self, a: f64, b: f64) -> Result<(), PastCap> {
        let sum = self.checked_add(&Self::product(a, b))?;
        self.replace(sum);
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

    /// `with` in place of this value. A compact word owns nothing, so it is overwritten without
    /// the drop glue; a heap word's block is freed.
    #[inline]
    pub(crate) fn replace(&mut self, with: Self) {
        #[cfg(feature = "alloc")]
        if !self.is_heap() {
            self.write_compact(with);
            return;
        }
        *self = with;
    }

    /// Negation in place: the signs flip, and a heap word's block is kept.
    #[inline]
    pub(crate) fn negate_assign(&mut self) {
        #[cfg(feature = "alloc")]
        if self.is_heap() {
            self.negate_block();
            return;
        }
        let negated = Self::assemble(-self.finite(), self.counts().map(|c| -c));
        self.replace(negated);
    }

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
}
