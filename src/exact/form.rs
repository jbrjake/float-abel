//! The standard traits of [`Exact`] across its two forms: `Clone` as a copy of the compact word or
//! a fresh block, structural `Eq` and `Hash` that are numeric because every operation lands the
//! canonical form, `Ord` numeric across forms, and a `Debug` that names the form.

use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};

use super::Exact;
use crate::fixed::Hex;

impl Clone for Exact {
    /// A copy of the compact form; a fresh block for the heap form.
    #[inline]
    fn clone(&self) -> Self {
        #[cfg(feature = "alloc")]
        if self.is_heap() {
            return self.clone_block();
        }
        Self {
            lo: self.lo,
            hi: self.hi,
            exp: self.exp,
            nan: self.nan,
            pos_inf: self.pos_inf,
            neg_inf: self.neg_inf,
        }
    }

    /// A copy when both are compact, with no block to free or to make.
    #[inline]
    fn clone_from(&mut self, source: &Self) {
        #[cfg(feature = "alloc")]
        if self.is_heap() || source.is_heap() {
            *self = source.clone();
            return;
        }
        self.lo = source.lo;
        self.hi = source.hi;
        self.exp = source.exp;
        self.nan = source.nan;
        self.pos_inf = source.pos_inf;
        self.neg_inf = source.neg_inf;
    }
}

impl Default for Exact {
    #[inline]
    fn default() -> Self {
        Self::ZERO
    }
}

impl PartialEq for Exact {
    /// Equal words are one value; two heap words are equal iff their blocks are; a compact and a
    /// heap word never are, by canonical form.
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        if self.lo == other.lo
            && self.hi == other.hi
            && self.exp == other.exp
            && self.nan == other.nan
            && self.pos_inf == other.pos_inf
            && self.neg_inf == other.neg_inf
        {
            return true;
        }
        #[cfg(feature = "alloc")]
        if self.is_heap() && other.is_heap() {
            return self.eq_blocks(other);
        }
        false
    }
}

impl Eq for Exact {}

impl Hash for Exact {
    fn hash<H: Hasher>(&self, state: &mut H) {
        #[cfg(feature = "alloc")]
        if self.is_heap() {
            return self.hash_block(state);
        }
        self.lo.hash(state);
        self.hi.hash(state);
        self.exp.hash(state);
        self.nan.hash(state);
        self.pos_inf.hash(state);
        self.neg_inf.hash(state);
    }
}

impl fmt::Debug for Exact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (value, inexact) = self.read();
        let about = if inexact { "≈" } else { "" };
        #[cfg(feature = "alloc")]
        if self.is_heap() {
            let p = self.parts();
            let amount = |i: usize| match p.amount(i).to_i64() {
                Some(k) => alloc::format!("{k}"),
                None => alloc::format!("≈{:e}", p.amount(i).read().0),
            };
            let finite = p.finite();
            return write!(
                f,
                "Exact({about}{value:e}; nan {}, +inf {}, -inf {}; heap {} limbs · 2^{})",
                amount(0),
                amount(1),
                amount(2),
                finite.mag.len(),
                finite.exp
            );
        }
        write!(
            f,
            "Exact({about}{value:e}; nan {}, +inf {}, -inf {}; {:?}:{:?} · 2^{})",
            self.nan,
            self.pos_inf,
            self.neg_inf,
            Hex(self.hi),
            Hex(self.lo),
            self.exp
        )
    }
}

impl PartialOrd for Exact {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Exact {
    /// Numeric on the finite part, then `(n, p, q)` lexicographic, across both forms: a total
    /// order.
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        #[cfg(feature = "alloc")]
        if self.is_heap() || other.is_heap() {
            return super::value::cmp(&self.operand(), &other.operand());
        }
        self.finite()
            .cmp(&other.finite())
            .then_with(|| self.counts().cmp(&other.counts()))
    }
}
