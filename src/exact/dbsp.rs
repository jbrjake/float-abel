//! DBSP's weight traits on [`Exact`], DESIGN.md § DBSP, so a weighted batch takes it as its `R`
//! and `aggregate_linear` takes it as the accumulator, which is the `CREATE LINEAR AGGREGATE`
//! shape. The group and the ring are the std operators dbsp blankets over: `AddByRef`,
//! `AddAssignByRef`, `NegByRef` and `MulByRef<Exact>` come from the `&Exact` operators, and
//! `HasZero`, `HasOne`, `MulByRef<i64>` and `MulByRef<isize>` are written here, the last two
//! being how a row's multiplicity and a retraction reach the accumulator. `SizeOf` counts a heap
//! word's block, and `IsNone` and `SupportsRoaring` are dbsp's own `never_none!` and
//! `never_roaring_filter!`.
//!
//! The storage traits are hand-written, because `Exact` has two forms and `Ord` on it is
//! hand-written, and dbsp's storage searches sorted files with `OrdRepr`, so the two must agree.
//! The archived form is the 32-byte word: for a compact value the word itself, the wire body
//! without its version byte; for a heap value the word with its tag in `lo` and, in `hi`, the
//! offset of the block `Serialize` wrote into the archive, the same header and runs the wire
//! carries. Both forms reach a value through one validating constructor each, [`Exact::canonical`]
//! and [`Parts::canonical`], the wire decoder's. rkyv and dbsp give a deserialiser no error channel
//! (dbsp unwraps it), so an archived word that is not canonical, which no operation writes, is
//! refused the way the operators refuse the cap: a panic naming the invariant, from the
//! deserialiser and the comparisons alike. An archive is trusted as rkyv trusts every archive, so
//! a heap word's block is bounded by the cap before it is read and validated before it is used,
//! and the two raw-memory pieces, writing the word and reaching the block, live in `exact::heap`.

use core::cmp::Ordering;

use dbsp::algebra::{HasOne, HasZero, MulByRef};
use dbsp::dynamic::OrdRepr;
use rkyv::ser::Serializer;
use rkyv::{Deserialize, Fallible, Serialize};
use size_of::{Context, SizeOf};

use super::heap::{Parts, HEADER, MAX_BLOCK};
use super::value::{self, Operand};
use super::Exact;
use crate::wire::WireError;

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

/// The rkyv archive of an [`Exact`], `rkyv::Archived<Exact>`: the 32-byte word, `repr(C)`. A
/// compact value archives as the word itself, the wire body without its version byte. A heap
/// value archives with `lo` holding the block's used length doubled (even and nonzero, the heap
/// tag as in memory), `hi` the block's offset from the `hi` field, as rkyv's relative pointers
/// are, the trailing fields zero, and the block, the heap form's wire body behind its tag, 8-aligned
/// earlier in the archive.
#[cfg_attr(docsrs, doc(cfg(feature = "dbsp")))]
#[derive(Debug)]
#[repr(C)]
pub struct ArchivedExact {
    pub(super) lo: u64,
    pub(super) hi: u64,
    pub(super) exp: i32,
    pub(super) nan: i32,
    pub(super) pos_inf: i32,
    pub(super) neg_inf: i32,
}

/// What `Serialize` hands `resolve`: the position of a heap word's block in the archive, and
/// nothing for the compact form.
#[cfg_attr(docsrs, doc(cfg(feature = "dbsp")))]
#[derive(Debug)]
pub struct ExactResolver {
    pub(super) block: usize,
}

impl<S: Serializer + ?Sized> Serialize<S> for Exact {
    /// A heap word's block written out of line, 8-aligned, each limb as the native `u64` rkyv
    /// archives one as; a compact word writes nothing here.
    fn serialize(&self, serializer: &mut S) -> Result<ExactResolver, S::Error> {
        if self.is_compact() {
            return Ok(ExactResolver { block: 0 });
        }
        let block = serializer.align_for::<u64>()?;
        for &limb in self.parts().used() {
            serializer.write(&limb.to_ne_bytes())?;
        }
        Ok(ExactResolver { block })
    }
}

/// An archived word's value, validated: the compact word, or the heap block borrowed from the
/// archive without a copy.
enum Form<'a> {
    Compact(Exact),
    Heap(Parts<'a>),
}

impl Form<'_> {
    fn operand(&self) -> Operand<'_> {
        match self {
            Self::Compact(x) => x.operand(),
            Self::Heap(p) => Operand::Heap(*p),
        }
    }

    /// [`Exact`]'s order, with two compact words on its compact path.
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Compact(a), Self::Compact(b)) => a.cmp(b),
            _ => value::cmp(&self.operand(), &other.operand()),
        }
    }

    fn cmp_value(&self, other: &Exact) -> Ordering {
        match self {
            Self::Compact(a) => a.cmp(other),
            Self::Heap(_) => value::cmp(&self.operand(), &other.operand()),
        }
    }
}

/// The named invariant an archived word that is not a value ends in.
#[cold]
#[inline(never)]
fn refuse(why: WireError) -> ! {
    panic!("Exact: an archived word is not canonical: {why}")
}

impl ArchivedExact {
    /// The form the archive holds, if it is canonical.
    ///
    /// # Panics
    ///
    /// With the named invariant on a word `Exact::from_bytes` would refuse: a compact word with
    /// an even mantissa, an exponent on a zero mantissa or outside the bound, or a count of
    /// `i32::MIN`; a heap word with a nonzero trailing field, a length outside the header and the
    /// cap, an offset that is not a whole number of limbs, or a block that is not a canonical
    /// heap form. The word's own checks run before its block is reached.
    fn form(&self) -> Form<'_> {
        if self.lo & 1 == 0 && self.lo != 0 {
            if self.exp != 0 || self.nan != 0 || self.pos_inf != 0 || self.neg_inf != 0 {
                refuse(WireError::Form);
            }
            let used = (self.lo >> 1) as usize;
            if used < HEADER || self.hi % 8 != 0 {
                refuse(WireError::Form);
            }
            if used > MAX_BLOCK {
                refuse(WireError::Limbs);
            }
            return Form::Heap(Parts::canonical(self.block()).unwrap_or_else(|e| refuse(e)));
        }
        let word = Exact::canonical(
            self.lo,
            self.hi,
            self.exp,
            [self.nan, self.pos_inf, self.neg_inf],
        );
        Form::Compact(word.unwrap_or_else(|e| refuse(e)))
    }

    /// The word the archive holds: a copy of the compact word, or a heap word over a fresh block.
    fn value(&self) -> Exact {
        match self.form() {
            Form::Compact(x) => x,
            Form::Heap(p) => Exact::from_parts(p),
        }
    }
}

impl<D: Fallible + ?Sized> Deserialize<Exact, D> for ArchivedExact {
    #[inline]
    fn deserialize(&self, _: &mut D) -> Result<Exact, D::Error> {
        Ok(self.value())
    }
}

impl PartialEq for ArchivedExact {
    /// Equal iff the values are: `Ord`'s tie, so the two agree, and no block is copied.
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for ArchivedExact {}

impl PartialOrd for ArchivedExact {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ArchivedExact {
    /// The order of the words held, which is [`Exact`]'s, over the archived blocks in place.
    fn cmp(&self, other: &Self) -> Ordering {
        self.form().cmp(&other.form())
    }
}

impl OrdRepr<Exact> for ArchivedExact {
    /// The word held against `other`, as `Ord` would compare them: the contract storage relies on
    /// to search a file sorted by `Ord` without deserialising it.
    #[inline]
    fn ord_cmp(&self, other: &Exact) -> Ordering {
        self.form().cmp_value(other)
    }
}
