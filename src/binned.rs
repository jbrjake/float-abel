//! [`Binned`]: the batch-only tier. `K` bins on [`Wide`]'s limb grid, each an independent signed
//! accumulator, a window that rises to the largest input, and three special counters. A fold is a
//! function of the multiset alone, so any order, partition or merge tree lands the same bits; what
//! it is not is a group, and the module implements nothing that resembles an inverse.
//!
//! The bins are independent on purpose. Demmel and Nguyen's accumulator truncates every input at
//! the same boundaries, past and future, which is what makes the fold order-independent once the
//! window has moved: the bins that survive a rise hold exactly what the inputs would have left in
//! them had the window been there from the start, and the dropped bin holds exactly what they
//! would have lost. That stops being true the moment a bin's overflow crosses into the bin above.
//! A carry that crossed before a rise survives the drop of the bin it came from, while the same
//! inputs deposited after the rise lose it, so the sum would depend on the order. Each bin
//! therefore keeps its own overflow, carry-save, as a `u64` limb and an `i64` carry, and nothing
//! ever crosses between bins until [`Binned::into_wide`] adds them all up at the end.

use core::fmt;
use core::iter::Sum;
use core::ops::{Add, AddAssign};

use crate::fixed::Hex;
use crate::ieee::{self, NAN, NEG_INF, POS_INF};
use crate::limbs;
use crate::wide::Wide;

/// The batch-only adaptive tier: `K` bins on [`Wide`]'s limb grid, an index and the special
/// counters. Constant size and the full `f64` range, at the price of the group laws: a fold is
/// order-independent, but there is no inverse, and the result is a function of the largest input
/// seen, not only of the current multiset.
///
/// Bin `b` is the independent signed accumulator `carries[b] · 2⁶⁴ + limbs[b]`, holding the sum
/// over the multiset of every input's slice at `Wide` limb `index − K + 1 + b`, signed as the
/// input was. A deposit takes the input's magnitude apart at `Wide`'s limb boundaries, adds each
/// slice to its bin, and drops what falls below the lowest bin, so it truncates toward zero. An
/// input whose top limb lies above `index` raises the index and drops the lowest bins. Specials
/// bump their counters and never move the window. The zero has its window at the bottom of the
/// grid, so a fold of subnormals is exact.
///
/// [`Add`] aligns both sides to the higher index and adds bin-wise: associative and commutative,
/// so a fold is bit-identical under any order and across any partition. There is no `Neg`, no
/// `Sub`, no scalar multiply and, behind the `differential` and `dbsp` features, none of the
/// engine weight traits: a compile-fail test pins that. Structural `Eq`, `Ord` and `Hash` are
/// the bits, not the value, because the zero is not unique: a fold that cancelled keeps the
/// index its largest input set.
///
/// The error: `|S − Σxⱼ| ≤ n · 2^(−64(K−1)) · max|xⱼ|`, plus one rounding on the read. The
/// lowest kept bit lies at most `64(K−1)` below the top bit of the largest input, and each input
/// loses less than one unit of it. [`Binned::into_wide`] is exact, and the read goes through
/// `Wide`. There is no wire form: a batch result crosses a boundary as the `Wide` it converts to.
///
/// A plain word: `Copy`, `repr(C)`, `16·K + 32` bytes (the `i16` index is padded to a word) with
/// alignment 8, pinned by a `const` assertion, and nothing allocates. The carries are the one
/// budget: a bin wraps at most once per deposit, so `2⁶³` deposits exhaust it, and past it the
/// operators panic by name in every build.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct Binned<const K: usize> {
    limbs: [u64; K],
    carries: [i64; K],
    /// `[n, p, q]`: NaN, +∞ and −∞ counts.
    specials: [i64; 3],
    /// The `Wide` limb number of the top bin: bin `K − 1` is `Wide` limb `index`.
    index: i16,
}

const _: () = assert!(
    core::mem::size_of::<Binned<3>>() == 80 && core::mem::align_of::<Binned<3>>() == 8,
    "Binned's layout moved"
);

impl<const K: usize> Binned<K> {
    /// At least one bin, and at most 33, so that the top bin's carry lies inside `Wide`'s 34
    /// limbs. Evaluated when the zero is instantiated, so `Binned<0>` does not build.
    const BINS_OK: () = assert!(
        K >= 1 && K < Wide::LIMBS,
        "Binned: K must lie in 1..=33, so the top bin's carry still fits Wide's 34 limbs"
    );

    /// The zero: empty bins with the window at the bottom of the grid, and no specials.
    pub const ZERO: Self = {
        #[allow(clippy::let_unit_value)] // evaluates the const assertion on K
        let () = Self::BINS_OK;
        Self {
            limbs: [0; K],
            carries: [0; K],
            specials: [0; 3],
            index: K as i16 - 1,
        }
    };

    /// The `Wide` limb number of the top bin. Starts at `K − 1`, rises to the top limb of the
    /// largest magnitude deposited, and never comes back down.
    #[must_use]
    pub const fn index(&self) -> i16 {
        self.index
    }

    /// The low 64 bits of each bin's accumulator, lowest bin first.
    #[must_use]
    pub const fn limbs(&self) -> &[u64; K] {
        &self.limbs
    }

    /// Each bin's carry: the accumulator's value above its limb, signed, so that bin `b` holds
    /// `carries[b] · 2⁶⁴ + limbs[b]`.
    #[must_use]
    pub const fn carries(&self) -> &[i64; K] {
        &self.carries
    }

    /// The special counts `(n, p, q)`: how many NaN, +∞ and −∞ inputs are in the multiset.
    #[must_use]
    pub const fn specials(&self) -> (i64, i64, i64) {
        (
            self.specials[NAN],
            self.specials[POS_INF],
            self.specials[NEG_INF],
        )
    }

    /// Whether the value is zero: every bin empty and no specials. The index is not a value, so
    /// a fold that cancelled is zero here though it is not [`Binned::ZERO`].
    #[must_use]
    pub fn is_zero(&self) -> bool {
        limbs::is_zero(&self.limbs)
            && self.carries.iter().all(|&c| c == 0)
            && self.specials == [0; 3]
    }

    /// Whether every counter is zero, so a read rounds the finite part.
    #[must_use]
    pub fn is_finite(&self) -> bool {
        self.specials == [0; 3]
    }

    /// Adds a double. A finite value lands in the bins its limbs fall in, less whatever lies
    /// below the window; either zero adds nothing; NaN, +∞ and −∞ each bump their counter.
    pub fn deposit(&mut self, x: f64) {
        match ieee::decompose(x) {
            Some(d) if d.mant == 0 => {}
            // A finite double's lowest bit is at or above the unit, so the shift is never negative.
            Some(d) => self.deposit_finite(d.negative, d.mant, (d.exp - Wide::LSB) as u32),
            None => self.bump(ieee::counter(ieee::special(x)), 1),
        }
    }

    /// Adds every double in the slice, as [`Binned::deposit`] would one at a time.
    pub fn extend_from_slice(&mut self, xs: &[f64]) {
        for &x in xs {
            self.deposit(x);
        }
    }

    /// Adds `(-1)^negative · mant · 2^(shift + Wide::LSB)`, `mant` odd and below `2⁵³`.
    fn deposit_finite(&mut self, negative: bool, mant: u64, shift: u32) {
        let top = (shift + 63 - mant.leading_zeros()) / 64;
        if top as i16 > self.index {
            self.raise(top as i16);
        }
        // The magnitude occupies Wide limb `q`, and `q + 1` when its 53 bits cross the boundary.
        let q = shift / 64;
        let off = shift % 64;
        let (low, high) = if off == 0 {
            (mant, 0)
        } else {
            (mant << off, mant >> (64 - off))
        };
        let bottom = i32::from(self.index) - K as i32 + 1;
        for (limb, word) in [(q as i32, low), (q as i32 + 1, high)] {
            // Below the window is dropped; above it cannot happen, since `top ≤ index`.
            if word != 0 && limb >= bottom {
                self.add_to_bin((limb - bottom) as usize, word, negative);
            }
        }
    }

    /// `bin b += ±word`, carry-save: the limb wraps and the carry counts the wrap.
    fn add_to_bin(&mut self, b: usize, word: u64, negative: bool) {
        let (limb, wrapped) = if negative {
            self.limbs[b].overflowing_sub(word)
        } else {
            self.limbs[b].overflowing_add(word)
        };
        self.limbs[b] = limb;
        if wrapped {
            let step = if negative { -1 } else { 1 };
            self.carries[b] = self.carries[b]
                .checked_add(step)
                .expect("Binned: a bin's carry overflowed, past 2^63 deposits into one bin");
        }
    }

    /// Moves the window up so that its top bin is `Wide` limb `top`: the lowest bins are dropped,
    /// the rest slide down, and the new top bins start empty.
    fn raise(&mut self, top: i16) {
        let by = (top - self.index) as usize;
        if by >= K {
            self.limbs = [0; K];
            self.carries = [0; K];
        } else {
            self.limbs.copy_within(by.., 0);
            self.limbs[K - by..].fill(0);
            self.carries.copy_within(by.., 0);
            self.carries[K - by..].fill(0);
        }
        self.index = top;
    }

    fn bump(&mut self, which: usize, by: i64) {
        self.specials[which] = self.specials[which]
            .checked_add(by)
            .expect("Binned: a special counter overflowed");
    }

    /// The exact value as a [`Wide`]: every bin's accumulator placed at its limb and the counters
    /// carried across. Nothing is lost here; what the fold dropped is already gone.
    ///
    /// # Panics
    ///
    /// If the value does not fit `Wide`'s width. A fold of doubles never reaches that: its value
    /// is at most the count times the largest magnitude, inside `Wide`'s budget of `2⁷⁷` deposits
    /// of `f64::MAX`, and the carries cap the count at `2⁶³`.
    #[must_use]
    pub fn into_wide(self) -> Wide {
        // One limb more than Wide, so that the top bin's carry has a slot and the fit is checked
        // once at the end rather than assumed. The value of a fold of doubles always fits: it is
        // at most the count times the largest magnitude, inside Wide's 2^77 budget.
        let mut acc = [0u64; Wide::LIMBS + 1];
        let mut term = [0u64; Wide::LIMBS + 1];
        let bottom = self.index as usize + 1 - K;
        for b in 0..K {
            let at = bottom + b;
            let fill = if self.carries[b] < 0 { u64::MAX } else { 0 };
            term.fill(0);
            term[at] = self.limbs[b];
            term[at + 1] = self.carries[b] as u64;
            term[at + 2..].fill(fill);
            let overflow = limbs::add_assign(&mut acc, &term);
            assert!(!overflow, "Binned: the value does not fit Wide's width");
        }
        let sign = if limbs::is_negative(&acc[..Wide::LIMBS]) {
            u64::MAX
        } else {
            0
        };
        assert_eq!(
            acc[Wide::LIMBS],
            sign,
            "Binned: the value does not fit Wide's width"
        );
        let mut wide = [0u64; Wide::LIMBS];
        wide.copy_from_slice(&acc[..Wide::LIMBS]);
        Wide::from_parts(wide, self.specials)
    }
}

impl Wide {
    /// The top `K` limbs of the magnitude as a [`Binned`], with the sign and the counters. Lossy:
    /// every limb of the magnitude below the window is dropped, toward zero. A single double
    /// spans at most two limbs, so `Wide::from(x).to_binned::<2>()` holds it whole.
    #[must_use]
    pub fn to_binned<const K: usize>(&self) -> Binned<K> {
        let mut mag = [0u64; Wide::LIMBS];
        let negative = limbs::abs_into(self.limbs(), &mut mag);
        let top = mag.iter().rposition(|&l| l != 0).unwrap_or(0).max(K - 1);
        let mut b = Binned::<K>::ZERO;
        b.index = top as i16;
        for (bin, &word) in mag[top + 1 - K..=top].iter().enumerate() {
            if word != 0 {
                b.add_to_bin(bin, word, negative);
            }
        }
        let (n, p, q) = self.specials();
        b.specials = [n, p, q];
        b
    }
}

impl<const K: usize> Default for Binned<K> {
    fn default() -> Self {
        Self::ZERO
    }
}

impl<const K: usize> fmt::Debug for Binned<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let wide = self.into_wide();
        let about = if wide.try_to_f64().is_some() {
            ""
        } else {
            "≈"
        };
        let [n, p, q] = self.specials;
        write!(
            f,
            "Binned<{K}>({about}{:e}; index {}; bins ",
            wide.to_f64(),
            self.index
        )?;
        f.debug_list()
            .entries(
                self.carries
                    .iter()
                    .zip(&self.limbs)
                    .map(|(&c, &l)| (c, Hex(l))),
            )
            .finish()?;
        write!(f, "; nan {n}, +inf {p}, -inf {q})")
    }
}

impl<const K: usize> From<f64> for Binned<K> {
    /// A fresh fold holding `x`: its value, or one count of its special.
    fn from(x: f64) -> Self {
        let mut b = Self::ZERO;
        b.deposit(x);
        b
    }
}

impl<const K: usize> AddAssign for Binned<K> {
    /// Aligns both sides to the higher index, dropping the other side's lowest bins, then adds
    /// bin-wise and counter-wise.
    fn add_assign(&mut self, mut rhs: Self) {
        if rhs.index > self.index {
            self.raise(rhs.index);
        } else if self.index > rhs.index {
            rhs.raise(self.index);
        }
        for b in 0..K {
            let (limb, wrapped) = self.limbs[b].overflowing_add(rhs.limbs[b]);
            self.limbs[b] = limb;
            self.carries[b] = self.carries[b]
                .checked_add(rhs.carries[b])
                .and_then(|c| c.checked_add(i64::from(wrapped)))
                .expect("Binned: a bin's carry overflowed in an add");
        }
        for (s, &r) in self.specials.iter_mut().zip(&rhs.specials) {
            *s = s
                .checked_add(r)
                .expect("Binned: a special counter overflowed in an add");
        }
    }
}

impl<const K: usize> Add for Binned<K> {
    type Output = Self;
    fn add(mut self, rhs: Self) -> Self {
        self += rhs;
        self
    }
}

impl<const K: usize> Sum for Binned<K> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, Add::add)
    }
}

impl<'a, const K: usize> Sum<&'a Self> for Binned<K> {
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, |acc, x| acc + *x)
    }
}
