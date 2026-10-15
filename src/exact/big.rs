//! The heap arm's working form: a normalised sign-magnitude dyadic over `u64` limbs, borrowed as
//! a [`View`] from a compact word, a count or a heap block, and owned as an [`Owned`] when an
//! operation makes one. Nothing here is `unsafe` and nothing executes a floating-point
//! instruction. The one allocation is a result's limbs, taken with room in front for the block
//! header so the heap form can take the buffer as its block without a copy, and never before the
//! limb cap has been checked on the operands' spans, so a result past the cap costs nothing.

use alloc::vec::Vec;
use core::cmp::Ordering;

use super::compact::{Compact, Overflow};
use super::heap::HEADER;
use super::{EXACT_MAX_EXP, EXACT_MAX_LIMBS};
use crate::limbs;
use crate::round;

/// Bits a run may span under the limb cap.
const CAP_BITS: i64 = 64 * EXACT_MAX_LIMBS as i64;

/// `(-1)^neg · mag · 2^exp`, normalised and borrowed: `mag` is empty for the zero, with `neg`
/// false and `exp` zero, and otherwise its first limb is odd and its last is nonzero, so the
/// lowest set bit sits at `exp` and one value has one spelling.
#[derive(Clone, Copy, Debug)]
pub(crate) struct View<'a> {
    pub(crate) neg: bool,
    pub(crate) exp: i64,
    pub(crate) mag: &'a [u64],
}

/// The same form, owned by an operation's result.
#[derive(Debug)]
pub(crate) struct Owned {
    pub(crate) neg: bool,
    pub(crate) exp: i64,
    pub(crate) mag: Vec<u64>,
}

/// A dyadic of at most two limbs held inline: a compact word's finite part or a count, viewed
/// without allocating.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Small {
    neg: bool,
    exp: i64,
    len: usize,
    mag: [u64; 2],
}

impl Small {
    const ZERO: Self = Self {
        neg: false,
        exp: 0,
        len: 0,
        mag: [0; 2],
    };

    pub(crate) fn from_compact(c: Compact) -> Self {
        let (neg, mant, exp) = c.dyadic();
        let mag = [mant as u64, (mant >> 64) as u64];
        let len = if mag[1] != 0 {
            2
        } else {
            usize::from(mag[0] != 0)
        };
        if len == 0 {
            return Self::ZERO;
        }
        Self {
            neg,
            exp: i64::from(exp),
            len,
            mag,
        }
    }

    /// An integer, canonicalised: its trailing zeros move into the exponent.
    pub(crate) fn from_i64(k: i64) -> Self {
        if k == 0 {
            return Self::ZERO;
        }
        let m = k.unsigned_abs();
        let tz = m.trailing_zeros();
        Self {
            neg: k < 0,
            exp: i64::from(tz),
            len: 1,
            mag: [m >> tz, 0],
        }
    }

    pub(crate) fn view(&self) -> View<'_> {
        View {
            neg: self.neg,
            exp: self.exp,
            mag: &self.mag[..self.len],
        }
    }
}

impl View<'_> {
    #[inline]
    pub(crate) fn is_zero(self) -> bool {
        self.mag.is_empty()
    }

    /// Bits from the lowest set bit to the highest; zero for the zero.
    pub(crate) fn bits(self) -> i64 {
        match self.mag.last() {
            None => 0,
            Some(&top) => 64 * self.mag.len() as i64 - i64::from(top.leading_zeros()),
        }
    }

    /// The highest set bit's position on the absolute scale, for a nonzero value.
    fn top(self) -> i64 {
        self.exp + self.bits() - 1
    }

    /// Bits `[at, at + 64)` of the magnitude on the absolute scale; bits outside the value are
    /// zero, so a window may straddle either end.
    fn window(self, at: i64) -> u64 {
        let off = at - self.exp;
        if off >= 0 {
            limbs::window(self.mag, off as u64, 64)
        } else if off <= -64 {
            0
        } else {
            limbs::window(self.mag, 0, (64 + off) as u32) << (-off) as u32
        }
    }

    /// `-1`, `0` or `1`.
    fn signum(self) -> i8 {
        if self.is_zero() {
            0
        } else if self.neg {
            -1
        } else {
            1
        }
    }

    /// The value as a compact dyadic when its span fits the width, `None` when it does not.
    ///
    /// # Errors
    ///
    /// [`Overflow::Exponent`] when the exponent leaves the bound.
    pub(crate) fn to_compact(self) -> Result<Option<Compact>, Overflow> {
        let Ok(lsb) = i32::try_from(self.exp) else {
            return Err(Overflow::Exponent);
        };
        match Compact::from_magnitude(self.neg, self.mag, lsb) {
            Ok(c) => Ok(Some(c)),
            Err(Overflow::NeedsWide) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// The value as an `i64` if it is an integer one holds.
    pub(crate) fn to_i64(self) -> Option<i64> {
        if self.is_zero() {
            return Some(0);
        }
        if self.exp < 0 || self.exp + self.bits() > 64 {
            return None;
        }
        let m = self.mag[0] << self.exp;
        if self.neg {
            (m <= 1 << 63).then(|| (m as i64).wrapping_neg())
        } else {
            i64::try_from(m).ok()
        }
    }

    /// The value as a count the compact form holds: an integer inside `i32`, never `i32::MIN`.
    pub(crate) fn to_count(self) -> Option<i32> {
        self.to_i64()
            .and_then(|k| i32::try_from(k).ok())
            .filter(|&c| c != i32::MIN)
    }

    /// The value rounded to nearest-even as a double, and whether that lost anything.
    pub(crate) fn read(self) -> (f64, bool) {
        let (bits, inexact) = round::round(self.neg, self.mag, self.exp as i32, false, round::F64);
        (f64::from_bits(bits), inexact)
    }

    /// The value rounded once to nearest-even as a single.
    pub(crate) fn read_f32(self) -> f32 {
        let (bits, _) = round::round(self.neg, self.mag, self.exp as i32, false, round::F32);
        f32::from_bits(bits as u32)
    }

    /// The correctly rounded quotient by an integer, with IEEE's answers for a zero divisor.
    pub(crate) fn div_f64(self, n: u64) -> f64 {
        round::div_f64(self.neg, self.mag, self.exp as i32, n)
    }
}

impl Owned {
    pub(crate) const ZERO: Self = Self {
        neg: false,
        exp: 0,
        mag: Vec::new(),
    };

    /// Room for `limbs` limbs and the block header in front of them.
    fn with_room(limbs: usize) -> Self {
        Self {
            neg: false,
            exp: 0,
            mag: Vec::with_capacity(limbs + HEADER),
        }
    }

    /// A copy of a view; the zero costs nothing.
    pub(crate) fn from_view(v: View<'_>) -> Self {
        if v.is_zero() {
            return Self::ZERO;
        }
        let mut o = Self::with_room(v.mag.len());
        o.mag.extend_from_slice(v.mag);
        o.neg = v.neg;
        o.exp = v.exp;
        o
    }

    pub(crate) fn from_i64(k: i64) -> Self {
        Self::from_view(Small::from_i64(k).view())
    }

    /// `(-1)^negative · mag · 2^lsb` for a magnitude in limbs as a fixed-width tier holds it,
    /// canonicalised.
    ///
    /// # Errors
    ///
    /// [`Overflow::Limbs`] past the cap, and [`Overflow::Exponent`] when the canonical exponent
    /// leaves the bound.
    pub(crate) fn from_magnitude(negative: bool, mag: &[u64], lsb: i64) -> Result<Self, Overflow> {
        let mut o = Self::with_room(mag.len());
        o.mag.extend_from_slice(mag);
        o.neg = negative;
        o.exp = lsb;
        o.normalise();
        o.check()
    }

    pub(crate) fn view(&self) -> View<'_> {
        View {
            neg: self.neg,
            exp: self.exp,
            mag: &self.mag,
        }
    }

    /// Canonical form: leading zero limbs stripped, trailing zero bits moved into the exponent,
    /// and the zero as the zero form.
    fn normalise(&mut self) {
        while self.mag.last() == Some(&0) {
            self.mag.pop();
        }
        if self.mag.is_empty() {
            self.neg = false;
            self.exp = 0;
            return;
        }
        let zero_limbs = self.mag.iter().take_while(|&&l| l == 0).count();
        if zero_limbs > 0 {
            self.mag.drain(..zero_limbs);
            self.exp += 64 * zero_limbs as i64;
        }
        let tz = self.mag[0].trailing_zeros();
        if tz > 0 {
            shr(&mut self.mag, tz);
            self.exp += i64::from(tz);
            if self.mag.last() == Some(&0) {
                self.mag.pop();
            }
        }
    }

    /// The cap and the bound on a normalised result.
    fn check(self) -> Result<Self, Overflow> {
        if self.mag.len() > EXACT_MAX_LIMBS {
            return Err(Overflow::Limbs);
        }
        let max = i64::from(EXACT_MAX_EXP);
        if !self.mag.is_empty() && !(-max..=max).contains(&self.exp) {
            return Err(Overflow::Exponent);
        }
        Ok(self)
    }
}

/// `a >>= bits` across limbs, for `0 < bits < 64`.
fn shr(a: &mut [u64], bits: u32) {
    let n = a.len();
    for i in 0..n {
        let high = if i + 1 < n {
            a[i + 1] << (64 - bits)
        } else {
            0
        };
        a[i] = (a[i] >> bits) | high;
    }
}

/// The sum, aligned to the lower exponent and canonicalised.
///
/// # Errors
///
/// [`Overflow::Limbs`] when the aligned span, with a bit for the carry, exceeds the cap, decided
/// before anything is allocated; [`Overflow::Exponent`] when the canonical exponent leaves the
/// bound.
pub(crate) fn add(a: View<'_>, b: View<'_>) -> Result<Owned, Overflow> {
    if a.is_zero() {
        return Ok(Owned::from_view(b));
    }
    if b.is_zero() {
        return Ok(Owned::from_view(a));
    }
    let low = a.exp.min(b.exp);
    let span = a.top().max(b.top()) - low + 2;
    if span > CAP_BITS {
        return Err(Overflow::Limbs);
    }
    let n = ((span + 63) / 64) as usize;
    let mut r = Owned::with_room(n);
    r.mag.resize(n, 0);
    r.exp = low;
    if a.neg == b.neg {
        r.neg = a.neg;
        let mut carry = 0u64;
        for (i, limb) in r.mag.iter_mut().enumerate() {
            let at = low + 64 * i as i64;
            let (s1, c1) = a.window(at).overflowing_add(b.window(at));
            let (s2, c2) = s1.overflowing_add(carry);
            *limb = s2;
            carry = u64::from(c1 | c2);
        }
        debug_assert_eq!(carry, 0, "the span bound left no room for the carry");
    } else {
        let (big, small) = match cmp_magnitude(a, b) {
            Ordering::Less => (b, a),
            Ordering::Equal => return Ok(Owned::ZERO),
            Ordering::Greater => (a, b),
        };
        r.neg = big.neg;
        let mut borrow = 0u64;
        for (i, limb) in r.mag.iter_mut().enumerate() {
            let at = low + 64 * i as i64;
            let (s1, b1) = big.window(at).overflowing_sub(small.window(at));
            let (s2, b2) = s1.overflowing_sub(borrow);
            *limb = s2;
            borrow = u64::from(b1 | b2);
        }
        debug_assert_eq!(
            borrow, 0,
            "the larger magnitude was subtracted from the smaller"
        );
    }
    r.normalise();
    r.check()
}

/// The product, schoolbook, canonicalised: two odd mantissas multiply to an odd one, so only a
/// leading zero limb is stripped; the exponents add.
///
/// # Errors
///
/// [`Overflow::Limbs`] when the factors' spans together exceed the cap, decided before anything
/// is allocated; [`Overflow::Exponent`] when the exponent leaves the bound.
pub(crate) fn mul(lhs: View<'_>, rhs: View<'_>) -> Result<Owned, Overflow> {
    if lhs.is_zero() || rhs.is_zero() {
        return Ok(Owned::ZERO);
    }
    if lhs.bits() + rhs.bits() > CAP_BITS {
        return Err(Overflow::Limbs);
    }
    let limbs = lhs.mag.len() + rhs.mag.len();
    let mut out = Owned::with_room(limbs);
    out.mag.resize(limbs, 0);
    for (i, &x) in lhs.mag.iter().enumerate() {
        let mut carry = 0u128;
        for (j, &y) in rhs.mag.iter().enumerate() {
            let term = u128::from(x) * u128::from(y) + u128::from(out.mag[i + j]) + carry;
            out.mag[i + j] = term as u64;
            carry = term >> 64;
        }
        out.mag[i + rhs.mag.len()] = carry as u64;
    }
    out.neg = lhs.neg != rhs.neg;
    out.exp = lhs.exp + rhs.exp;
    out.normalise();
    out.check()
}

/// Numeric order, without allocating: the sign decides first, then the position of the leading
/// bit, and only a tie walks the aligned windows down from the top.
pub(crate) fn cmp(a: View<'_>, b: View<'_>) -> Ordering {
    let (sa, sb) = (a.signum(), b.signum());
    match sa.cmp(&sb) {
        Ordering::Equal if sa != 0 => {
            let by_magnitude = cmp_magnitude(a, b);
            if sa < 0 {
                by_magnitude.reverse()
            } else {
                by_magnitude
            }
        }
        decided => decided,
    }
}

/// The magnitudes of two nonzero values.
fn cmp_magnitude(a: View<'_>, b: View<'_>) -> Ordering {
    match a.top().cmp(&b.top()) {
        Ordering::Equal => {}
        decided => return decided,
    }
    let low = a.exp.min(b.exp);
    let mut at = a.top() - 63;
    loop {
        match a.window(at).cmp(&b.window(at)) {
            Ordering::Equal => {}
            decided => return decided,
        }
        if at <= low {
            return Ordering::Equal;
        }
        at -= 64;
    }
}
