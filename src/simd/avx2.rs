//! The AVX2 kernel: four doubles per 256-bit step, the portable kernel in scalar.rs op for op,
//! chosen at runtime by `is_x86_feature_detected!("avx2")`. The variable shifts `vpsllvq` and
//! `vpsrlvq` take an unsigned count per lane and give zero for any count of 64 or more, so a
//! negative count is a zero too: one left and one right shift place a mantissa at any limb or drop
//! it with no clamping. AVX2 has no 64-bit arithmetic shift and no unsigned 64-bit compare, so the
//! sign mask is a signed compare against zero, and the running minimum and maximum of the span pass
//! are signed compares and blends over values that are small and non-negative.

// The intrinsics are `unsafe fn` on the MSRV and safe to call inside an AVX2 `target_feature`
// function since Rust 1.87, so the blocks below are required on the one and unused on the other.
#![allow(unused_unsafe)]
// Every load and store here is the unaligned `loadu`/`storeu`, which takes a pointer of any
// alignment; the lint reads the `__m256i` pointer type as a 32-byte promise it is not.
#![allow(clippy::cast_ptr_alignment)]

use core::arch::x86_64::{
    __m256i, _mm256_add_epi64, _mm256_and_si256, _mm256_andnot_si256, _mm256_blendv_epi8,
    _mm256_castsi256_pd, _mm256_cmpeq_epi64, _mm256_cmpgt_epi64, _mm256_loadu_si256,
    _mm256_movemask_pd, _mm256_or_si256, _mm256_set1_epi64x, _mm256_setzero_si256,
    _mm256_slli_epi64, _mm256_sllv_epi64, _mm256_srli_epi64, _mm256_srlv_epi64,
    _mm256_storeu_si256, _mm256_sub_epi64, _mm256_xor_si256,
};

use super::{max_shift, Partials};

const FRAC_MASK: i64 = (1 << 52) - 1;
const EXP_MASK: i64 = 0x7FF;
const NORMAL_BIAS: i64 = 1075;
const LO32: i64 = 0xFFFF_FFFF;

/// Whether this kernel covers the window: four limbs, the register budget (two accumulators a
/// limb), for the exact and the truncating tiers alike.
pub(crate) const fn handles<const LIMBS: usize, const TRUNC: bool>() -> bool {
    LIMBS <= 4
}

/// Whether this machine has AVX2: the standard library's detection, cached behind one atomic
/// load, so asking per tile costs nothing.
pub(crate) fn available() -> bool {
    std::is_x86_feature_detected!("avx2")
}

/// The AVX2 fold of one tile; see `super::fold_tile` for the contract.
///
/// # Panics
///
/// If this machine has no AVX2: the dispatcher asks [`available`] first.
pub(crate) fn fold_tile<const LIMBS: usize, const TRUNC: bool>(
    xs: &[f64],
    lsb: i32,
    partials: &mut Partials<LIMBS>,
    leftover: &mut dyn FnMut(f64) -> bool,
) -> bool {
    assert!(available(), "the AVX2 kernel on a machine without AVX2");
    debug_assert!(handles::<LIMBS, TRUNC>());
    // SAFETY: AVX2 was just detected on this machine.
    unsafe { fold_avx2::<LIMBS, TRUNC>(xs, lsb, partials, leftover) }
}

/// The span of a tile's finite nonzero inputs, as `super::scalar::span` defines it.
///
/// # Panics
///
/// If this machine has no AVX2.
pub(crate) fn span(xs: &[f64]) -> (i32, i32) {
    assert!(available(), "the AVX2 kernel on a machine without AVX2");
    // SAFETY: AVX2 was just detected on this machine.
    unsafe { span_avx2(xs) }
}

/// The `Wide` limb of the highest top bit among a tile's finite inputs, as
/// `super::scalar::top_limb` defines it.
///
/// # Panics
///
/// If this machine has no AVX2.
pub(crate) fn top_limb(xs: &[f64]) -> i16 {
    assert!(available(), "the AVX2 kernel on a machine without AVX2");
    // SAFETY: AVX2 was just detected on this machine.
    unsafe { top_limb_avx2(xs) }
}

/// # Safety
///
/// The caller has seen [`available`] return `true` on this machine.
#[target_feature(enable = "avx2")]
#[allow(clippy::too_many_lines)] // one straight pass; splitting it would hide the register budget
unsafe fn fold_avx2<const LIMBS: usize, const TRUNC: bool>(
    xs: &[f64],
    lsb: i32,
    partials: &mut Partials<LIMBS>,
    leftover: &mut dyn FnMut(f64) -> bool,
) -> bool {
    // SAFETY: the caller established AVX2; the constants touch no memory.
    let (frac_mask, exp_mask, implicit, sixty_four, shift_base, max_ok, lo32, zero, ones) = unsafe {
        (
            _mm256_set1_epi64x(FRAC_MASK),
            _mm256_set1_epi64x(EXP_MASK),
            _mm256_set1_epi64x(1 << 52),
            _mm256_set1_epi64x(64),
            _mm256_set1_epi64x(-NORMAL_BIAS - i64::from(lsb)),
            _mm256_set1_epi64x(max_shift(LIMBS)),
            _mm256_set1_epi64x(LO32),
            _mm256_setzero_si256(),
            _mm256_set1_epi64x(-1),
        )
    };
    // Per limb, the digit sums over the four lanes: the low and the high 32-bit halves.
    let mut acc_lo = [zero; LIMBS];
    let mut acc_hi = [zero; LIMBS];
    let mut negatives = zero;

    let mut chunks = xs.chunks_exact(4);
    for chunk in &mut chunks {
        // SAFETY: the caller established AVX2; the unaligned load reads the four doubles of
        // `chunk`, which `chunks_exact(4)` guarantees are there, as the four lanes of one register.
        let suspect_any = unsafe {
            let bits = _mm256_loadu_si256(chunk.as_ptr().cast::<__m256i>());
            let neg = _mm256_cmpgt_epi64(zero, bits);
            let biased = _mm256_and_si256(_mm256_srli_epi64::<52>(bits), exp_mask);
            let frac = _mm256_and_si256(bits, frac_mask);
            let subnormal = _mm256_cmpeq_epi64(biased, zero);
            let mant = _mm256_or_si256(frac, _mm256_andnot_si256(subnormal, implicit));
            // exp + 1075 is max(biased, 1): the subnormal mask is -1, so subtracting it adds one.
            let shift = _mm256_add_epi64(_mm256_sub_epi64(biased, subnormal), shift_base);
            let mut suspect = _mm256_cmpeq_epi64(biased, exp_mask);
            if !TRUNC {
                // Bits below the grid: the mantissa shifted left by 64 + shift, for shifts in
                // [-63, -1]; a shift at or below -64 drops every bit, so the count clamps to zero
                // and the test is the mantissa itself; a shift at or above zero shifts by 64 or
                // more and tests nothing.
                let k = _mm256_add_epi64(shift, sixty_four);
                let k = _mm256_andnot_si256(_mm256_cmpgt_epi64(zero, k), k);
                let low = _mm256_sllv_epi64(mant, k);
                let off_grid = _mm256_xor_si256(_mm256_cmpeq_epi64(low, zero), ones);
                let over = _mm256_cmpgt_epi64(shift, max_ok);
                suspect = _mm256_or_si256(suspect, _mm256_or_si256(off_grid, over));
            }
            if _mm256_movemask_pd(_mm256_castsi256_pd(suspect)) != 0 {
                true
            } else {
                // Limb i takes mant << (shift - 64i) or mant >> (64i - shift): whichever count is
                // in range, the other being 64 or more and giving zero.
                let mut left = shift;
                let mut right = _mm256_sub_epi64(zero, shift);
                for (lo, hi) in acc_lo.iter_mut().zip(acc_hi.iter_mut()) {
                    let placed = _mm256_or_si256(
                        _mm256_sllv_epi64(mant, left),
                        _mm256_srlv_epi64(mant, right),
                    );
                    let w = _mm256_xor_si256(placed, neg);
                    *lo = _mm256_add_epi64(*lo, _mm256_and_si256(w, lo32));
                    *hi = _mm256_add_epi64(*hi, _mm256_srli_epi64::<32>(w));
                    left = _mm256_sub_epi64(left, sixty_four);
                    right = _mm256_add_epi64(right, sixty_four);
                }
                negatives = _mm256_sub_epi64(negatives, neg);
                false
            }
        };
        if suspect_any {
            for &x in chunk {
                if !leftover(x) {
                    return false;
                }
            }
        }
    }
    for &x in chunks.remainder() {
        if !leftover(x) {
            return false;
        }
    }
    // SAFETY: the caller established AVX2; the reductions read registers only.
    unsafe {
        for i in 0..LIMBS {
            partials.lo[i] += lanes(acc_lo[i]).iter().sum::<u64>();
            partials.hi[i] += lanes(acc_hi[i]).iter().sum::<u64>();
        }
        partials.negatives += lanes(negatives).iter().sum::<u64>();
    }
    true
}

/// The four lanes of a register, through memory: the reduction's one store per accumulator.
///
/// # Safety
///
/// The caller has seen [`available`] return `true` on this machine.
#[target_feature(enable = "avx2")]
unsafe fn lanes(v: __m256i) -> [u64; 4] {
    let mut out = [0u64; 4];
    // SAFETY: the caller established AVX2; the store writes 32 bytes into a 32-byte array.
    unsafe { _mm256_storeu_si256(out.as_mut_ptr().cast::<__m256i>(), v) };
    out
}

/// # Safety
///
/// The caller has seen [`available`] return `true` on this machine.
#[target_feature(enable = "avx2")]
unsafe fn span_avx2(xs: &[f64]) -> (i32, i32) {
    const NONE_LOW: i64 = 1 << 40;
    const NONE_HIGH: i64 = -(1 << 40);
    // SAFETY: the caller established AVX2; the one load per step reads the four doubles of a
    // `chunks_exact(4)` chunk.
    let (mut min_e, mut max_t) = unsafe {
        let exp_mask = _mm256_set1_epi64x(EXP_MASK);
        let none_low = _mm256_set1_epi64x(NONE_LOW);
        let none_high = _mm256_set1_epi64x(NONE_HIGH);
        let zero = _mm256_setzero_si256();
        let mut min_e = none_low;
        let mut max_t = none_high;
        let mut chunks = xs.chunks_exact(4);
        for chunk in &mut chunks {
            let bits = _mm256_loadu_si256(chunk.as_ptr().cast::<__m256i>());
            let biased = _mm256_and_si256(_mm256_srli_epi64::<52>(bits), exp_mask);
            let skip = _mm256_or_si256(
                _mm256_cmpeq_epi64(biased, exp_mask),
                _mm256_cmpeq_epi64(_mm256_slli_epi64::<1>(bits), zero),
            );
            let subnormal = _mm256_cmpeq_epi64(biased, zero);
            let e = _mm256_blendv_epi8(_mm256_sub_epi64(biased, subnormal), none_low, skip);
            let t = _mm256_blendv_epi8(biased, none_high, skip);
            min_e = _mm256_blendv_epi8(min_e, e, _mm256_cmpgt_epi64(min_e, e));
            max_t = _mm256_blendv_epi8(max_t, t, _mm256_cmpgt_epi64(t, max_t));
        }
        let min_e = lanes(min_e)
            .iter()
            .map(|&v| v as i64)
            .min()
            .unwrap_or(NONE_LOW);
        let max_t = lanes(max_t)
            .iter()
            .map(|&v| v as i64)
            .max()
            .unwrap_or(NONE_HIGH);
        let mut min_e = min_e;
        let mut max_t = max_t;
        for &x in chunks.remainder() {
            let bits = x.to_bits();
            let biased = ((bits >> 52) & EXP_MASK as u64) as i64;
            if biased != EXP_MASK && bits << 1 != 0 {
                min_e = min_e.min(biased.max(1));
                max_t = max_t.max(biased);
            }
        }
        (min_e, max_t)
    };
    if min_e == NONE_LOW {
        min_e = NORMAL_BIAS;
        max_t = 1023;
    }
    ((min_e - NORMAL_BIAS) as i32, (max_t - 1023) as i32)
}

/// # Safety
///
/// The caller has seen [`available`] return `true` on this machine.
#[target_feature(enable = "avx2")]
unsafe fn top_limb_avx2(xs: &[f64]) -> i16 {
    // SAFETY: the caller established AVX2; the one load per step reads the four doubles of a
    // `chunks_exact(4)` chunk.
    unsafe {
        let exp_mask = _mm256_set1_epi64x(EXP_MASK);
        let mut max_b = _mm256_setzero_si256();
        let mut chunks = xs.chunks_exact(4);
        for chunk in &mut chunks {
            let bits = _mm256_loadu_si256(chunk.as_ptr().cast::<__m256i>());
            let biased = _mm256_and_si256(_mm256_srli_epi64::<52>(bits), exp_mask);
            let finite = _mm256_andnot_si256(_mm256_cmpeq_epi64(biased, exp_mask), biased);
            max_b = _mm256_blendv_epi8(max_b, finite, _mm256_cmpgt_epi64(finite, max_b));
        }
        let mut top = lanes(max_b)
            .iter()
            .map(|&b| (b + 51) >> 6)
            .max()
            .unwrap_or(0) as i16;
        for &x in chunks.remainder() {
            let biased = (x.to_bits() >> 52) & EXP_MASK as u64;
            if biased != EXP_MASK as u64 {
                top = top.max(((biased + 51) >> 6) as i16);
            }
        }
        top
    }
}
