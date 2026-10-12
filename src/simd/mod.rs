//! The bulk entries, DESIGN.md § Speed: every tier's `extend_from_slice` and `deposit_products`
//! come through here. A slice is folded in tiles; inside a tile, lanes fold strided subsets into
//! partial sums that merge at the tile's end, and every tier is associative and commutative, so the
//! merged bits are the bits the one-at-a-time deposits would land. The one-at-a-time loop is the
//! definition of correct, and the tests in tests/simd.rs hold every kernel to it.
//!
//! A kernel never decides a refusal. Each lane computes a sound "certainly inside the window" test,
//! and a vector with any lane that fails it is handed whole to the tier's scalar deposit, which is
//! exact on every input. So a kernel's checks may be conservative, and what it does accept it
//! places exactly: the same mantissa bits at the same limb offsets the scalar `set_shifted` writes.
//!
//! The portable kernel in scalar.rs is that algorithm in plain Rust over `[u64; N]` lanes, and it
//! runs on every target. Behind the `simd` feature, neon.rs and avx2.rs are the same algorithm on
//! 128- and 256-bit registers, chosen at runtime; the only `unsafe` in the crate is theirs.

use crate::binned::Binned;
use crate::fixed::{Fixed, OutOfWindow};
use crate::limbs;
use crate::wide::Wide;

mod scalar;

#[cfg(all(feature = "simd", target_arch = "x86_64"))]
#[allow(unsafe_code)] // the AVX2 intrinsics; every block states its invariant
mod avx2;
#[cfg(all(feature = "simd", target_arch = "aarch64"))]
#[allow(unsafe_code)] // the NEON intrinsics; every block states its invariant
mod neon;

/// Doubles per tile. The digit accumulators hold `2³²` adds, so any tile under that is exact; this
/// one is small enough that a `Wide` tile's window and a `Binned` tile's index follow the data.
pub(crate) const TILE: usize = 1024;

/// The window a `Wide` tile runs through: four limbs, the exact tiers' kernel width.
const WIDE_WINDOW: usize = 4;

/// Bits a magnitude may occupy inside a `LIMBS`-limb window: the width less a sign bit and 64 bits
/// of headroom, as `Fixed` and `Wide` define their ceilings.
pub(crate) const fn ceiling_bits(limbs: usize) -> i64 {
    64 * limbs as i64 - 65
}

/// A lane certainly places a double inside the window when its mantissa's bit 0 lands at a shift in
/// `[-63, max_shift(LIMBS)]` with nothing below the grid: 53 bits from `max_shift` end under the
/// ceiling, so a normal double's top bit is checked exactly and a subnormal's conservatively.
pub(crate) const fn max_shift(limbs: usize) -> i64 {
    ceiling_bits(limbs) - 53
}

/// What a tile's lanes accumulate, summed over every lane: for each limb the sums of the low and
/// high 32-bit halves of every placed word (each one's complement where the input was negative),
/// and the count of negative inputs. The tile's value follows from these alone, see [`Partials::twos_complement`]
/// and [`Partials::bin`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Partials<const LIMBS: usize> {
    pub(crate) lo: [u64; LIMBS],
    pub(crate) hi: [u64; LIMBS],
    pub(crate) negatives: u64,
}

impl<const LIMBS: usize> Partials<LIMBS> {
    pub(crate) const ZERO: Self = Self {
        lo: [0; LIMBS],
        hi: [0; LIMBS],
        negatives: 0,
    };

    /// Limb `i`'s accumulated words as one integer: the digit sums recombined.
    fn word_sum(&self, i: usize) -> u128 {
        u128::from(self.lo[i]) + (u128::from(self.hi[i]) << 32)
    }

    /// The tile's sum as a `LIMBS`-limb two's-complement integer. A negative input entered as the
    /// one's complement of its placed words, so adding the count of negatives at bit 0 completes
    /// each negation, and the wrap past the top is the modular wrap of two's complement itself.
    pub(crate) fn twos_complement(&self) -> [u64; LIMBS] {
        let mut sum = [0u64; LIMBS];
        for i in 0..LIMBS {
            limbs::add_u128_at(&mut sum, i, self.word_sum(i));
        }
        limbs::add_u128_at(&mut sum, 0, u128::from(self.negatives));
        sum
    }

    /// Bin `b`'s accumulator as `Binned` keeps it, `(limb, carry)`: every negative input's word
    /// entered as its one's complement, which is `2⁶⁴ − 1 − word`, so the bin's signed sum is the
    /// digit sum plus the negatives, less `2⁶⁴` per negative.
    pub(crate) fn bin(&self, b: usize) -> (u64, i64) {
        let n = i128::from(self.negatives);
        let v = self.word_sum(b) as i128 + n - (n << 64);
        (v as u64, (v >> 64) as i64)
    }
}

/// Folds one tile over a `LIMBS`-limb window whose bit 0 weighs `2^lsb`, adding every lane-placed
/// input to `partials` and handing every other input, in order, to `leftover`, which returns
/// `false` to stop the fold. `TRUNC` is the `Binned` mode: bits below the window are dropped rather
/// than refused, and no ceiling is checked because the window's top is the tile's own maximum.
/// Returns `false` iff `leftover` stopped it.
fn fold_tile<const LIMBS: usize, const TRUNC: bool>(
    xs: &[f64],
    lsb: i32,
    partials: &mut Partials<LIMBS>,
    leftover: &mut dyn FnMut(f64) -> bool,
) -> bool {
    #[cfg(all(feature = "simd", target_arch = "aarch64"))]
    if neon::handles::<LIMBS, TRUNC>() {
        return neon::fold_tile::<LIMBS, TRUNC>(xs, lsb, partials, leftover);
    }
    #[cfg(all(feature = "simd", target_arch = "x86_64"))]
    if avx2::handles::<LIMBS, TRUNC>() && avx2::available() {
        return avx2::fold_tile::<LIMBS, TRUNC>(xs, lsb, partials, leftover);
    }
    scalar::fold_tile::<4, LIMBS, TRUNC>(xs, lsb, partials, leftover)
}

/// The exponent of bit 0 of a four-limb window for a `Wide` tile: the lowest mantissa bit of its
/// finite nonzero inputs when the tile's span fits the window, else the window anchored at the
/// top so the largest inputs run through the lanes and the smallest fall to the scalar deposit.
fn wide_window(xs: &[f64]) -> i32 {
    let (min_exp, max_top) = span(xs);
    if max_top - min_exp < ceiling_bits(WIDE_WINDOW) as i32 {
        min_exp
    } else {
        max_top - (ceiling_bits(WIDE_WINDOW) as i32 - 1)
    }
}

/// The `Wide` limb of the largest finite input's top bit, or `K - 1` when nothing exceeds the
/// zero's window: the index a `Binned<K>` fold of the tile ends at.
fn binned_index<const K: usize>(xs: &[f64]) -> i16 {
    top_limb(xs).max(K as i16 - 1)
}

/// The tile's span by whichever kernel runs; a debug build holds the vector pass to the portable
/// one.
fn span(xs: &[f64]) -> (i32, i32) {
    #[cfg(all(feature = "simd", target_arch = "aarch64"))]
    {
        let v = neon::span(xs);
        debug_assert_eq!(v, scalar::span(xs));
        v
    }
    #[cfg(all(feature = "simd", target_arch = "x86_64"))]
    if avx2::available() {
        let v = avx2::span(xs);
        debug_assert_eq!(v, scalar::span(xs));
        return v;
    }
    #[cfg(not(all(feature = "simd", target_arch = "aarch64")))]
    scalar::span(xs)
}

/// The tile's top limb by whichever kernel runs, held to the portable pass in a debug build.
fn top_limb(xs: &[f64]) -> i16 {
    #[cfg(all(feature = "simd", target_arch = "aarch64"))]
    {
        let v = neon::top_limb(xs);
        debug_assert_eq!(v, scalar::top_limb(xs));
        v
    }
    #[cfg(all(feature = "simd", target_arch = "x86_64"))]
    if avx2::available() {
        let v = avx2::top_limb(xs);
        debug_assert_eq!(v, scalar::top_limb(xs));
        return v;
    }
    #[cfg(not(all(feature = "simd", target_arch = "aarch64")))]
    scalar::top_limb(xs)
}

/// `Wide::extend_from_slice`: each tile picks its window, folds, and lands in the limbs shifted
/// up to `Wide`'s grid; what the window does not hold is deposited one at a time.
pub(crate) fn wide_extend(w: &mut Wide, xs: &[f64]) {
    for tile in xs.chunks(TILE) {
        let lsb = wide_window(tile);
        let mut partials = Partials::<WIDE_WINDOW>::ZERO;
        fold_tile::<WIDE_WINDOW, false>(tile, lsb, &mut partials, &mut |x| {
            w.deposit(x);
            true
        });
        let sum = partials.twos_complement();
        if sum != [0; WIDE_WINDOW] {
            let mut term = [0u64; Wide::LIMBS];
            // The tile's sum is under 2^(201 + lsb - LSB) units, far inside the width.
            limbs::shl_extended(&sum, &mut term, (lsb - Wide::LSB) as u32);
            *w += Wide::from_parts(term, [0; 3]);
        }
    }
}

/// `Fixed::extend_from_slice`: the window is the type's own, the tiles accumulate into a local
/// state, and the first refusal in slice order ends the fold with the caller's state untouched.
pub(crate) fn fixed_extend<const LSB: i32, const LIMBS: usize>(
    f: &mut Fixed<LSB, LIMBS>,
    xs: &[f64],
) -> Result<(), OutOfWindow> {
    let mut total = Fixed::<LSB, LIMBS>::ZERO;
    let mut refused = None;
    for tile in xs.chunks(TILE) {
        let mut partials = Partials::<LIMBS>::ZERO;
        let went_on = fold_tile::<LIMBS, false>(tile, LSB, &mut partials, &mut |x| match total
            .try_deposit(x)
        {
            Ok(()) => true,
            Err(e) => {
                refused = Some(e);
                false
            }
        });
        if let Some(e) = refused {
            return Err(e);
        }
        debug_assert!(went_on);
        total += Fixed::from_limbs(partials.twos_complement());
    }
    *f += total;
    Ok(())
}

/// `Binned::extend_from_slice`: each tile's index is its largest input's, the lanes truncate
/// below that window as the fold would, the tile's bins come out of the partials, and the tiles
/// merge into the state through `Add`, which aligns the windows.
pub(crate) fn binned_extend<const K: usize>(b: &mut Binned<K>, xs: &[f64]) {
    for tile in xs.chunks(TILE) {
        let index = binned_index::<K>(tile);
        let lsb = Wide::LSB + 64 * (i32::from(index) - K as i32 + 1);
        let mut partials = Partials::<K>::ZERO;
        fold_tile::<K, true>(tile, lsb, &mut partials, &mut |x| {
            b.deposit(x);
            true
        });
        let mut bins = [0u64; K];
        let mut carries = [0i64; K];
        for (i, (limb, carry)) in bins.iter_mut().zip(carries.iter_mut()).enumerate() {
            (*limb, *carry) = partials.bin(i);
        }
        *b += Binned::from_bins(bins, carries, [0; 3], index);
    }
}
