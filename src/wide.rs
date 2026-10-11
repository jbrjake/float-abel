//! [`Wide`]: a `Copy` 34-limb two's-complement integer in units of `2^-1074` plus three signed
//! special counters. Every `f64` sum, exactly, with NaN and ±∞ counted rather than refused.

use core::cmp::Ordering;
use core::fmt;
use core::iter::Sum;
use core::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

use crate::fixed::{Fixed, Hex, OutOfWindow};
use crate::ieee::{
    self, factor, special_times_finite, special_times_special, Dyadic, Factor, NAN, NEG_INF,
    POS_INF,
};
use crate::limbs;
use crate::round;

const LIMBS: usize = 34;
/// The unit is the smallest subnormal, so every finite double is a whole number of units.
const LSB: i32 = -1074;
/// Bits a product's magnitude may occupy, in units: the width less a sign bit and 64 bits of
/// headroom. A finite double needs at most 2098, so only products can reach it.
const CEILING_BITS: u64 = 64 * LIMBS as u64 - 65;

/// An exact abelian group of every `f64`, specials included: `[u64; 34]` two's complement in
/// units of `2^-1074` for the finite sum, and `[i64; 3]` signed counts of the NaN, +∞ and −∞
/// inputs in the multiset.
///
/// [`From<f64>`] and [`Wide::deposit`] are total. A finite double lands in the limbs exactly, in
/// bitrep's layout; a special bumps its counter. Addition, negation and integer scaling act on the
/// limbs and the counters alike, so the group laws are inherited from ℤ³ ⊕ D and retracting a NaN
/// restores the finite sum bit for bit. Projection happens once, at read: a nonzero NaN count, or
/// both infinities present, reads NaN; one infinity reads by the sign of its count; otherwise the
/// finite part rounds to nearest-even. Every finite part is at least one unit, and the unit is the
/// smallest subnormal, so a nonzero finite part never rounds to zero; the zero state reads `+0.0`.
///
/// The group operations are infallible for `2^77` deposits of `f64::MAX`, and the counters for
/// `i64`; past either budget they panic by name in every build, because a wrapped sum is silently
/// wrong. Structural `Eq`, `Ord` and `Hash` are numeric, because a two's-complement integer and
/// three integers have one bit pattern per value. No operation executes a floating-point
/// instruction.
///
/// [`Wide::deposit_product`] and `TryFrom<&Exact>` are the fallible entries: the exact product of
/// two doubles, or an [`Exact`](crate::Exact), may lie below the unit or past `2^1037`, and then it
/// is refused by name rather than rounded. `From<Fixed>` is exact, pinned at compile time to the
/// `(LSB, LIMBS)` pairs `Wide` holds, and `TryFrom<&Wide>` into `Exact` refuses past its width.
///
/// A plain word: `repr(C)`, the limbs then the counters, with no padding and no invalid bit
/// pattern, so on a little-endian machine the in-memory bytes are the wire body and an engine may
/// cast a mapping to `Wide` without a copy. The size and alignment are pinned by a `const`
/// assertion, and [`Wide::from_parts`] is the safe inverse of [`Wide::limbs`] and
/// [`Wide::specials`].
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Wide {
    limbs: [u64; LIMBS],
    /// `[n, p, q]`: NaN, +∞ and −∞ counts, signed so that a retraction subtracts.
    specials: [i64; 3],
}

const _: () = assert!(
    core::mem::size_of::<Wide>() == 8 * (LIMBS + 3) && core::mem::align_of::<Wide>() == 8,
    "Wide's layout moved"
);

impl Wide {
    /// The grid: every held finite value is a multiple of `2^LSB`, the smallest subnormal.
    pub const LSB: i32 = LSB;
    /// The width of the finite part in 64-bit limbs.
    pub const LIMBS: usize = LIMBS;
    /// Products are accepted iff `|a · b| < 2^CEILING_EXP`. Every finite double is far below it.
    pub const CEILING_EXP: i32 = CEILING_BITS as i32 + LSB;
    /// The zero, which is the only zero.
    pub const ZERO: Self = Self {
        limbs: [0; LIMBS],
        specials: [0; 3],
    };

    /// The two's-complement limbs of the finite part, least significant first, in units of
    /// `2^-1074`: bitrep's layout, so its conformance vectors apply to these bytes directly.
    #[must_use]
    pub const fn limbs(&self) -> &[u64; LIMBS] {
        &self.limbs
    }

    /// The value with these limbs and `[n, p, q]` counts. Total: every bit pattern of the two
    /// arrays is a canonical value, which is what makes a cast over a mapping sound, and
    /// `from_parts(*w.limbs(), [n, p, q])` with `(n, p, q) = w.specials()` is `w` again.
    #[must_use]
    pub const fn from_parts(limbs: [u64; LIMBS], specials: [i64; 3]) -> Self {
        Self { limbs, specials }
    }

    /// The special counts `(n, p, q)`: how many NaN, +∞ and −∞ inputs are in the multiset, each
    /// negative when more were retracted than inserted.
    #[must_use]
    pub const fn specials(&self) -> (i64, i64, i64) {
        (
            self.specials[NAN],
            self.specials[POS_INF],
            self.specials[NEG_INF],
        )
    }

    /// Whether the state is the zero: no finite part and no specials.
    #[must_use]
    pub fn is_zero(&self) -> bool {
        limbs::is_zero(&self.limbs) && self.specials == [0; 3]
    }

    /// Whether every counter is zero, so a read rounds the finite part. The finite part may still
    /// round to `±∞` when it lies past `f64::MAX`.
    #[must_use]
    pub fn is_finite(&self) -> bool {
        self.specials == [0; 3]
    }

    /// Adds a double. A finite value lands in the limbs exactly, either zero adds nothing, and NaN,
    /// +∞ and −∞ each bump their counter.
    pub fn deposit(&mut self, x: f64) {
        match ieee::decompose(x) {
            Some(d) if d.mant == 0 => {}
            // A finite double's lowest bit is at or above the unit, so the shift is never negative.
            Some(d) => self.deposit_finite(d.negative, u128::from(d.mant), (d.exp - LSB) as u32),
            None => self.bump(ieee::counter(ieee::special(x)), 1),
        }
    }

    /// Adds the exact product of two doubles: the full 106-bit mantissa product, without a
    /// floating-point multiply. This is the join-fold and dot-product shape.
    ///
    /// A special factor follows the NaN algebra of the design with a unit count: a NaN factor adds
    /// one NaN; two infinities add one +∞ when their signs agree and one −∞ when they differ; an
    /// infinity times a nonzero finite factor adds `±1` to that infinity's counter by the finite
    /// factor's sign, so a negative factor flips the infinity through the projection rule. A zero
    /// factor makes the product an exact zero whatever the other factor is, as bilinearity
    /// requires: `0 · ∞` and `0 · NaN` deposit nothing.
    ///
    /// # Errors
    ///
    /// A finite product with a set bit below the unit is [`OutOfWindow::OffGrid`], and one at or
    /// beyond `2^CEILING_EXP` is [`OutOfWindow::AboveCeiling`]; the state is then unchanged. A
    /// hardware multiply has already overflowed to `±∞` well below the ceiling, and underflowed
    /// to zero well above the unit. [`OutOfWindow::NotFinite`] never occurs: specials are counted.
    pub fn deposit_product(&mut self, a: f64, b: f64) -> Result<(), OutOfWindow> {
        match (factor(a), factor(b)) {
            (Factor::Finite(da), Factor::Finite(db)) => self.deposit_finite_product(da, db),
            (Factor::Finite(d), Factor::Special(_)) | (Factor::Special(_), Factor::Finite(d))
                if d.mant == 0 =>
            {
                Ok(())
            }
            (Factor::Finite(d), Factor::Special(s)) | (Factor::Special(s), Factor::Finite(d)) => {
                let (which, by) = special_times_finite(s, d.negative);
                self.bump(which, by);
                Ok(())
            }
            (Factor::Special(s), Factor::Special(t)) => {
                self.bump(special_times_special(s, t), 1);
                Ok(())
            }
        }
    }

    fn deposit_finite_product(&mut self, da: Dyadic, db: Dyadic) -> Result<(), OutOfWindow> {
        self.try_deposit_dyadic(
            da.negative != db.negative,
            u128::from(da.mant) * u128::from(db.mant),
            da.exp + db.exp,
        )
    }

    /// Adds `(-1)^negative · mant · 2^exp` if it lies in the window: the one check a product and
    /// an [`Exact`](crate::Exact) share. A zero mantissa adds nothing.
    ///
    /// # Errors
    ///
    /// [`OutOfWindow::OffGrid`] when a set bit would lie below the unit, and
    /// [`OutOfWindow::AboveCeiling`] when the magnitude reaches `2^CEILING_EXP`; the state is then
    /// unchanged.
    pub(crate) fn try_deposit_dyadic(
        &mut self,
        negative: bool,
        mant: u128,
        exp: i32,
    ) -> Result<(), OutOfWindow> {
        if mant == 0 {
            return Ok(());
        }
        let shift = i64::from(exp) - i64::from(LSB);
        if shift < 0 {
            return Err(OutOfWindow::OffGrid);
        }
        let bits = u64::from(128 - mant.leading_zeros()) + shift as u64;
        if bits > CEILING_BITS {
            return Err(OutOfWindow::AboveCeiling);
        }
        self.deposit_finite(negative, mant, shift as u32);
        Ok(())
    }

    /// Adds `(-1)^negative · mant · 2^(shift + LSB)`. The caller has checked that it fits.
    fn deposit_finite(&mut self, negative: bool, mant: u128, shift: u32) {
        let mut term = [0u64; LIMBS];
        limbs::set_shifted(&mut term, mant, shift);
        if negative {
            limbs::negate(&mut term);
        }
        let overflow = limbs::add_assign(&mut self.limbs, &term);
        assert!(
            !overflow,
            "Wide: headroom exhausted, past 2^77 deposits of f64::MAX"
        );
    }

    fn bump(&mut self, which: usize, by: i64) {
        let (v, overflow) = self.specials[which].overflowing_add(by);
        assert!(!overflow, "Wide: a special counter overflowed in a deposit");
        self.specials[which] = v;
    }

    /// The special the counters project to, or `None` when they are all zero.
    fn projection(&self) -> Option<f64> {
        let [n, p, q] = self.specials;
        ieee::project(n, p, q)
    }

    /// The value as a double: the special projection if the counters say so, else the finite part
    /// rounded to nearest-even in integer arithmetic, with overflow to `±∞`. The zero state reads
    /// `+0.0`.
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
        if let Some(special) = self.projection() {
            return (special, false);
        }
        let mut mag = [0u64; LIMBS];
        let negative = limbs::abs_into(&self.limbs, &mut mag);
        let (bits, inexact) = round::round(negative, &mag, LSB, false, round::F64);
        (f64::from_bits(bits), inexact)
    }

    /// The correctly rounded quotient of the exact value by an integer: what an `AVG` calls, so
    /// every engine divides the same exact numerator once. A special projection is returned as it
    /// is, as IEEE divides it; a finite part follows IEEE for a zero divisor too, `0/0` NaN and
    /// `x/0` `±∞`.
    #[must_use]
    pub fn div_f64(&self, n: u64) -> f64 {
        if let Some(special) = self.projection() {
            return special;
        }
        let mut mag = [0u64; LIMBS];
        let negative = limbs::abs_into(&self.limbs, &mut mag);
        round::div_f64(negative, &mag, LSB, n)
    }
}

impl Default for Wide {
    fn default() -> Self {
        Self::ZERO
    }
}

impl fmt::Debug for Wide {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (value, inexact) = self.read();
        let about = if inexact { "≈" } else { "" };
        let [n, p, q] = self.specials;
        write!(
            f,
            "Wide({about}{value:e}; nan {n}, +inf {p}, -inf {q}; limbs "
        )?;
        // Limbs above the highest one that differs from the sign fill carry no information.
        let fill = if limbs::is_negative(&self.limbs) {
            u64::MAX
        } else {
            0
        };
        let shown = self
            .limbs
            .iter()
            .rposition(|&l| l != fill)
            .map_or(0, |i| i + 1);
        f.debug_list()
            .entries(self.limbs[..shown].iter().map(|&l| Hex(l)))
            .finish()?;
        write!(f, " then {} × {fill:#x})", LIMBS - shown)
    }
}

impl From<f64> for Wide {
    /// A fresh state holding `x`: its finite value, or one count of its special.
    fn from(x: f64) -> Self {
        let mut w = Self::ZERO;
        w.deposit(x);
        w
    }
}

/// The `(LSB, LIMBS)` pairs of a [`Fixed`] that `Wide` holds exactly: the grid at or above the
/// unit, so a `Fixed` unit is a whole number of `Wide` units, and the width shifted onto `Wide`'s
/// grid inside `Wide`'s width, so no state is cut. Both ceilings carry the same 65 bits of sign
/// and headroom, so the second condition is also `Fixed`'s ceiling lying inside `Wide`'s.
struct Embeds<const LSB: i32, const LIMBS: usize>;

impl<const LSB: i32, const LIMBS: usize> Embeds<LSB, LIMBS> {
    const OK: () = assert!(
        LSB >= Wide::LSB
            && 64 * (LIMBS as i64) + (LSB as i64 - Wide::LSB as i64) <= 64 * (Wide::LIMBS as i64),
        "Wide: From<Fixed> is exact only for a grid at or above 2^-1074 whose width, shifted onto \
         Wide's grid, fits Wide's 34 limbs"
    );
}

impl<const LSB: i32, const LIMBS: usize> From<Fixed<LSB, LIMBS>> for Wide {
    /// The same finite value, exactly, with no specials: the limbs sign-extended and shifted
    /// `LSB + 1074` bits up onto `Wide`'s grid. The impl is pinned at compile time to the
    /// `(LSB, LIMBS)` pairs it is exact for, by a `const` assertion rather than a runtime refusal:
    /// `Fixed<-1076, 20>` does not build.
    fn from(f: Fixed<LSB, LIMBS>) -> Self {
        #[allow(clippy::let_unit_value)] // evaluates the const assertion on the pair
        let () = Embeds::<LSB, LIMBS>::OK;
        let mut w = Self::ZERO;
        limbs::shl_extended(f.limbs(), &mut w.limbs, (LSB - Wide::LSB) as u32);
        w
    }
}

impl PartialOrd for Wide {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Wide {
    /// Numeric on the finite part, then `(n, p, q)` lexicographic: a total order.
    fn cmp(&self, other: &Self) -> Ordering {
        limbs::cmp(&self.limbs, &other.limbs).then_with(|| self.specials.cmp(&other.specials))
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
