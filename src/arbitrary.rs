//! `Arbitrary` on the group types, behind the `arbitrary` feature: fuzz inputs. Every draw is a
//! canonical value. `Wide` and `Fixed` draw their limbs, because every bit pattern is a value;
//! `Exact` draws the width of its mantissa and the size of its exponent and counts before their
//! bits, so that two draws fit the width together often enough for a fuzzer to reach the
//! operators and not only the cap, while the bounds are still reached.

use arbitrary::{Arbitrary, Result, Unstructured};

use crate::{Exact, Fixed, Wide, EXACT_MAX_EXP};

/// An `i32` whose magnitude is cut to a drawn number of bits, so a small value is as likely as a
/// large one.
fn scaled_i32(u: &mut Unstructured<'_>) -> Result<i32> {
    let keep = u.int_in_range(0..=31u32)?;
    Ok(i32::arbitrary(u)? >> (31 - keep))
}

#[cfg_attr(docsrs, doc(cfg(feature = "arbitrary")))]
impl<'a> Arbitrary<'a> for Exact {
    /// Any canonical word: a mantissa of a drawn width, an exponent of a drawn size inside the
    /// bound, and, half the time, three counts of a drawn size away from `i32::MIN`, so the
    /// finite words an engine mostly holds are half the draws and the zero is reached. An
    /// exhausted stream draws the zero.
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let width = u.int_in_range(0..=127u32)?;
        let mag = if width == 0 {
            0
        } else {
            u128::arbitrary(u)? & (u128::MAX >> (128 - width))
        };
        let negative = bool::arbitrary(u)?;
        let exp = scaled_i32(u)?.clamp(-EXACT_MAX_EXP, EXACT_MAX_EXP);
        let mut counts = [0i32; 3];
        if bool::arbitrary(u)? {
            for count in &mut counts {
                *count = scaled_i32(u)?.max(i32::MIN + 1);
            }
        }
        let (lo, hi, exp) = if mag == 0 {
            (0, 0, 0)
        } else {
            // Canonical form: an odd mantissa, two's complement across the two limbs.
            let m = (mag | 1) as i128;
            let m = if negative { -m } else { m };
            (m as u64, (m >> 64) as u64, exp)
        };
        Self::canonical(lo, hi, exp, counts).map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "arbitrary")))]
impl<'a> Arbitrary<'a> for Wide {
    /// Any limbs and counters, which is any value. The headroom budget is the caller's to
    /// respect, as it is on every `Wide`.
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        Ok(Self::from_parts(
            <[u64; Self::LIMBS]>::arbitrary(u)?,
            <[i64; 3]>::arbitrary(u)?,
        ))
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "arbitrary")))]
impl<'a, const LSB: i32, const LIMBS: usize> Arbitrary<'a> for Fixed<LSB, LIMBS> {
    /// Any limbs, which is any value.
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        Ok(Self::from_limbs(<[u64; LIMBS]>::arbitrary(u)?))
    }
}
