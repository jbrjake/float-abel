//! The 32-byte niche form `N<INLINE, DROP>`: the word, its two block shapes, the compact fast
//! paths over `Exact`, the heap path over `Big`, and the traits differential asks of a diff.

use std::cmp::Ordering;
use std::fmt;
use std::mem;
use std::ptr;
use std::slice;

use differential_dataflow::difference::{Abelian, IsZero, Monoid, Multiply, Semigroup};
use float_abel::{Exact, PastCap, Wide};
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::big::Big;

// ------------------------------------------------------------------------ the niche form N

/// The 32-byte niche form. See the module doc.
#[repr(C)]
pub(crate) struct N<const INLINE: bool, const DROP: bool> {
    lo: u64,
    hi: u64,
    exp: i32,
    nan: i32,
    pos: i32,
    neg: i32,
}

const _: () = assert!(mem::size_of::<N<true, true>>() == 32 && mem::size_of::<Exact>() == 32);

/// The heap form's fields, borrowed from wherever the shape keeps them.
struct Parts<'a> {
    neg: bool,
    exp: i32,
    counts: [i32; 3],
    limbs: &'a [u64],
}

// SAFETY: the block behind `hi` is owned by exactly this word, allocated by `heap` and freed by
// `Drop`; nothing shares it, so moving the word between threads is sound.
unsafe impl<const INLINE: bool, const DROP: bool> Send for N<INLINE, DROP> {}
unsafe impl<const INLINE: bool, const DROP: bool> Sync for N<INLINE, DROP> {}

impl<const INLINE: bool, const DROP: bool> N<INLINE, DROP> {
    pub(crate) const ZERO: Self = Self {
        lo: 0,
        hi: 0,
        exp: 0,
        nan: 0,
        pos: 0,
        neg: 0,
    };

    #[inline]
    pub(crate) fn is_heap(&self) -> bool {
        self.lo & 1 == 0 && self.lo != 0
    }

    /// The compact word as the `Exact` it is bit for bit.
    #[inline]
    pub(crate) fn as_exact(&self) -> &Exact {
        debug_assert!(!self.is_heap());
        // SAFETY: both are repr(C) {u64, u64, i32, i32, i32, i32}, and a compact word is a
        // canonical Exact by construction.
        unsafe { &*(self as *const Self).cast::<Exact>() }
    }

    #[inline]
    pub(crate) fn from_exact(e: Exact) -> Self {
        // SAFETY: same layout; an Exact's lo is odd or zero, so it is never read as heap.
        unsafe { mem::transmute::<Exact, Self>(e) }
    }

    #[inline]
    fn counts(&self) -> [i32; 3] {
        if self.is_heap() {
            self.parts().counts
        } else {
            [self.nan, self.pos, self.neg]
        }
    }

    /// The heap form's fields. Shape A reads the word; shape B reads the block's header.
    #[inline]
    fn parts(&self) -> Parts<'_> {
        debug_assert!(self.is_heap());
        let p = self.hi as *const u64;
        if INLINE {
            let len = (self.lo >> 2) as usize;
            // SAFETY: a heap word's hi is the pointer heap() stored, with lo its limb count.
            let limbs = unsafe { slice::from_raw_parts(p, len) };
            Parts {
                neg: self.lo & 2 != 0,
                exp: self.exp,
                counts: [self.nan, self.pos, self.neg],
                limbs,
            }
        } else {
            let len = (self.lo >> 1) as usize;
            // SAFETY: as above, with three header limbs before the magnitude.
            let (h0, h1, h2, limbs) =
                unsafe { (*p, *p.add(1), *p.add(2), slice::from_raw_parts(p.add(3), len)) };
            Parts {
                neg: (h0 >> 32) & 1 != 0,
                exp: h0 as u32 as i32,
                counts: [h1 as u32 as i32, (h1 >> 32) as u32 as i32, h2 as u32 as i32],
                limbs,
            }
        }
    }

    /// The whole block, for the free.
    fn block_len(&self) -> usize {
        if INLINE {
            (self.lo >> 2) as usize
        } else {
            (self.lo >> 1) as usize + 3
        }
    }

    /// A heap word over one fresh allocation.
    fn heap(neg: bool, exp: i32, counts: [i32; 3], limbs: &[u64]) -> Self {
        debug_assert!(limbs.len() >= 3);
        if INLINE {
            let block: Box<[u64]> = limbs.into();
            let ptr = Box::into_raw(block).cast::<u64>();
            Self {
                lo: ((limbs.len() as u64) << 2) | (u64::from(neg) << 1),
                hi: ptr as u64,
                exp,
                nan: counts[0],
                pos: counts[1],
                neg: counts[2],
            }
        } else {
            let mut v = Vec::with_capacity(limbs.len() + 3);
            v.push(u64::from(exp as u32) | (u64::from(neg) << 32));
            v.push(u64::from(counts[0] as u32) | (u64::from(counts[1] as u32) << 32));
            v.push(u64::from(counts[2] as u32));
            v.extend_from_slice(limbs);
            let block: Box<[u64]> = v.into_boxed_slice();
            let ptr = Box::into_raw(block).cast::<u64>();
            Self {
                lo: (limbs.len() as u64) << 1,
                hi: ptr as u64,
                exp: 0,
                nan: 0,
                pos: 0,
                neg: 0,
            }
        }
    }

    /// The canonical word for a working-form value: compact iff it fits 127 bits.
    fn land(b: &Big, counts: [i32; 3]) -> Self {
        if b.bits() <= 127 {
            let mag = u128::from(b.mag[0]) | (u128::from(b.mag[1]) << 64);
            let m = if b.neg { -(mag as i128) } else { mag as i128 };
            Self {
                lo: m as u64,
                hi: (m >> 64) as u64,
                exp: b.exp as i32,
                nan: counts[0],
                pos: counts[1],
                neg: counts[2],
            }
        } else {
            Self::heap(b.neg, b.exp as i32, counts, b.limbs())
        }
    }

    fn big(&self) -> Big {
        if self.is_heap() {
            let p = self.parts();
            Big::from_limbs(p.neg, p.exp, p.limbs)
        } else {
            Big::from_compact(self.lo, self.hi, self.exp)
        }
    }

    fn add_counts(a: [i32; 3], b: [i32; 3]) -> [i32; 3] {
        [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
    }

    /// The slow path: both sides through the working form.
    #[inline(never)]
    fn add_slow(&self, rhs: &Self) -> Self {
        let sum = Big::add(&self.big(), &rhs.big());
        Self::land(&sum, Self::add_counts(self.counts(), rhs.counts()))
    }

    #[inline(never)]
    fn mul_slow(&self, k: i64) -> Self {
        let prod = self.big().mul_i64(k);
        let c = self.counts();
        let k32 = k as i32;
        Self::land(&prod, [c[0] * k32, c[1] * k32, c[2] * k32])
    }

    pub(crate) fn to_f64(&self) -> f64 {
        if self.is_heap() {
            let p = self.parts();
            let mut w = Wide::ZERO;
            // Exact on these bands: each 32-bit half of a limb is a double, deposited at its
            // place, and Wide rounds once at the read.
            for (i, &l) in p.limbs.iter().enumerate() {
                let e = p.exp + 64 * i as i32;
                for (half, shift) in [(l & 0xffff_ffff, 0), (l >> 32, 32)] {
                    let x = half as f64 * 2f64.powi(e + shift);
                    w.deposit(if p.neg { -x } else { x });
                }
            }
            w.to_f64()
        } else {
            self.as_exact().to_f64()
        }
    }
}

impl<const INLINE: bool, const DROP: bool> Drop for N<INLINE, DROP> {
    /// `DROP = false` is the diagnostic that leaks every block: it prices the drop glue a
    /// pointer-owning word puts into differential's vectors, apart from the tag test.
    #[inline]
    fn drop(&mut self) {
        if DROP && self.is_heap() {
            let len = self.block_len();
            // SAFETY: hi is the pointer heap() took from Box::into_raw over exactly len limbs.
            unsafe {
                drop(Box::from_raw(ptr::slice_from_raw_parts_mut(
                    self.hi as *mut u64,
                    len,
                )));
            }
        }
    }
}

impl<const INLINE: bool, const DROP: bool> Clone for N<INLINE, DROP> {
    #[inline]
    fn clone(&self) -> Self {
        if self.is_heap() {
            let p = self.parts();
            Self::heap(p.neg, p.exp, p.counts, p.limbs)
        } else {
            Self {
                lo: self.lo,
                hi: self.hi,
                exp: self.exp,
                nan: self.nan,
                pos: self.pos,
                neg: self.neg,
            }
        }
    }

    /// Two compact words copy without dropping anything, which is what consolidation does on
    /// every run of equal keys.
    #[inline]
    fn clone_from(&mut self, source: &Self) {
        if !self.is_heap() && !source.is_heap() {
            // SAFETY: neither owns a block, so a plain copy of the 32 bytes is a clone.
            unsafe { ptr::copy_nonoverlapping(source, self, 1) };
        } else {
            *self = source.clone();
        }
    }
}

impl<const INLINE: bool, const DROP: bool> PartialEq for N<INLINE, DROP> {
    /// Equal words are equal values: two compact words, or one block. Otherwise only two heap
    /// words with equal blocks are equal; a compact and a heap word never are, by canonical form.
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        if self.lo == other.lo
            && self.hi == other.hi
            && self.exp == other.exp
            && self.nan == other.nan
            && self.pos == other.pos
            && self.neg == other.neg
        {
            return true;
        }
        if self.is_heap() && other.is_heap() {
            let (a, b) = (self.parts(), other.parts());
            a.neg == b.neg && a.exp == b.exp && a.counts == b.counts && a.limbs == b.limbs
        } else {
            false
        }
    }
}
impl<const INLINE: bool, const DROP: bool> Eq for N<INLINE, DROP> {}

impl<const INLINE: bool, const DROP: bool> PartialOrd for N<INLINE, DROP> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<const INLINE: bool, const DROP: bool> Ord for N<INLINE, DROP> {
    fn cmp(&self, other: &Self) -> Ordering {
        if !self.is_heap() && !other.is_heap() {
            return self.as_exact().cmp(other.as_exact());
        }
        self.big()
            .cmp(&other.big())
            .then_with(|| self.counts().cmp(&other.counts()))
    }
}

impl<const INLINE: bool, const DROP: bool> fmt::Debug for N<INLINE, DROP> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_heap() {
            let p = self.parts();
            write!(f, "N(heap {} limbs, 2^{}, ≈{:e})", p.limbs.len(), p.exp, self.to_f64())
        } else {
            write!(f, "N({:?})", self.as_exact())
        }
    }
}

const HEAP_TAG: u8 = 0x02;

impl<const INLINE: bool, const DROP: bool> Serialize for N<INLINE, DROP> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        if self.is_heap() {
            let p = self.parts();
            let mut v = Vec::with_capacity(1 + 4 + 1 + 4 + 12 + 8 * p.limbs.len());
            v.push(HEAP_TAG);
            v.extend_from_slice(&p.exp.to_le_bytes());
            v.push(u8::from(p.neg));
            v.extend_from_slice(&(p.limbs.len() as u32).to_le_bytes());
            for c in p.counts {
                v.extend_from_slice(&c.to_le_bytes());
            }
            for l in p.limbs {
                v.extend_from_slice(&l.to_le_bytes());
            }
            s.serialize_bytes(&v)
        } else {
            s.serialize_bytes(&self.as_exact().to_bytes())
        }
    }
}

struct NVisitor<const INLINE: bool, const DROP: bool>;
impl<'de, const INLINE: bool, const DROP: bool> Visitor<'de> for NVisitor<INLINE, DROP> {
    type Value = N<INLINE, DROP>;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("an Exact wire body or a heap body")
    }
    fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<N<INLINE, DROP>, E> {
        match v.first() {
            Some(&HEAP_TAG) => {
                let exp = i32::from_le_bytes(v[1..5].try_into().unwrap());
                let neg = v[5] != 0;
                let len = u32::from_le_bytes(v[6..10].try_into().unwrap()) as usize;
                let mut counts = [0i32; 3];
                for (i, c) in counts.iter_mut().enumerate() {
                    *c = i32::from_le_bytes(v[10 + 4 * i..14 + 4 * i].try_into().unwrap());
                }
                let limbs: Vec<u64> = v[22..]
                    .chunks_exact(8)
                    .map(|c| u64::from_le_bytes(c.try_into().unwrap()))
                    .collect();
                if limbs.len() != len || len < 3 {
                    return Err(de::Error::custom("heap body length"));
                }
                Ok(N::heap(neg, exp, counts, &limbs))
            }
            _ => Exact::from_bytes(v).map(N::from_exact).map_err(de::Error::custom),
        }
    }
}
impl<'de, const INLINE: bool, const DROP: bool> Deserialize<'de> for N<INLINE, DROP> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_bytes(NVisitor::<INLINE, DROP>)
    }
}

impl<const INLINE: bool, const DROP: bool> IsZero for N<INLINE, DROP> {
    /// Bitwise against the zero word: a heap word has a nonzero `lo`, so no tag test is needed.
    #[inline]
    fn is_zero(&self) -> bool {
        self.lo == 0
            && self.hi == 0
            && self.exp == 0
            && self.nan == 0
            && self.pos == 0
            && self.neg == 0
    }
}
impl<const INLINE: bool, const DROP: bool> Semigroup for N<INLINE, DROP> {
    #[inline]
    fn plus_equals(&mut self, rhs: &Self) {
        if !self.is_heap() && !rhs.is_heap() {
            match self.as_exact().checked_add(rhs.as_exact()) {
                Ok(s) => {
                    // SAFETY: self is compact, so there is nothing to drop before overwriting.
                    unsafe { ptr::write(self, Self::from_exact(s)) };
                    return;
                }
                Err(PastCap::Width) => {}
                Err(cap) => panic!("{cap}"),
            }
        }
        *self = self.add_slow(rhs);
    }
}
impl<const INLINE: bool, const DROP: bool> Monoid for N<INLINE, DROP> {
    #[inline]
    fn zero() -> Self {
        Self::ZERO
    }
}
impl<const INLINE: bool, const DROP: bool> Abelian for N<INLINE, DROP> {
    #[inline]
    fn negate(&mut self) {
        if self.is_heap() {
            if INLINE {
                self.lo ^= 2;
                self.nan = -self.nan;
                self.pos = -self.pos;
                self.neg = -self.neg;
            } else {
                let p = self.hi as *mut u64;
                // SAFETY: a heap word's block, header limbs 0..3.
                unsafe {
                    *p ^= 1 << 32;
                    let h1 = *p.add(1);
                    let (a, b) = (-(h1 as u32 as i32), -((h1 >> 32) as u32 as i32));
                    *p.add(1) = u64::from(a as u32) | (u64::from(b as u32) << 32);
                    let c = -(*p.add(2) as u32 as i32);
                    *p.add(2) = u64::from(c as u32);
                }
            }
        } else {
            // SAFETY: self is compact, so there is nothing to drop before overwriting.
            unsafe { ptr::write(self, Self::from_exact(-*self.as_exact())) };
        }
    }
}
impl<const INLINE: bool, const DROP: bool> Multiply<isize> for N<INLINE, DROP> {
    type Output = Self;
    #[inline]
    fn multiply(self, rhs: &isize) -> Self {
        let k = *rhs as i64;
        if !self.is_heap() {
            match self.as_exact().checked_mul_i64(k) {
                Ok(p) => {
                    // A compact self owns nothing, so skip its drop glue.
                    mem::forget(self);
                    return Self::from_exact(p);
                }
                Err(PastCap::Width) => {}
                Err(cap) => panic!("{cap}"),
            }
        }
        self.mul_slow(k)
    }
}

