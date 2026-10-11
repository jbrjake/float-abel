//! [`Fixed`]: a `Copy` two's-complement fixed-point integer in units of `2^LSB`, exact or refused.

use core::cmp::Ordering;
use core::fmt;
use core::iter::Sum;
use core::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

use crate::ieee;
use crate::limbs;
use crate::round;

/// Why [`Fixed`] refused an input. Its contract is *exact or refused*: a value it cannot hold
/// bit for bit never changes the state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OutOfWindow {
    /// NaN or ±∞. `Fixed` holds finite values only.
    NotFinite,
    /// A finite value with a set bit below the grid's unit, `2^LSB`. [`Fixed::quantize`] rounds
    /// such a value onto the grid for callers who declare that floor.
    OffGrid,
    /// A finite value at or beyond the magnitude ceiling, `2^CEILING_EXP`.
    AboveCeiling,
}

impl fmt::Display for OutOfWindow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NotFinite => "not finite: Fixed holds finite values only",
            Self::OffGrid => "off the grid: a set bit lies below the unit 2^LSB",
            Self::AboveCeiling => {
                "above the ceiling: the magnitude needs more bits than the window has"
            }
        })
    }
}

#[cfg(feature = "std")]
impl std::error::Error for OutOfWindow {}

/// An exact abelian group of finite doubles on the grid `2^LSB`, as a `[u64; LIMBS]`
/// two's-complement integer in units of `2^LSB`.
///
/// The window is the whole contract. [`Fixed::try_deposit`] accepts a finite double iff it is
/// exactly a multiple of `2^LSB` and `|x| < 2^CEILING_EXP`, where
/// `CEILING_EXP = 64·LIMBS − 65 + LSB`: a sign bit and 64 bits of headroom come off the width.
/// Everything else is refused by name as an [`OutOfWindow`], or rounded onto the grid first by
/// [`Fixed::quantize`] for callers who declare a floor. Inside the window the group operations are
/// infallible for a budget of `2^64` deposits at the ceiling; past that they panic by name in every
/// build, because a wrapped sum is silently wrong.
///
/// Structural `Eq`, `Ord` and `Hash` are numeric: a two's-complement integer has one bit pattern
/// per value, so there is nothing to canonicalise and the zero has no sign. No operation executes
/// a floating-point instruction, so the bits are the same on every architecture.
///
/// A plain word: `repr(C)` over the limbs alone, with no padding and no invalid bit pattern, so on
/// a little-endian machine the in-memory bytes are the wire body and an engine may cast a mapping
/// to `Fixed` without a copy. [`Fixed256`]'s size and alignment are pinned by a `const` assertion,
/// and [`Fixed::from_limbs`] is the safe inverse of [`Fixed::limbs`].
///
/// [`Fixed256`] is the named default.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Fixed<const LSB: i32, const LIMBS: usize> {
    limbs: [u64; LIMBS],
}

/// 256 bits in units of `2^-96`: every double with `|x| ≥ 2^-44` is on the grid, the ceiling is
/// `2^95`, and the headroom is `2^64` deposits at the ceiling.
pub type Fixed256 = Fixed<-96, 4>;

const _: () = assert!(
    core::mem::size_of::<Fixed256>() == 32 && core::mem::align_of::<Fixed256>() == 8,
    "Fixed256's layout moved"
);

impl<const LSB: i32, const LIMBS: usize> Fixed<LSB, LIMBS> {
    /// The grid: every held value is a multiple of `2^LSB`.
    pub const LSB: i32 = LSB;
    /// The width in 64-bit limbs.
    pub const LIMBS: usize = LIMBS;
    /// Deposits are accepted iff `|x| < 2^CEILING_EXP`.
    pub const CEILING_EXP: i32 = 64 * LIMBS as i32 - 65 + LSB;
    /// The zero, which is the only zero.
    pub const ZERO: Self = Self { limbs: [0; LIMBS] };

    /// Bits a magnitude may occupy, in units: the width less a sign bit and 64 bits of headroom.
    const CEILING_BITS: u64 = 64 * LIMBS as u64 - 65;
    const WINDOW_OK: () = assert!(
        LIMBS >= 2,
        "Fixed needs at least two limbs: a sign bit and 64 bits of headroom leave nothing in one"
    );

    /// The two's-complement limbs, least significant first.
    #[must_use]
    pub const fn limbs(&self) -> &[u64; LIMBS] {
        &self.limbs
    }

    /// The value with these limbs. Total: every bit pattern is a canonical value, which is what
    /// makes a cast over a mapping sound, and `from_limbs(*f.limbs())` is `f` again.
    #[must_use]
    pub const fn from_limbs(limbs: [u64; LIMBS]) -> Self {
        Self { limbs }
    }

    /// Whether the state is zero.
    #[must_use]
    pub fn is_zero(&self) -> bool {
        limbs::is_zero(&self.limbs)
    }

    /// Adds a finite double that is exactly on the grid and inside the window.
    ///
    /// # Errors
    ///
    /// NaN and ±∞ are [`OutOfWindow::NotFinite`]; a value with a set bit below `2^LSB` is
    /// [`OutOfWindow::OffGrid`]; a magnitude at or beyond `2^CEILING_EXP` is
    /// [`OutOfWindow::AboveCeiling`]. A refusal leaves the state as it was.
    pub fn try_deposit(&mut self, x: f64) -> Result<(), OutOfWindow> {
        let d = ieee::decompose(x).ok_or(OutOfWindow::NotFinite)?;
        self.deposit_dyadic(d.negative, u128::from(d.mant), d.exp)
    }

    /// Adds the exact product of two finite doubles: the full 106-bit mantissa product, without a
    /// floating-point multiply. This is the join-fold and dot-product shape.
    ///
    /// # Errors
    ///
    /// As [`Fixed::try_deposit`], judged on the exact product. A zero factor makes the product an
    /// exact zero, which is always accepted.
    pub fn deposit_product(&mut self, a: f64, b: f64) -> Result<(), OutOfWindow> {
        let da = ieee::decompose(a).ok_or(OutOfWindow::NotFinite)?;
        let db = ieee::decompose(b).ok_or(OutOfWindow::NotFinite)?;
        let mant = u128::from(da.mant) * u128::from(db.mant);
        self.deposit_dyadic(da.negative != db.negative, mant, da.exp + db.exp)
    }

    /// Adds `(-1)^negative · mant · 2^exp`, checking the grid and the ceiling first.
    fn deposit_dyadic(&mut self, negative: bool, mant: u128, exp: i32) -> Result<(), OutOfWindow> {
        #[allow(clippy::let_unit_value)] // evaluates the const assertion on the window
        let () = Self::WINDOW_OK;
        if mant == 0 {
            return Ok(());
        }
        let shift = i64::from(exp) - i64::from(LSB);
        if shift < 0 {
            return Err(OutOfWindow::OffGrid);
        }
        let bits = u64::from(128 - mant.leading_zeros()) + shift as u64;
        if bits > Self::CEILING_BITS {
            return Err(OutOfWindow::AboveCeiling);
        }
        let mut term = [0u64; LIMBS];
        limbs::set_shifted(&mut term, mant, shift as u32);
        if negative {
            limbs::negate(&mut term);
        }
        let overflow = limbs::add_assign(&mut self.limbs, &term);
        assert!(
            !overflow,
            "Fixed: headroom exhausted, past 2^64 deposits at the ceiling"
        );
        Ok(())
    }

    /// Rounds a double onto the grid, to nearest with ties to even, so that
    /// [`Fixed::try_deposit`] accepts it unless it is above the ceiling. On-grid values, zeros
    /// and specials pass through unchanged. The result is a function of the input alone, so
    /// every engine quantises identically before depositing.
    ///
    /// Rounding is odd: `quantize(-x) == -quantize(x)` bit for bit, and a value below half a unit
    /// becomes a zero of its own sign.
    #[must_use]
    pub fn quantize(x: f64) -> f64 {
        let Some(d) = ieee::decompose(x) else {
            return x;
        };
        if d.mant == 0 {
            return x;
        }
        let shift = i64::from(LSB) - i64::from(d.exp);
        if shift <= 0 {
            return x;
        }
        // A mantissa has at most 53 bits, so a shift of 64 or more is below half a unit.
        let kept = if shift >= 64 {
            0
        } else {
            let shift = shift as u32;
            let q = d.mant >> shift;
            let round_bit = (d.mant >> (shift - 1)) & 1 == 1;
            let below = d.mant & ((1u64 << (shift - 1)) - 1) != 0;
            q + u64::from(round_bit && (below || q & 1 == 1))
        };
        // `kept · 2^LSB` has at most 54 bits at a weight the input's grid lay below, so it is
        // exactly a double; `round` only builds its bits.
        let (bits, _) = round::round(d.negative, &[kept], LSB, false, round::F64);
        f64::from_bits(bits)
    }

    /// The value rounded to nearest-even as a double, in integer arithmetic, with subnormals and
    /// overflow to ±∞. A nonzero value that rounds to zero keeps its sign, as IEEE rounds it.
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
        let mut mag = [0u64; LIMBS];
        let negative = limbs::abs_into(&self.limbs, &mut mag);
        let (bits, inexact) = round::round(negative, &mag, LSB, false, round::F64);
        (f64::from_bits(bits), inexact)
    }

    /// The correctly rounded quotient of the exact value by an integer: what an `AVG` calls, so
    /// every engine divides the same exact numerator once. Division by zero follows IEEE: `0/0`
    /// is NaN and `x/0` is `±∞`.
    #[must_use]
    pub fn div_f64(&self, n: u64) -> f64 {
        let mut mag = [0u64; LIMBS];
        let negative = limbs::abs_into(&self.limbs, &mut mag);
        round::div_f64(negative, &mag, LSB, n)
    }
}

impl<const LSB: i32, const LIMBS: usize> Default for Fixed<LSB, LIMBS> {
    fn default() -> Self {
        Self::ZERO
    }
}

/// A limb printed in hex inside a `Debug` list.
pub(crate) struct Hex(pub(crate) u64);

impl fmt::Debug for Hex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

impl<const LSB: i32, const LIMBS: usize> fmt::Debug for Fixed<LSB, LIMBS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (value, inexact) = self.read();
        let about = if inexact { "≈" } else { "" };
        write!(f, "Fixed<{LSB}, {LIMBS}>({about}{value:e}; limbs ")?;
        f.debug_list()
            .entries(self.limbs.iter().map(|&l| Hex(l)))
            .finish()?;
        f.write_str(")")
    }
}

impl<const LSB: i32, const LIMBS: usize> TryFrom<f64> for Fixed<LSB, LIMBS> {
    type Error = OutOfWindow;

    /// A fresh state holding `x`, or the refusal [`Fixed::try_deposit`] would give.
    fn try_from(x: f64) -> Result<Self, OutOfWindow> {
        let mut f = Self::ZERO;
        f.try_deposit(x)?;
        Ok(f)
    }
}

impl<const LSB: i32, const LIMBS: usize> PartialOrd for Fixed<LSB, LIMBS> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<const LSB: i32, const LIMBS: usize> Ord for Fixed<LSB, LIMBS> {
    fn cmp(&self, other: &Self) -> Ordering {
        limbs::cmp(&self.limbs, &other.limbs)
    }
}

impl<const LSB: i32, const LIMBS: usize> AddAssign for Fixed<LSB, LIMBS> {
    fn add_assign(&mut self, rhs: Self) {
        let overflow = limbs::add_assign(&mut self.limbs, &rhs.limbs);
        assert!(!overflow, "Fixed: headroom exhausted in an add");
    }
}

impl<const LSB: i32, const LIMBS: usize> Add for Fixed<LSB, LIMBS> {
    type Output = Self;
    fn add(mut self, rhs: Self) -> Self {
        self += rhs;
        self
    }
}

impl<const LSB: i32, const LIMBS: usize> Neg for Fixed<LSB, LIMBS> {
    type Output = Self;
    fn neg(mut self) -> Self {
        let overflow = limbs::negate(&mut self.limbs);
        assert!(!overflow, "Fixed: headroom exhausted in a negation");
        self
    }
}

impl<const LSB: i32, const LIMBS: usize> SubAssign for Fixed<LSB, LIMBS> {
    fn sub_assign(&mut self, rhs: Self) {
        *self += -rhs;
    }
}

impl<const LSB: i32, const LIMBS: usize> Sub for Fixed<LSB, LIMBS> {
    type Output = Self;
    fn sub(mut self, rhs: Self) -> Self {
        self -= rhs;
        self
    }
}

impl<const LSB: i32, const LIMBS: usize> Mul<i64> for Fixed<LSB, LIMBS> {
    type Output = Self;

    /// Integer scaling, as an engine applies a multiplicity or a retraction.
    fn mul(self, k: i64) -> Self {
        let mut mag = [0u64; LIMBS];
        let negative = limbs::abs_into(&self.limbs, &mut mag);
        let carry = limbs::mul_assign_u64(&mut mag, k.unsigned_abs());
        assert!(
            carry == 0 && !limbs::is_negative(&mag),
            "Fixed: headroom exhausted in a scalar multiply"
        );
        if negative != (k < 0) {
            limbs::negate(&mut mag);
        }
        Self { limbs: mag }
    }
}

impl<const LSB: i32, const LIMBS: usize> Mul<isize> for Fixed<LSB, LIMBS> {
    type Output = Self;
    fn mul(self, k: isize) -> Self {
        self * (k as i64)
    }
}

impl<const LSB: i32, const LIMBS: usize> Sum for Fixed<LSB, LIMBS> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, Add::add)
    }
}

impl<'a, const LSB: i32, const LIMBS: usize> Sum<&'a Self> for Fixed<LSB, LIMBS> {
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, |acc, x| acc + *x)
    }
}
