//! The heap form of [`Exact`], DESIGN.md § Exact, and the one module beside `src/simd/` where
//! `unsafe` is allowed. A canonical compact mantissa is odd or the zero, so an even nonzero `lo`
//! is no compact value, and the heap form takes that niche: `lo` holds the block's capacity in
//! limbs, doubled; `hi` holds the pointer to one allocation that is a header of four packed
//! entries (sign, exponent, limb count) for the finite part and the three special amounts, then
//! the four runs of limbs in that order; and the word's four trailing fields are zero. The block
//! is owned by exactly the word that holds it: `Clone` copies it, `Drop` frees it, and nothing
//! outside this module reads the fields of a heap word.
//!
//! The block's used prefix, the header and the runs, is also the heap form's wire body behind its
//! tag and its rkyv archive behind the word, so one layout serves memory, the wire and storage,
//! and one validator, [`check_runs`], gates every block that comes from outside.
//!
//! The invariants every `unsafe` block relies on:
//!
//! - a word whose `lo` is even and nonzero was made by [`Exact::from_block`] from a `Vec<u64>` of
//!   capacity `lo >> 1` holding the header and the runs, and is freed only by `Drop`;
//! - the header's four counts sum, with [`HEADER`], to at most that capacity, and every run is
//!   in canonical form, because every result lands through [`Exact::land`] and every block from
//!   outside through [`Exact::from_canonical_block`];
//! - a compact word owns nothing, so a write over it needs no drop;
//! - an archived heap word was written by `resolve` beside the block `serialize` wrote, so its
//!   offset reaches that block inside the same archive, which rkyv reads as it was written.

#![allow(unsafe_code)]

use alloc::vec::Vec;
use core::hash::{Hash, Hasher};
use core::mem::ManuallyDrop;
use core::{ptr, slice};

use super::big::View;
use super::compact::Overflow;
#[cfg(feature = "dbsp")]
use super::dbsp::{ArchivedExact, ExactResolver};
use super::value::Value;
use super::{Exact, EXACT_MAX_EXP, EXACT_MAX_LIMBS};
use crate::wire::WireError;

/// Limbs of header in front of the runs: one packed entry per run.
pub(crate) const HEADER: usize = 4;

/// The most limbs a block holds: the header and four runs at the cap. An archived block is bounded
/// by it before it is read.
#[cfg(feature = "dbsp")]
pub(crate) const MAX_BLOCK: usize = HEADER + 4 * EXACT_MAX_LIMBS;

/// The compact form's width in bits, which a heap finite part must exceed.
const WIDTH: i64 = 127;

const _: () = assert!(
    EXACT_MAX_LIMBS < 1 << 31,
    "EXACT_MAX_LIMBS must fit an entry's 31-bit limb count"
);

/// One header entry: the sign in the top bit, the limb count in the next 31, the exponent in the
/// low 32.
fn entry(v: View<'_>) -> u64 {
    (u64::from(v.neg) << 63) | ((v.mag.len() as u64) << 32) | u64::from(v.exp as i32 as u32)
}

/// `(sign, exponent, limb count)` of an entry.
pub(crate) fn unpack(e: u64) -> (bool, i64, usize) {
    (
        e >> 63 != 0,
        i64::from(e as u32 as i32),
        ((e >> 32) & 0x7FFF_FFFF) as usize,
    )
}

/// The canonical-form rules of a block's four runs, over any source of limbs: `limb(i)` is limb
/// `i` behind the header, of which there are `limbs`. Every count under the cap, the counts
/// summing to `limbs`, then each run in order normalised (empty with no sign or exponent for the
/// zero; else an odd first limb, a nonzero last limb and an exponent inside the bound), and the
/// whole not fitting the compact form, which is the operations' own rule: a finite part of at
/// most [`WIDTH`] bits with every amount an `i32` count demotes. The rules run in that order so
/// the first failing one names the refusal. The wire decoder runs this over the bytes before it
/// allocates anything, and the archive over its block.
///
/// # Errors
///
/// [`WireError::Limbs`], [`WireError::Form`], [`WireError::EvenMantissa`] and
/// [`WireError::Exponent`], as above.
pub(crate) fn check_runs(
    header: &[u64; HEADER],
    limbs: usize,
    limb: impl Fn(usize) -> u64,
) -> Result<(), WireError> {
    let mut total = 0usize;
    for &e in header {
        let len = unpack(e).2;
        if len > EXACT_MAX_LIMBS {
            return Err(WireError::Limbs);
        }
        total += len;
    }
    if total != limbs {
        return Err(WireError::Form);
    }
    let max = i64::from(EXACT_MAX_EXP);
    let mut at = 0;
    let mut fits = true;
    for (i, &e) in header.iter().enumerate() {
        let (neg, exp, len) = unpack(e);
        if len == 0 {
            if neg || exp != 0 {
                return Err(WireError::Form);
            }
            continue;
        }
        let (first, last) = (limb(at), limb(at + len - 1));
        at += len;
        if first & 1 == 0 {
            return Err(WireError::EvenMantissa);
        }
        if last == 0 {
            return Err(WireError::Form);
        }
        if !(-max..=max).contains(&exp) {
            return Err(WireError::Exponent);
        }
        let bits = 64 * len as i64 - i64::from(last.leading_zeros());
        fits &= if i == 0 {
            bits <= WIDTH
        } else {
            len == 1 && exp >= 0 && exp + bits <= 31
        };
    }
    if fits {
        return Err(WireError::Form);
    }
    Ok(())
}

/// A heap word's block, borrowed: the header and the four runs, which is the whole value.
#[derive(Clone, Copy)]
pub(crate) struct Parts<'a> {
    block: &'a [u64],
}

impl<'a> Parts<'a> {
    /// `block` as a value, if it is a canonical heap form: the header, then [`check_runs`].
    ///
    /// # Errors
    ///
    /// [`WireError::Form`] for a block shorter than its header, and [`check_runs`]'s.
    pub(crate) fn canonical(block: &'a [u64]) -> Result<Self, WireError> {
        let Some(header) = block.get(..HEADER) else {
            return Err(WireError::Form);
        };
        let header: &[u64; HEADER] = header.try_into().expect("HEADER limbs");
        check_runs(header, block.len() - HEADER, |i| block[HEADER + i])?;
        Ok(Self { block })
    }

    fn run(&self, i: usize) -> View<'a> {
        let mut at = HEADER;
        for &e in &self.block[..i] {
            at += unpack(e).2;
        }
        let (neg, exp, len) = unpack(self.block[i]);
        View {
            neg,
            exp,
            mag: &self.block[at..at + len],
        }
    }

    pub(crate) fn finite(&self) -> View<'a> {
        self.run(0)
    }

    /// Amount `i` of `[n, p, q]`.
    pub(crate) fn amount(&self, i: usize) -> View<'a> {
        self.run(1 + i)
    }

    /// The header and the runs: what `Eq`, `Hash`, `Clone` and the encoders read.
    pub(crate) fn used(&self) -> &'a [u64] {
        self.block
    }
}

impl Exact {
    /// Whether the word is in the heap form: `lo` even and nonzero, which no compact word is.
    #[inline]
    pub(crate) fn is_heap(&self) -> bool {
        self.lo & 1 == 0 && self.lo != 0
    }

    fn capacity(&self) -> usize {
        (self.lo >> 1) as usize
    }

    fn block_ptr(&self) -> *mut u64 {
        self.hi as usize as *mut u64
    }

    /// The bytes a heap word's block occupies.
    #[cfg(feature = "dbsp")]
    pub(crate) fn block_bytes(&self) -> usize {
        self.capacity() * core::mem::size_of::<u64>()
    }

    /// The block of a heap word.
    pub(crate) fn parts(&self) -> Parts<'_> {
        debug_assert!(self.is_heap(), "parts of a compact word");
        let p = self.block_ptr().cast_const();
        // SAFETY: a heap word's `hi` is the pointer `from_block` took from a `Vec<u64>` of at
        // least HEADER limbs, live until this word drops, and `&self` keeps any `&mut` away.
        let header = unsafe { slice::from_raw_parts(p, HEADER) };
        let used = HEADER + header.iter().map(|&e| unpack(e).2).sum::<usize>();
        debug_assert!(
            used <= self.capacity(),
            "a block's runs outrun its capacity"
        );
        // SAFETY: as above, and `from_block` wrote `used` limbs inside the capacity.
        let block = unsafe { slice::from_raw_parts(p, used) };
        Parts { block }
    }

    /// A word over `block`, which holds the header and the runs: the word takes the allocation.
    fn from_block(block: Vec<u64>) -> Self {
        debug_assert!(block.len() >= HEADER, "a block without its header");
        let mut block = ManuallyDrop::new(block);
        Self {
            lo: (block.capacity() as u64) << 1,
            hi: block.as_mut_ptr() as usize as u64,
            exp: 0,
            nan: 0,
            pos_inf: 0,
            neg_inf: 0,
        }
    }

    /// A word over `block` from outside, the one gate every decoded block passes: the heap form
    /// when the block is canonical, the refusal otherwise, with the block dropped.
    ///
    /// # Errors
    ///
    /// [`Parts::canonical`]'s.
    pub(crate) fn from_canonical_block(block: Vec<u64>) -> Result<Self, WireError> {
        Parts::canonical(&block)?;
        Ok(Self::from_block(block))
    }

    /// A word over a fresh block of exactly `p`'s used length.
    pub(crate) fn from_parts(p: Parts<'_>) -> Self {
        let used = p.used();
        let mut block = Vec::with_capacity(used.len());
        block.extend_from_slice(used);
        Self::from_block(block)
    }

    /// A result in its canonical form: the compact word when the finite part spans at most 127
    /// bits and every amount is an `i32` count, else the heap form over one block, which takes the
    /// finite part's buffer when it has room in front for the header.
    ///
    /// # Errors
    ///
    /// [`Overflow::Exponent`] when the finite part's exponent leaves the bound.
    pub(crate) fn land(v: Value) -> Result<Self, Overflow> {
        let finite = v.finite.view().to_compact()?;
        let counts = [
            v.amounts[0].view().to_count(),
            v.amounts[1].view().to_count(),
            v.amounts[2].view().to_count(),
        ];
        if let (Some(f), [Some(n), Some(p), Some(q)]) = (finite, counts) {
            return Ok(Self::assemble(f, [n, p, q]));
        }
        let Value { finite, amounts } = v;
        let header = [
            entry(finite.view()),
            entry(amounts[0].view()),
            entry(amounts[1].view()),
            entry(amounts[2].view()),
        ];
        let mut block = finite.mag;
        let len = block.len();
        block.resize(len + HEADER, 0);
        block.copy_within(..len, HEADER);
        block[..HEADER].copy_from_slice(&header);
        for a in &amounts {
            block.extend_from_slice(&a.mag);
        }
        Ok(Self::from_block(block))
    }

    /// A copy of a heap word over a fresh block of exactly the used length.
    pub(crate) fn clone_block(&self) -> Self {
        Self::from_parts(self.parts())
    }

    /// `with` written over a compact word, which owns nothing, so the drop glue is skipped.
    #[inline]
    pub(crate) fn write_compact(&mut self, with: Self) {
        debug_assert!(
            !self.is_heap(),
            "a write over a heap word would leak its block"
        );
        // SAFETY: `self` is compact, so it owns no block and nothing is leaked by not dropping
        // it; `with` is moved in whole.
        unsafe { ptr::write(self, with) };
    }

    /// Negation of a heap word in place: the sign bit of each nonzero run's entry flips.
    pub(crate) fn negate_block(&mut self) {
        debug_assert!(self.is_heap(), "negate_block on a compact word");
        // SAFETY: a heap word's block through `&mut self`, so no other reference to it is live,
        // and its first HEADER limbs are the header.
        let header = unsafe { slice::from_raw_parts_mut(self.block_ptr(), HEADER) };
        for e in header {
            if unpack(*e).2 != 0 {
                *e ^= 1 << 63;
            }
        }
    }

    /// Two heap words hold one value iff their blocks' used prefixes agree.
    pub(crate) fn eq_blocks(&self, other: &Self) -> bool {
        self.parts().used() == other.parts().used()
    }

    pub(crate) fn hash_block<H: Hasher>(&self, state: &mut H) {
        self.parts().used().hash(state);
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl Drop for Exact {
    #[inline]
    fn drop(&mut self) {
        if self.is_heap() {
            // SAFETY: a heap word's pointer and capacity are the ones `from_block` took from a
            // `Vec<u64>`, and this is the one place they are given back; a length of zero reads
            // no limb.
            drop(unsafe { Vec::from_raw_parts(self.block_ptr(), 0, self.capacity()) });
        }
    }
}

/// The two pieces of the rkyv archive that touch raw memory, DESIGN.md § DBSP: writing the
/// archived word, and reaching an archived heap word's block. The rest of the adapter is safe
/// code in `exact::dbsp`.
#[cfg(feature = "dbsp")]
impl rkyv::Archive for Exact {
    type Archived = ArchivedExact;
    type Resolver = ExactResolver;

    /// The archived word: the compact word as it is, so the archive of a compact value is the
    /// wire body; a heap word with `lo` the block's used length doubled, `hi` the block's offset
    /// from the `hi` field, as rkyv's relative pointers are, and the trailing fields zero.
    unsafe fn resolve(&self, pos: usize, resolver: ExactResolver, out: *mut ArchivedExact) {
        let word = if self.is_heap() {
            let used = self.parts().used().len() as u64;
            let from = pos + core::mem::offset_of!(ArchivedExact, hi);
            ArchivedExact {
                lo: used << 1,
                hi: (resolver.block as i64).wrapping_sub(from as i64) as u64,
                exp: 0,
                nan: 0,
                pos_inf: 0,
                neg_inf: 0,
            }
        } else {
            ArchivedExact {
                lo: self.lo,
                hi: self.hi,
                exp: self.exp,
                nan: self.nan,
                pos_inf: self.pos_inf,
                neg_inf: self.neg_inf,
            }
        };
        // SAFETY: rkyv's contract for `resolve`: `out` is aligned, writable memory for one
        // `ArchivedExact`, and `resolver` is what `serialize` returned for this value.
        unsafe { out.write(word) };
    }
}

#[cfg(feature = "dbsp")]
impl ArchivedExact {
    /// The block of an archived heap word: `lo >> 1` limbs at `hi`'s offset from the `hi` field,
    /// which is a whole number of limbs because both sit 8-aligned in the archive. The caller has
    /// checked the tag, the offset's alignment, and bounded the length by [`MAX_BLOCK`].
    pub(crate) fn block(&self) -> &[u64] {
        let used = (self.lo >> 1) as usize;
        let offset = self.hi as i64 as isize;
        debug_assert!(
            self.lo & 1 == 0 && (HEADER..=MAX_BLOCK).contains(&used) && offset % 8 == 0,
            "block of a word that is not a bounded, aligned heap word"
        );
        let base: *const u64 = ptr::addr_of!(self.hi);
        // SAFETY: an archived heap word was written by `resolve` from a block of `used` limbs
        // that `serialize` wrote into the same archive, 8-aligned, this many bytes from the `hi`
        // field; rkyv's contract is that an archive is read as it was written, and the archive
        // outlives `&self`.
        unsafe { slice::from_raw_parts(base.offset(offset / 8), used) }
    }
}
