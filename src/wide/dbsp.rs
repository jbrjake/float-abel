//! DBSP's weight traits on [`Wide`], DESIGN.md § DBSP: a weighted batch takes it as its `R` and
//! `aggregate_linear` takes it as the accumulator, the `CREATE LINEAR AGGREGATE` shape, as the
//! `Copy` weight with nothing to check at the door. `AddByRef`, `AddAssignByRef` and `NegByRef`
//! are dbsp's blankets over the by-reference operators in `wide::ops`; `HasZero`, `MulByRef<i64>`
//! and `MulByRef<isize>` are written here, the last two being how a row's multiplicity and a
//! retraction reach the accumulator; `SizeOf` counts nothing past the word; and `IsNone` and
//! `SupportsRoaring` are dbsp's own `never_none!` and `never_roaring_filter!`.
//!
//! The storage traits are rkyv's derives on the struct with the archived form declared to be the
//! word itself, `#[archive(as = "Wide")]`: a `Copy` `repr(C)` word with no invalid bit pattern is
//! its own archive, so storage writes the wire body behind its version byte, `Ord` on the archived
//! form is [`Wide`]'s, `OrdRepr`, which must agree with `Ord`, is that same `cmp`, and
//! `Deserialize` is a copy. Nothing here touches raw memory: the derive's `resolve` writes the two
//! arrays in place.
//!
//! There is no `HasOne` and no `Mul<Wide>`: the product of two words has no carrier, so `Wide` is
//! not a `RingValue`, and `neg`, `minus` and `differentiate` on a `Wide`-weighted stream, which
//! bound the weight on its ring through the leaf's `Neg`, do not compile. The weighted batch, its
//! merge and `aggregate_linear` do, which is the `SUM` shape.

use core::cmp::Ordering;

use dbsp::algebra::{HasZero, MulByRef};
use dbsp::dynamic::OrdRepr;
use rkyv::{Deserialize, Fallible};
use size_of::{Context, SizeOf};

use super::Wide;

impl HasZero for Wide {
    #[inline]
    fn is_zero(&self) -> bool {
        Wide::is_zero(self)
    }

    #[inline]
    fn zero() -> Self {
        Self::ZERO
    }
}

impl MulByRef<i64> for Wide {
    type Output = Wide;
    #[inline]
    fn mul_by_ref(&self, k: &i64) -> Wide {
        *self * *k
    }
}

impl MulByRef<isize> for Wide {
    type Output = Wide;
    #[inline]
    fn mul_by_ref(&self, k: &isize) -> Wide {
        *self * *k
    }
}

impl SizeOf for Wide {
    /// The word owns nothing past itself.
    fn size_of_children(&self, _: &mut Context) {}
}

dbsp::never_none!(Wide);
dbsp::never_roaring_filter!(Wide);

impl<D: Fallible + ?Sized> Deserialize<Wide, D> for Wide {
    /// The archived form is the word, so deserialising is a copy.
    #[inline]
    fn deserialize(&self, _: &mut D) -> Result<Wide, D::Error> {
        Ok(*self)
    }
}

impl OrdRepr<Wide> for Wide {
    /// The archived form is the word, so the order storage searches a sorted file by is `Ord`'s.
    #[inline]
    fn ord_cmp(&self, other: &Wide) -> Ordering {
        self.cmp(other)
    }
}
