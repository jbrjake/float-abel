//! DBSP's weight traits on [`Fixed`], DESIGN.md § DBSP: a weighted batch takes it as its `R` and
//! `aggregate_linear` takes it as the accumulator, the `CREATE LINEAR AGGREGATE` shape, as the
//! declared window, with the door check per value where the row becomes a weight,
//! `Fixed::try_from` inside the aggregate's closure. `AddByRef`, `AddAssignByRef` and `NegByRef`
//! are dbsp's blankets over the by-reference operators in `fixed::ops`; `HasZero`, `MulByRef<i64>`
//! and `MulByRef<isize>` are written here, the last two being how a row's multiplicity and a
//! retraction reach the accumulator; `SizeOf` counts nothing past the word; and `IsNone` and
//! `SupportsRoaring` are written out, because dbsp's `never_none!` and `never_roaring_filter!`
//! take a type with no free parameters.
//!
//! The storage traits are rkyv's derives on the struct with the archived form declared to be the
//! word itself, `#[archive(as = "Fixed<LSB, LIMBS>")]`: a `Copy` `repr(C)` word with no invalid
//! bit pattern is its own archive, so storage writes the wire body behind its version byte, `Ord`
//! on the archived form is [`Fixed`]'s, `OrdRepr`, which must agree with `Ord`, is that same
//! `cmp`, and `Deserialize` is a copy. Nothing here touches raw memory.
//!
//! There is no `HasOne` and no `Mul<Fixed>`: the product of two windows is not a value in either,
//! so `Fixed` is not a `RingValue`, and `neg`, `minus` and `differentiate` on a `Fixed`-weighted
//! stream, which bound the weight on its ring through the leaf's `Neg`, do not compile. The
//! weighted batch, its merge and `aggregate_linear` do, which is the `SUM` shape.

use core::cmp::Ordering;

use dbsp::algebra::{HasZero, MulByRef};
use dbsp::dynamic::OrdRepr;
use dbsp::utils::{IsNone, SupportsRoaring};
use rkyv::{Deserialize, Fallible};
use size_of::{Context, SizeOf};

use super::Fixed;

impl<const LSB: i32, const LIMBS: usize> HasZero for Fixed<LSB, LIMBS> {
    #[inline]
    fn is_zero(&self) -> bool {
        Fixed::is_zero(self)
    }

    #[inline]
    fn zero() -> Self {
        Self::ZERO
    }
}

impl<const LSB: i32, const LIMBS: usize> MulByRef<i64> for Fixed<LSB, LIMBS> {
    type Output = Self;
    #[inline]
    fn mul_by_ref(&self, k: &i64) -> Self {
        *self * *k
    }
}

impl<const LSB: i32, const LIMBS: usize> MulByRef<isize> for Fixed<LSB, LIMBS> {
    type Output = Self;
    #[inline]
    fn mul_by_ref(&self, k: &isize) -> Self {
        *self * *k
    }
}

impl<const LSB: i32, const LIMBS: usize> SizeOf for Fixed<LSB, LIMBS> {
    /// The word owns nothing past itself.
    fn size_of_children(&self, _: &mut Context) {}
}

impl<const LSB: i32, const LIMBS: usize> IsNone for Fixed<LSB, LIMBS> {
    type Inner = Self;

    #[inline]
    fn is_none(&self) -> bool {
        false
    }

    #[inline]
    fn unwrap_or_self(&self) -> &Self {
        self
    }

    #[inline]
    fn from_inner(inner: Self) -> Self {
        inner
    }
}

impl<const LSB: i32, const LIMBS: usize> SupportsRoaring for Fixed<LSB, LIMBS> {}

impl<const LSB: i32, const LIMBS: usize, D: Fallible + ?Sized> Deserialize<Fixed<LSB, LIMBS>, D>
    for Fixed<LSB, LIMBS>
{
    /// The archived form is the word, so deserialising is a copy.
    #[inline]
    fn deserialize(&self, _: &mut D) -> Result<Fixed<LSB, LIMBS>, D::Error> {
        Ok(*self)
    }
}

impl<const LSB: i32, const LIMBS: usize> OrdRepr<Fixed<LSB, LIMBS>> for Fixed<LSB, LIMBS> {
    /// The archived form is the word, so the order storage searches a sorted file by is `Ord`'s.
    #[inline]
    fn ord_cmp(&self, other: &Fixed<LSB, LIMBS>) -> Ordering {
        self.cmp(other)
    }
}
