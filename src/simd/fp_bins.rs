//! `Binned` kernel (ii), DESIGN.md § Speed: `ReproBLAS`'s pinned-exponent floating-point fold, built
//! as the bench-only experiment the design calls for beside kernel (i), the integer fold in
//! scalar.rs. Nothing here is reachable from the public surface but two hidden entries on `Binned`
//! that tests/simd.rs and benches/simd.rs use.
//!
//! The shape that can land the fold's bits: a tile's window is the one kernel (i) picks, and each
//! input's magnitude is peeled from the top down through `2K` half-bins of 32 bits on `Wide`'s
//! half-limb grid. At each level the piece is `q = (M + m) − M` with `M = 1.5 · 2^(e + 52)`,
//! which rounds `m` to the nearest multiple of `2^e` exactly (the sum's unit is `2^e` and the
//! subtraction is Sterbenz-exact), then one fix-up turns nearest into the fold's truncation toward
//! zero: `q > m` means `q −= 2^e`. The piece takes the input's sign by one xor and goes into a
//! per-lane bin; the remainder `m −= q` carries on. A piece is a multiple of `2^e` under `2³²` of
//! them, so a bin stays exact for `2²¹` adds, far past a tile. At the tile's end each half-bin is
//! read as an integer through its bits, with no floating-point instruction, and the two halves of
//! a limb combine as `Partials::bin` does. Bins must align with the fold's 64-bit limbs because
//! the fold's bins are independent: a half-bin's sum is one limb's digit sum and never carries
//! into the next, exactly as the one-at-a-time deposits never do.
//!
//! The nearest-rounding variant without the fix-up, `ReproBLAS`'s actual kernel, is here too, bench
//! only: it cannot land the fold's bits, and it bounds what the floating-point shape could cost
//! if truncation were free.
//!
//! The constants stay finite only while the top half-bin's `M` does, which is an index of 31 at
//! most; a tile holding anything at or above `2^974` goes whole to the scalar deposit.

use super::TILE;

/// Lanes per step: four doubles, so the autovectoriser has two NEON or one AVX2 register's worth
/// of independent work per level.
pub(crate) const N: usize = 4;

/// The highest `Wide` limb a tile's index may reach: the top half-bin's rounding constant is
/// `1.5 · 2^(lsb + 64K − 32 + 52)`, finite while `lsb + 64K ≤ 1003`, which with `lsb = −1074 +
/// 64(index − K + 1)` is `index ≤ 31`.
pub(crate) const MAX_INDEX: i16 = 31;

const FRAC_BITS: u32 = 52;
const FRAC_MASK: u64 = (1 << FRAC_BITS) - 1;
const EXP_MASK: u64 = 0x7FF;
const SIGN: u64 = 1 << 63;

/// `2^e` by its bits, subnormals included; `e` runs from `−1074` to `1023`.
fn pow2(e: i32) -> f64 {
    debug_assert!((-1074..=1023).contains(&e));
    if e >= -1022 {
        f64::from_bits(((e + 1023) as u64) << FRAC_BITS)
    } else {
        f64::from_bits(1u64 << (e + 1074))
    }
}

/// The rounding constant for unit `2^e`: `1.5 · 2^(e + 52)`, so that `M + m` for `0 ≤ m <
/// 2^(e + 51)` lies in `[M, 2M)`, where the unit in the last place is `2^e`.
fn rounder(e: i32) -> f64 {
    debug_assert!((-1074..=971).contains(&e));
    f64::from_bits((((e + 52 + 1023) as u64) << FRAC_BITS) | (1 << (FRAC_BITS - 1)))
}

/// Per-lane half-bin sums for a `K`-bin tile: `[b][h]` is limb `b`'s low (`h = 0`) or high
/// (`h = 1`) 32 bits, each lane's sum in units of `2^(lsb + 32(2b + h))`.
pub(crate) type Sums<const K: usize> = [[[f64; N]; 2]; K];

/// The peel of one tile: see the module doc. `FIXUP` is the truncating, bit-identical kernel;
/// `false` is the nearest-rounding variant. A vector with a special goes whole to `leftover`.
#[allow(clippy::inline_always)] // inlined into the AVX2 wrapper so the vectoriser sees the loop
#[inline(always)]
fn peel<const K: usize, const FIXUP: bool>(
    xs: &[f64],
    lsb: i32,
    sums: &mut Sums<K>,
    leftover: &mut dyn FnMut(f64),
) {
    debug_assert!(xs.len() <= TILE);
    let mut units = [[0.0f64; 2]; K];
    let mut rounders = [[0.0f64; 2]; K];
    for (b, (u, r)) in units.iter_mut().zip(rounders.iter_mut()).enumerate() {
        for h in 0..2 {
            let e = lsb + 32 * (2 * b + h) as i32;
            u[h] = pow2(e);
            r[h] = rounder(e);
        }
    }
    let mut chunks = xs.chunks_exact(N);
    for chunk in &mut chunks {
        let mut mag = [0.0f64; N];
        let mut sign = [0u64; N];
        let mut finite = true;
        for ((m, s), &x) in mag.iter_mut().zip(sign.iter_mut()).zip(chunk) {
            *m = x.abs();
            *s = x.to_bits() & SIGN;
            finite &= x.is_finite();
        }
        if !finite {
            for &x in chunk {
                leftover(x);
            }
            continue;
        }
        for b in (0..K).rev() {
            for h in (0..2).rev() {
                let big = rounders[b][h];
                let unit = units[b][h];
                for ((bin, m), &s) in sums[b][h].iter_mut().zip(mag.iter_mut()).zip(&sign) {
                    let nearest = (big + *m) - big;
                    let q = if FIXUP && nearest > *m {
                        nearest - unit
                    } else {
                        nearest
                    };
                    *bin += f64::from_bits(q.to_bits() ^ s);
                    *m -= q;
                }
            }
        }
    }
    for &x in chunks.remainder() {
        leftover(x);
    }
}

/// The peel compiled for AVX2, so the autovectoriser has 256-bit registers; without it an x86_64
/// build vectorises for the baseline SSE2 and works two doubles a step.
#[cfg(all(feature = "simd", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
unsafe fn peel_avx2<const K: usize, const FIXUP: bool>(
    xs: &[f64],
    lsb: i32,
    sums: &mut Sums<K>,
    leftover: &mut dyn FnMut(f64),
) {
    peel::<K, FIXUP>(xs, lsb, sums, leftover);
}

/// The fold of one tile into `sums`, by the widest vectors this machine has.
pub(crate) fn fold_tile<const K: usize, const FIXUP: bool>(
    xs: &[f64],
    lsb: i32,
    sums: &mut Sums<K>,
    leftover: &mut dyn FnMut(f64),
) {
    #[cfg(all(feature = "simd", target_arch = "x86_64"))]
    if super::avx2::available() {
        // SAFETY: the only precondition of a function compiled for AVX2 is that the CPU has it,
        // which the runtime detection just established.
        return unsafe { peel_avx2::<K, FIXUP>(xs, lsb, sums, leftover) };
    }
    peel::<K, FIXUP>(xs, lsb, sums, leftover);
}

/// The integer a half-bin's sum stands for, in units of `2^e`: the sum is a multiple of `2^e`
/// under `2⁵³` of them, so its mantissa shifted down to the unit is exact, and the shift is
/// never up. Read from the bits alone.
fn integer(sum: f64, e: i32) -> i64 {
    let bits = sum.to_bits();
    let biased = (bits >> FRAC_BITS) & EXP_MASK;
    let frac = bits & FRAC_MASK;
    let (mant, exp) = if biased == 0 {
        (frac, 1 - 1075)
    } else {
        (frac | (1 << FRAC_BITS), biased as i32 - 1075)
    };
    if mant == 0 {
        return 0;
    }
    let down = e - exp;
    debug_assert!((0..53).contains(&down), "a half-bin's sum left its unit");
    debug_assert_eq!(
        mant & ((1 << down) - 1),
        0,
        "a half-bin's sum left its unit"
    );
    let mag = (mant >> down) as i64;
    if bits & SIGN != 0 {
        -mag
    } else {
        mag
    }
}

/// The tile's bins as `Binned` keeps them, `(limb, carry)` per bin: the lanes' half-bin integers
/// summed, the high half shifted up 32, and the signed total split at `2⁶⁴`.
pub(crate) fn bins<const K: usize>(sums: &Sums<K>, lsb: i32) -> ([u64; K], [i64; K]) {
    let mut limbs = [0u64; K];
    let mut carries = [0i64; K];
    for (b, (limb, carry)) in limbs.iter_mut().zip(carries.iter_mut()).enumerate() {
        let mut v = 0i128;
        for (h, half) in sums[b].iter().enumerate() {
            let e = lsb + 32 * (2 * b + h) as i32;
            let digit: i64 = half.iter().map(|&s| integer(s, e)).sum();
            v += i128::from(digit) << (32 * h);
        }
        *limb = v as u64;
        *carry = (v >> 64) as i64;
    }
    (limbs, carries)
}
