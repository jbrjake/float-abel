//! The heap form of [`Exact`], DESIGN.md § Exact, and the one module beside `src/simd/` where
//! `unsafe` is allowed. A canonical compact mantissa is odd or the zero, so an even nonzero `lo`
//! is no compact value, and the heap form takes that niche: `lo` holds the block's capacity in
//! limbs, doubled; `hi` holds the pointer to one allocation that is a header of four packed
//! entries (sign, exponent, limb count) for the finite part and the three special amounts, then
//! the four runs of limbs in that order; and the word's four trailing fields are zero. The block
//! is owned by exactly the word that holds it: `Clone` copies it, `Drop` frees it, and nothing
//! outside this module reads the fields of a heap word.
//!
//! The invariants every `unsafe` block relies on:
//!
//! - a word whose `lo` is even and nonzero was made by [`Exact::from_block`] from a `Vec<u64>` of
//!   capacity `lo >> 1` holding the header and the runs, and is freed only by `Drop`;
//! - the header's four counts sum, with [`HEADER`], to at most that capacity, and every run is
//!   in canonical form, because every result lands through [`Exact::land`];
//! - a compact word owns nothing, so a write over it needs no drop.

#![allow(unsafe_code)]

use alloc::vec::Vec;
use core::hash::{Hash, Hasher};
use core::mem::ManuallyDrop;
use core::{ptr, slice};

use super::big::View;
use super::compact::Overflow;
use super::value::Value;
use super::Exact;

/// Limbs of header in front of the runs: one packed entry per run.
pub(crate) const HEADER: usize = 4;

/// One header entry: the sign in the top bit, the limb count in the next 31, the exponent in the
/// low 32.
fn entry(v: View<'_>) -> u64 {
    (u64::from(v.neg) << 63) | ((v.mag.len() as u64) << 32) | u64::from(v.exp as i32 as u32)
}

/// `(sign, exponent, limb count)` of an entry.
fn unpack(e: u64) -> (bool, i64, usize) {
    (
        e >> 63 != 0,
        i64::from(e as u32 as i32),
        ((e >> 32) & 0x7FFF_FFFF) as usize,
    )
}

/// A heap word's block, borrowed: the header and the four runs, which is the whole value.
#[derive(Clone, Copy)]
pub(crate) struct Parts<'a> {
    block: &'a [u64],
}

impl<'a> Parts<'a> {
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

    /// The header and the runs: what `Eq`, `Hash` and `Clone` read.
    fn used(&self) -> &'a [u64] {
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
        let used = self.parts().used();
        let mut block = Vec::with_capacity(used.len());
        block.extend_from_slice(used);
        Self::from_block(block)
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
