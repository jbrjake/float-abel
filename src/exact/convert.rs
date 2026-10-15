//! The conversions between [`Exact`] and the fixed-width tiers, DESIGN.md § Operations and
//! conversions. Into `Exact` is a `TryFrom`: a fixed-width state is an integer in units of the
//! grid, and it is a compact `Exact` iff the span from its lowest set bit to its highest fits 127
//! bits; with `alloc` a wider one comes in as a heap value, so the conversion refuses only the
//! exponent bound, and without it the width is refused by name. Out of `Exact` into [`Wide`] is a
//! `TryFrom` that refuses exactly where [`Wide::deposit_product`] would: below the unit or at the
//! ceiling, with a dyadic amount off the counters' grid and an amount past `i64` above their
//! ceiling. Nothing lossy has a `From`, and a refusal builds no partial state.

use super::compact::{Compact, Overflow};
use super::{Exact, PastCap};
use crate::fixed::{Fixed, OutOfWindow};
use crate::limbs;
use crate::wide::Wide;

impl Exact {
    /// `(-1)^negative · mag · 2^lsb` with the counts `[n, p, q]`: compact when it fits, else the
    /// heap form with `alloc`, else the cap by name.
    fn from_magnitude(
        negative: bool,
        mag: &[u64],
        lsb: i32,
        counts: [i64; 3],
    ) -> Result<Self, PastCap> {
        let compact = Compact::from_magnitude(negative, mag, lsb).and_then(|finite| {
            let mut narrow = [0i32; 3];
            for (c, k) in narrow.iter_mut().zip(counts) {
                *c = Self::narrow(i128::from(k)).ok_or(Overflow::Count)?;
            }
            Ok(Self::assemble(finite, narrow))
        });
        #[cfg(feature = "alloc")]
        {
            use super::big::Owned;
            Self::or_promote(compact, || {
                Self::land(super::value::Value {
                    finite: Owned::from_magnitude(negative, mag, i64::from(lsb))?,
                    amounts: counts.map(Owned::from_i64),
                })
            })
        }
        #[cfg(not(feature = "alloc"))]
        compact.map_err(Self::cap)
    }
}

impl TryFrom<&Wide> for Exact {
    type Error = PastCap;

    /// The same value: the finite part canonicalised onto the compact dyadic when it spans at
    /// most 127 bits and the counts fit `i32`, else onto the heap form.
    ///
    /// # Errors
    ///
    /// Without `alloc`, [`PastCap::Width`] when the finite part's set bits span more than 127
    /// bits, and [`PastCap::Count`] when a counter lies outside `i32` or is `i32::MIN`, which the
    /// compact form keeps out so that negation stays total. With `alloc` every `Wide` converts.
    fn try_from(wide: &Wide) -> Result<Self, PastCap> {
        let mut mag = [0u64; Wide::LIMBS];
        let negative = limbs::abs_into(wide.limbs(), &mut mag);
        let (n, p, q) = wide.specials();
        Self::from_magnitude(negative, &mag, Wide::LSB, [n, p, q])
    }
}

impl<const LSB: i32, const LIMBS: usize> TryFrom<&Fixed<LSB, LIMBS>> for Exact {
    type Error = PastCap;

    /// The same value, with no specials: the limbs canonicalised onto the compact dyadic, or
    /// onto the heap form past the width with `alloc`.
    ///
    /// # Errors
    ///
    /// [`PastCap::Exponent`] when `LSB` plus the lowest set bit's index leaves `±EXACT_MAX_EXP`,
    /// which only a grid far outside the doubles can reach; without `alloc`, also
    /// [`PastCap::Width`] when the set bits span more than 127 bits.
    fn try_from(f: &Fixed<LSB, LIMBS>) -> Result<Self, PastCap> {
        let mut mag = [0u64; LIMBS];
        let negative = limbs::abs_into(f.limbs(), &mut mag);
        Self::from_magnitude(negative, &mag, LSB, [0; 3])
    }
}

impl TryFrom<&Exact> for Wide {
    type Error = OutOfWindow;

    /// The same value: the finite part deposited onto `Wide`'s grid, and the amounts as its
    /// `i64` counters.
    ///
    /// # Errors
    ///
    /// [`OutOfWindow::OffGrid`] when the finite part has a set bit below the unit, `2^-1074`, or
    /// an amount is not an integer, as `2.5 · NaN`'s is; [`OutOfWindow::AboveCeiling`] when the
    /// finite part's magnitude reaches `2^CEILING_EXP`, the bound [`Wide::deposit_product`]
    /// applies, or an integer amount lies outside `i64`. [`OutOfWindow::NotFinite`] never occurs:
    /// specials are counted. The state is built only once every part has passed.
    fn try_from(exact: &Exact) -> Result<Self, OutOfWindow> {
        #[cfg(feature = "alloc")]
        if !exact.is_compact() {
            let p = exact.parts();
            let mut counts = [0i64; 3];
            for (c, i) in counts.iter_mut().zip(0..3) {
                let a = p.amount(i);
                *c = a.to_i64().ok_or(if a.exp < 0 {
                    OutOfWindow::OffGrid
                } else {
                    OutOfWindow::AboveCeiling
                })?;
            }
            let finite = p.finite();
            let mut wide = Self::from_parts([0; Wide::LIMBS], counts);
            wide.try_deposit_limbs(finite.neg, finite.mag, finite.exp as i32)?;
            return Ok(wide);
        }
        let (negative, mant, exp) = exact.finite().dyadic();
        let [n, p, q] = exact.counts().map(i64::from);
        let mut wide = Self::from_parts([0; Wide::LIMBS], [n, p, q]);
        wide.try_deposit_dyadic(negative, mant, exp)?;
        Ok(wide)
    }
}
