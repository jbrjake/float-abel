//! The conversions between [`Exact`] and the fixed-width tiers, DESIGN.md § Operations and
//! conversions. Into `Exact` is a `TryFrom` that refuses past the width by name: a fixed-width
//! state is an integer in units of the grid, and it is an `Exact` iff the span from its lowest set
//! bit to its highest fits 127 bits. Out of `Exact` into [`Wide`] is a `TryFrom` that refuses
//! exactly where [`Wide::deposit_product`] would: below the unit or at the ceiling. Nothing lossy
//! has a `From`, and a refusal builds no partial state.

use super::compact::Compact;
use super::{Exact, PastCap};
use crate::fixed::{Fixed, OutOfWindow};
use crate::limbs;
use crate::wide::Wide;

impl TryFrom<&Wide> for Exact {
    type Error = PastCap;

    /// The same value: the finite part canonicalised onto the compact dyadic, and the counts
    /// narrowed.
    ///
    /// # Errors
    ///
    /// [`PastCap::Width`] when the finite part's set bits span more than 127 bits, and
    /// [`PastCap::Count`] when a counter lies outside `i32` or is `i32::MIN`, which `Exact` keeps
    /// out so that negation stays total.
    fn try_from(wide: &Wide) -> Result<Self, PastCap> {
        let mut mag = [0u64; Wide::LIMBS];
        let negative = limbs::abs_into(wide.limbs(), &mut mag);
        let finite = Compact::from_magnitude(negative, &mag, Wide::LSB).map_err(Self::cap)?;
        let (n, p, q) = wide.specials();
        let counts = [
            Self::narrow(i128::from(n))?,
            Self::narrow(i128::from(p))?,
            Self::narrow(i128::from(q))?,
        ];
        Ok(Self::assemble(finite, counts))
    }
}

impl<const LSB: i32, const LIMBS: usize> TryFrom<&Fixed<LSB, LIMBS>> for Exact {
    type Error = PastCap;

    /// The same value, with no specials: the limbs canonicalised onto the compact dyadic.
    ///
    /// # Errors
    ///
    /// [`PastCap::Width`] when the set bits span more than 127 bits, and [`PastCap::Exponent`]
    /// when `LSB` plus the lowest set bit's index leaves `±EXACT_MAX_EXP`, which only a grid far
    /// outside the doubles can reach.
    fn try_from(f: &Fixed<LSB, LIMBS>) -> Result<Self, PastCap> {
        let mut mag = [0u64; LIMBS];
        let negative = limbs::abs_into(f.limbs(), &mut mag);
        let finite = Compact::from_magnitude(negative, &mag, LSB).map_err(Self::cap)?;
        Ok(Self::assemble(finite, [0; 3]))
    }
}

impl TryFrom<&Exact> for Wide {
    type Error = OutOfWindow;

    /// The same value: the finite part deposited onto `Wide`'s grid, and the counts widened. A
    /// count is always an integer, so there is no amount to refuse.
    ///
    /// # Errors
    ///
    /// [`OutOfWindow::OffGrid`] when the finite part has a set bit below the unit, `2^-1074`, and
    /// [`OutOfWindow::AboveCeiling`] when its magnitude reaches `2^CEILING_EXP`, the bound
    /// [`Wide::deposit_product`] applies. [`OutOfWindow::NotFinite`] never occurs: specials are
    /// counted.
    fn try_from(exact: &Exact) -> Result<Self, OutOfWindow> {
        let (negative, mant, exp) = exact.finite().dyadic();
        let (n, p, q) = exact.specials();
        let mut wide =
            Self::from_parts([0; Wide::LIMBS], [i64::from(n), i64::from(p), i64::from(q)]);
        wide.try_deposit_dyadic(negative, mant, exp)?;
        Ok(wide)
    }
}
