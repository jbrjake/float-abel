//! DBSP's weight traits on [`Exact`], DESIGN.md § DBSP, so a weighted batch takes it as its `R`
//! and `aggregate_linear` takes it as the accumulator, which is the `CREATE LINEAR AGGREGATE`
//! shape. The group and the ring are the std operators dbsp blankets over: `AddByRef`,
//! `AddAssignByRef`, `NegByRef` and `MulByRef<Exact>` come from the `&Exact` operators, and
//! `HasZero`, `HasOne`, `MulByRef<i64>` and `MulByRef<isize>` are written here, the last two
//! being how a row's multiplicity and a retraction reach the accumulator. The storage traits are
//! rkyv's derive on the struct for `Archive` and `Serialize`, and on the archived form a
//! hand-written `Deserialize`, `Ord` and `OrdRepr` that go through one validating constructor,
//! because `Ord` on `Exact` is hand-written and dbsp's storage searches sorted files with
//! `OrdRepr`, so the two must agree. `SizeOf` has no children, and `IsNone` and `SupportsRoaring`
//! are dbsp's own `never_none!` and `never_roaring_filter!`.
//!
//! The archived form is the 32-byte compact word itself, the wire body without its version byte.
//! rkyv and dbsp give a deserialiser no error channel (dbsp unwraps it), so a non-canonical
//! archived word, which no operation writes, is refused the way the operators refuse the cap: a
//! panic naming the invariant, from the deserialiser and the comparisons alike. A heap word
//! reaching the archive is refused the same way, by its even low limb: the archived form of the
//! heap form is the wire task's, and until it lands a promoted weight lives in DBSP's in-memory
//! batches only.

use core::cmp::Ordering;

use dbsp::algebra::{HasOne, HasZero, MulByRef};
use dbsp::dynamic::OrdRepr;
use rkyv::{Deserialize, Fallible};
use size_of::{Context, SizeOf};

use super::{ArchivedExact, Exact};

impl HasZero for Exact {
    #[inline]
    fn is_zero(&self) -> bool {
        Exact::is_zero(self)
    }

    #[inline]
    fn zero() -> Self {
        Self::ZERO
    }
}

impl HasOne for Exact {
    #[inline]
    fn one() -> Self {
        Self::from(1i8)
    }
}

impl MulByRef<i64> for Exact {
    type Output = Exact;
    #[inline]
    fn mul_by_ref(&self, k: &i64) -> Exact {
        self * *k
    }
}

impl MulByRef<isize> for Exact {
    type Output = Exact;
    #[inline]
    fn mul_by_ref(&self, k: &isize) -> Exact {
        self * *k
    }
}

impl SizeOf for Exact {
    /// The compact word owns nothing past itself; a heap word owns its block.
    fn size_of_children(&self, context: &mut Context) {
        if !self.is_compact() {
            context.add_distinct_allocation().add(self.block_bytes());
        }
    }
}

dbsp::never_none!(Exact);
dbsp::never_roaring_filter!(Exact);

impl ArchivedExact {
    /// The word the archive holds, if it is a canonical form.
    ///
    /// # Panics
    ///
    /// With the named invariant on a word `Exact::from_bytes` would refuse: an even mantissa,
    /// which is also what a heap word archived as its bits has, an exponent on a zero mantissa
    /// or outside the bound, or a count of `i32::MIN`.
    fn value(&self) -> Exact {
        Exact::canonical(
            self.lo,
            self.hi,
            self.exp,
            [self.nan, self.pos_inf, self.neg_inf],
        )
        .unwrap_or_else(|e| panic!("Exact: an archived word is not canonical: {e}"))
    }
}

impl<D: Fallible + ?Sized> Deserialize<Exact, D> for ArchivedExact {
    #[inline]
    fn deserialize(&self, _: &mut D) -> Result<Exact, D::Error> {
        Ok(self.value())
    }
}

impl PartialEq for ArchivedExact {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
    }
}

impl Eq for ArchivedExact {}

impl PartialOrd for ArchivedExact {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ArchivedExact {
    /// The order of the words held, which is [`Exact`]'s.
    fn cmp(&self, other: &Self) -> Ordering {
        self.value().cmp(&other.value())
    }
}

impl OrdRepr<Exact> for ArchivedExact {
    /// The word held against `other`, as `Ord` would compare them: the contract storage relies on
    /// to search a file sorted by `Ord` without deserialising it.
    #[inline]
    fn ord_cmp(&self, other: &Exact) -> Ordering {
        self.value().cmp(other)
    }
}
