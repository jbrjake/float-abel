//! The NEON kernel: two doubles per 128-bit step, the portable kernel in scalar.rs op for op. NEON
//! is part of every Rust aarch64 target's baseline, so there is no runtime check and no
//! `#[target_feature]`; the intrinsics are called inside `unsafe` blocks because that is how both
//! the MSRV and the current compiler spell them.
//!
//! Two NEON facts shape the code. `USHL` takes its shift from the low byte of each lane as a
//! signed count, a negative count shifting right and any count past ±63 giving zero, so one
//! instruction places a mantissa at any limb or drops it, but the count has to fit a byte: the
//! per-limb counts come from a saturating byte chain centred on limb 1, which keeps every count
//! in range for shifts in `[-64, 191]`, the range the window checks leave. And `UADDW` adds a
//! vector's 32-bit halves, widened, into 64-bit lanes, which is the digit accumulator with no
//! carry to detect.

use core::arch::aarch64::{
    int8x16_t, vaddq_s64, vaddq_u64, vaddvq_u64, vaddw_high_u32, vaddw_u32, vandq_u64, vbicq_s64,
    vbicq_u64, vbslq_s64, vceqq_u64, vceqzq_u64, vcgtq_s64, vdupq_n_s64, vdupq_n_u64, veorq_u64,
    vget_low_u32, vgetq_lane_u64, vld1q_u64, vmaxvq_u32, vorrq_u64, vqaddq_s8, vqsubq_s8,
    vreinterpretq_s64_s8, vreinterpretq_s64_u64, vreinterpretq_s8_s64, vreinterpretq_s8_u64,
    vreinterpretq_u32_u64, vreinterpretq_u64_s64, vshlq_n_u64, vshlq_u64, vshrq_n_s64, vshrq_n_u64,
    vsubq_s64, vsubq_u64, vtstq_u64,
};

use super::{max_shift, Partials};

const FRAC_MASK: u64 = (1 << 52) - 1;
const EXP_MASK: u64 = 0x7FF;
const NORMAL_BIAS: i64 = 1075;

/// Whether this kernel covers the window: four limbs for the exact tiers, whose ceiling keeps every
/// accepted shift in `[-63, 138]`, and three bins for the truncating tier, whose shifts reach
/// `64K - 53` and are clamped below at `-64`. Both keep the byte chain in range.
pub(crate) const fn handles<const LIMBS: usize, const TRUNC: bool>() -> bool {
    if TRUNC {
        LIMBS <= 3
    } else {
        LIMBS <= 4
    }
}

/// The NEON fold of one tile; see `super::fold_tile` for the contract.
#[allow(clippy::too_many_lines)] // one straight pass; splitting it would hide the register budget
pub(crate) fn fold_tile<const LIMBS: usize, const TRUNC: bool>(
    xs: &[f64],
    lsb: i32,
    partials: &mut Partials<LIMBS>,
    leftover: &mut dyn FnMut(f64) -> bool,
) -> bool {
    debug_assert!(handles::<LIMBS, TRUNC>());
    // SAFETY: NEON is baseline on every aarch64 target, so every intrinsic's precondition holds;
    // the constants here touch no memory.
    let (frac_mask, exp_mask, implicit, sixty_four, minus_sixty_four, shift_base, max_ok, zero) = unsafe {
        (
            vdupq_n_u64(FRAC_MASK),
            vdupq_n_u64(EXP_MASK),
            vdupq_n_u64(1 << 52),
            vdupq_n_s64(64),
            vdupq_n_s64(-64),
            vdupq_n_s64(-NORMAL_BIAS - i64::from(lsb)),
            vdupq_n_s64(max_shift(LIMBS)),
            vdupq_n_u64(0),
        )
    };
    // SAFETY: as above; a `u64` lane whose low byte is 64 is the byte chain's step.
    let step: int8x16_t = unsafe { vreinterpretq_s8_u64(vdupq_n_u64(64)) };
    // Per limb, the digit sums of lane 0 and of lane 1: `[Σ low half, Σ high half]` each.
    let mut acc = [[zero; 2]; LIMBS];
    let mut negatives = zero;

    let mut chunks = xs.chunks_exact(2);
    for chunk in &mut chunks {
        // SAFETY: NEON is baseline on aarch64; the load reads the two doubles of `chunk`, which
        // `chunks_exact(2)` guarantees are there, as the two `u64` lanes of one register.
        let suspect_any = unsafe {
            let bits = vld1q_u64(chunk.as_ptr().cast::<u64>());
            let neg = vreinterpretq_u64_s64(vshrq_n_s64::<63>(vreinterpretq_s64_u64(bits)));
            let biased = vandq_u64(vshrq_n_u64::<52>(bits), exp_mask);
            let frac = vandq_u64(bits, frac_mask);
            let subnormal = vceqzq_u64(biased);
            let mant = vorrq_u64(frac, vbicq_u64(implicit, subnormal));
            // exp + 1075 is max(biased, 1): the subnormal mask is -1, so subtracting it adds one.
            let shift = vaddq_s64(
                vsubq_s64(
                    vreinterpretq_s64_u64(biased),
                    vreinterpretq_s64_u64(subnormal),
                ),
                shift_base,
            );
            let mut suspect = vceqq_u64(biased, exp_mask);
            if !TRUNC {
                // Bits below the grid: the mantissa shifted left by 64 + shift, for shifts in
                // [-63, -1]; a shift at or below -64 drops every bit, so the count clamps to zero
                // and the test is the mantissa itself; a shift at or above zero shifts by 64 or
                // more and tests nothing.
                let k = vaddq_s64(shift, sixty_four);
                let k = vbicq_s64(k, vshrq_n_s64::<63>(k));
                let low = vshlq_u64(mant, k);
                let off_grid = vtstq_u64(low, low);
                let over = vcgtq_s64(shift, max_ok);
                suspect = vorrq_u64(suspect, vorrq_u64(off_grid, over));
            }
            if vmaxvq_u32(vreinterpretq_u32_u64(suspect)) != 0 {
                true
            } else {
                // Everything below the window is dropped alike, so a truncating shift clamps to
                // -64 and the byte chain stays in range.
                let shift = if TRUNC {
                    vbslq_s64(vcgtq_s64(minus_sixty_four, shift), minus_sixty_four, shift)
                } else {
                    shift
                };
                let a1 = vreinterpretq_s8_s64(vsubq_s64(shift, sixty_four));
                let mut count = [a1; LIMBS];
                count[0] = vqaddq_s8(a1, step);
                for i in 2..LIMBS {
                    count[i] = vqsubq_s8(count[i - 1], step);
                }
                for (acc_i, &count_i) in acc.iter_mut().zip(&count) {
                    let w = veorq_u64(vshlq_u64(mant, vreinterpretq_s64_s8(count_i)), neg);
                    let halves = vreinterpretq_u32_u64(w);
                    acc_i[0] = vaddw_u32(acc_i[0], vget_low_u32(halves));
                    acc_i[1] = vaddw_high_u32(acc_i[1], halves);
                }
                negatives = vsubq_u64(negatives, neg);
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
    // SAFETY: NEON is baseline on aarch64; the reductions read registers only.
    unsafe {
        for (i, acc_i) in acc.iter().enumerate() {
            let both = vaddq_u64(acc_i[0], acc_i[1]);
            partials.lo[i] += vgetq_lane_u64::<0>(both);
            partials.hi[i] += vgetq_lane_u64::<1>(both);
        }
        partials.negatives += vaddvq_u64(negatives);
    }
    true
}

/// The span of a tile's finite nonzero inputs, as `super::scalar::span` defines it.
pub(crate) fn span(xs: &[f64]) -> (i32, i32) {
    const NONE_LOW: i64 = 1 << 40;
    const NONE_HIGH: i64 = -(1 << 40);
    // SAFETY: NEON is baseline on aarch64; the one load per step reads the two doubles of a
    // `chunks_exact(2)` chunk.
    let (min_e, max_t) = unsafe {
        let exp_mask = vdupq_n_u64(EXP_MASK);
        let none_low = vdupq_n_s64(NONE_LOW);
        let none_high = vdupq_n_s64(NONE_HIGH);
        let mut min_e = none_low;
        let mut max_t = none_high;
        let mut chunks = xs.chunks_exact(2);
        for chunk in &mut chunks {
            let bits = vld1q_u64(chunk.as_ptr().cast::<u64>());
            let biased = vandq_u64(vshrq_n_u64::<52>(bits), exp_mask);
            let skip = vorrq_u64(
                vceqq_u64(biased, exp_mask),
                vceqzq_u64(vshlq_n_u64::<1>(bits)),
            );
            let subnormal = vceqzq_u64(biased);
            let e = vsubq_s64(
                vreinterpretq_s64_u64(biased),
                vreinterpretq_s64_u64(subnormal),
            );
            let e = vbslq_s64(skip, none_low, e);
            let t = vbslq_s64(skip, none_high, vreinterpretq_s64_u64(biased));
            min_e = vbslq_s64(vcgtq_s64(min_e, e), e, min_e);
            max_t = vbslq_s64(vcgtq_s64(t, max_t), t, max_t);
        }
        let mut min_e = (vgetq_lane_u64::<0>(vreinterpretq_u64_s64(min_e)) as i64)
            .min(vgetq_lane_u64::<1>(vreinterpretq_u64_s64(min_e)) as i64);
        let mut max_t = (vgetq_lane_u64::<0>(vreinterpretq_u64_s64(max_t)) as i64)
            .max(vgetq_lane_u64::<1>(vreinterpretq_u64_s64(max_t)) as i64);
        for &x in chunks.remainder() {
            let bits = x.to_bits();
            let biased = ((bits >> 52) & EXP_MASK) as i64;
            if biased != EXP_MASK as i64 && bits << 1 != 0 {
                min_e = min_e.min(biased.max(1));
                max_t = max_t.max(biased);
            }
        }
        (min_e, max_t)
    };
    if min_e == NONE_LOW {
        (0, 0)
    } else {
        ((min_e - NORMAL_BIAS) as i32, (max_t - 1023) as i32)
    }
}

/// The `Wide` limb of the highest top bit among a tile's finite inputs, as
/// `super::scalar::top_limb` defines it.
pub(crate) fn top_limb(xs: &[f64]) -> i16 {
    // SAFETY: NEON is baseline on aarch64; the one load per step reads the two doubles of a
    // `chunks_exact(2)` chunk.
    unsafe {
        let exp_mask = vdupq_n_u64(EXP_MASK);
        let fifty_one = vdupq_n_u64(51);
        let mut max_b = vdupq_n_u64(0);
        let mut chunks = xs.chunks_exact(2);
        for chunk in &mut chunks {
            let bits = vld1q_u64(chunk.as_ptr().cast::<u64>());
            let biased = vandq_u64(vshrq_n_u64::<52>(bits), exp_mask);
            let finite = vbicq_u64(biased, vceqq_u64(biased, exp_mask));
            // Unsigned max of two small values through a signed compare, which is exact for them.
            let more = vcgtq_s64(vreinterpretq_s64_u64(finite), vreinterpretq_s64_u64(max_b));
            max_b = vorrq_u64(vandq_u64(more, finite), vbicq_u64(max_b, more));
        }
        let max_b = vaddq_u64(max_b, fifty_one);
        let top = vgetq_lane_u64::<0>(max_b).max(vgetq_lane_u64::<1>(max_b)) >> 6;
        let mut top = top as i16;
        for &x in chunks.remainder() {
            let biased = (x.to_bits() >> 52) & EXP_MASK;
            if biased != EXP_MASK {
                top = top.max(((biased + 51) >> 6) as i16);
            }
        }
        top
    }
}
