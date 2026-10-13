//! The portable kernel: the lane algorithm in plain Rust over `[u64; N]` arrays, with no
//! floating-point instruction and no `unsafe`. It is what every target without a vector kernel
//! runs, and it is the specification the vector kernels mirror op for op: the same decode, the
//! same soundness test, the same placement, the same digit accumulators. Its lane count is a
//! parameter so the tests can show the bits do not depend on it.

use super::{max_shift, Partials};

const FRAC_BITS: u32 = 52;
const FRAC_MASK: u64 = (1 << FRAC_BITS) - 1;
const EXP_MASK: u64 = 0x7FF;
/// The weight of fraction bit 0 at biased exponent `e` is `2^(e - 1075)`; a subnormal's is that
/// of `e = 1`.
const NORMAL_BIAS: i64 = 1075;
const LO32: u64 = 0xFFFF_FFFF;

/// One double taken apart against a window: its mantissa, the shift that puts the mantissa's bit 0
/// on the window's bit 0, the sign as a mask, and whether the lane may place it.
#[derive(Clone, Copy, Default)]
struct Lane {
    mant: u64,
    shift: i64,
    /// `0` for a positive input, `!0` for a negative one.
    neg: u64,
    /// Set when the lane cannot vouch for the input: a special, a set bit below the grid, or a
    /// magnitude that may reach the ceiling. The scalar deposit takes it instead.
    suspect: bool,
}

impl Lane {
    fn decode<const LIMBS: usize, const TRUNC: bool>(bits: u64, lsb: i32) -> Self {
        let neg = ((bits as i64) >> 63) as u64;
        let biased = (bits >> FRAC_BITS) & EXP_MASK;
        let frac = bits & FRAC_MASK;
        let subnormal = biased == 0;
        let mant = if subnormal {
            frac
        } else {
            frac | (1 << FRAC_BITS)
        };
        let exp = if subnormal { 1 } else { biased as i64 } - NORMAL_BIAS;
        let shift = exp - i64::from(lsb);
        let special = biased == EXP_MASK;
        let suspect = if TRUNC {
            special
        } else {
            // Bits below the grid: the mantissa bits a right shift by `-shift` would drop. A shift
            // of -64 or less drops them all, so the test is the mantissa itself.
            let low = if shift >= 0 {
                0
            } else if shift > -64 {
                mant << (64 + shift)
            } else {
                mant
            };
            special || low != 0 || shift > max_shift(LIMBS)
        };
        Self {
            mant,
            shift,
            neg,
            suspect,
        }
    }

    /// The 64-bit word of `mant · 2^shift` at limb `i`, before the sign is applied: what
    /// `limbs::set_shifted` would write there, with bits below the window dropped.
    fn word(&self, i: usize) -> u64 {
        let a = self.shift - 64 * i as i64;
        if a >= 64 || a <= -64 {
            0
        } else if a >= 0 {
            self.mant << a
        } else {
            self.mant >> -a
        }
    }
}

/// The portable fold of one tile, `N` lanes wide; see `super::fold_tile` for the contract.
pub(crate) fn fold_tile<const N: usize, const LIMBS: usize, const TRUNC: bool>(
    xs: &[f64],
    lsb: i32,
    partials: &mut Partials<LIMBS>,
    leftover: &mut dyn FnMut(f64) -> bool,
) -> bool {
    let mut lo = [[0u64; N]; LIMBS];
    let mut hi = [[0u64; N]; LIMBS];
    let mut negatives = [0u64; N];
    let mut chunks = xs.chunks_exact(N);
    for chunk in &mut chunks {
        let mut lanes = [Lane::default(); N];
        let mut suspect = false;
        for (lane, &x) in lanes.iter_mut().zip(chunk) {
            *lane = Lane::decode::<LIMBS, TRUNC>(x.to_bits(), lsb);
            suspect |= lane.suspect;
        }
        if suspect {
            for &x in chunk {
                if !leftover(x) {
                    return false;
                }
            }
            continue;
        }
        for (i, (lo_i, hi_i)) in lo.iter_mut().zip(hi.iter_mut()).enumerate() {
            for ((l, h), lane) in lo_i.iter_mut().zip(hi_i.iter_mut()).zip(&lanes) {
                let w = lane.word(i) ^ lane.neg;
                *l += w & LO32;
                *h += w >> 32;
            }
        }
        for (n, lane) in negatives.iter_mut().zip(&lanes) {
            *n += lane.neg & 1;
        }
    }
    for &x in chunks.remainder() {
        if !leftover(x) {
            return false;
        }
    }
    for i in 0..LIMBS {
        partials.lo[i] += lo[i].iter().sum::<u64>();
        partials.hi[i] += hi[i].iter().sum::<u64>();
    }
    partials.negatives += negatives.iter().sum::<u64>();
    true
}

/// The span of a tile's finite nonzero inputs: the lowest exponent of any mantissa's bit 0, and
/// the highest top bit any normal has (a subnormal counts as `-1023`, a bound on its top). Both
/// `0` when there is none; the window then holds nothing but zeros, and any works.
pub(crate) fn span(xs: &[f64]) -> (i32, i32) {
    let mut min_exp = i32::MAX;
    let mut max_top = i32::MIN;
    for &x in xs {
        let bits = x.to_bits();
        let biased = ((bits >> FRAC_BITS) & EXP_MASK) as i32;
        if biased == EXP_MASK as i32 || bits << 1 == 0 {
            continue;
        }
        min_exp = min_exp.min(biased.max(1) - NORMAL_BIAS as i32);
        max_top = max_top.max(biased - 1023);
    }
    if min_exp == i32::MAX {
        (0, 0)
    } else {
        (min_exp, max_top)
    }
}

/// The `Wide` limb of the highest top bit among a tile's finite inputs: `(biased + 51) / 64`,
/// which is limb 0 for every subnormal and zero, as the top bit of a normal is `2^(biased - 1023)`
/// and sits `biased + 51` bits above `2^-1074`.
pub(crate) fn top_limb(xs: &[f64]) -> i16 {
    let mut top = 0u64;
    for &x in xs {
        let biased = (x.to_bits() >> FRAC_BITS) & EXP_MASK;
        if biased != EXP_MASK {
            top = top.max((biased + 51) >> 6);
        }
    }
    top as i16
}

#[cfg(test)]
mod tests {
    use super::*;

    /// splitmix64, so the test needs no `std` randomness.
    fn next(state: &mut u64) -> u64 {
        *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = *state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A multiset of mixed kinds: any bits one time in four (specials and subnormals included),
    /// else a normal with its top bit in `[2^-40, 2^60)`.
    fn multiset(state: &mut u64, len: usize) -> [f64; 2048] {
        let mut xs = [0.0; 2048];
        for x in xs.iter_mut().take(len) {
            let r = next(state);
            *x = if r % 4 == 0 {
                f64::from_bits(next(state))
            } else {
                let e = (1023 - 40 + (r % 100) as i64) as u64;
                f64::from_bits((r & (1 << 63)) | (e << 52) | (next(state) >> 12))
            };
        }
        xs
    }

    /// The inputs a run handed to the scalar path, in a fixed buffer: the lib is `no_std`.
    struct Rest {
        bits: [u64; 2048],
        len: usize,
    }

    impl Rest {
        fn as_slice(&self) -> &[u64] {
            &self.bits[..self.len]
        }
    }

    fn run<const N: usize, const LIMBS: usize, const TRUNC: bool>(
        xs: &[f64],
        lsb: i32,
    ) -> (Partials<LIMBS>, Rest) {
        let mut p = Partials::<LIMBS>::ZERO;
        let mut rest = Rest {
            bits: [0; 2048],
            len: 0,
        };
        let went_on = fold_tile::<N, LIMBS, TRUNC>(xs, lsb, &mut p, &mut |x| {
            rest.bits[rest.len] = x.to_bits();
            rest.len += 1;
            true
        });
        assert!(went_on);
        (p, rest)
    }

    /// The tile's value is the same under every lane count: the partials differ in which inputs
    /// the lanes took and which fell to the scalar path, so the comparison is the merged sum plus
    /// the scalar leftovers, each merged exactly.
    fn total<const LIMBS: usize>(p: &Partials<LIMBS>, rest: &[u64], lsb: i32) -> [u64; LIMBS] {
        let mut sum = p.twos_complement();
        for &bits in rest {
            let lane = Lane::decode::<LIMBS, true>(bits, lsb);
            assert!(!lane.suspect || (bits >> 52) & 0x7FF == 0x7FF);
            if lane.suspect {
                continue;
            }
            let mut term = [0u64; LIMBS];
            for (i, t) in term.iter_mut().enumerate() {
                *t = lane.word(i);
            }
            if lane.neg != 0 {
                crate::limbs::negate(&mut term);
            }
            crate::limbs::add_assign(&mut sum, &term);
        }
        sum
    }

    #[test]
    fn simd_portable_kernel_lands_the_same_bits_at_one_two_four_and_eight_lanes() {
        let mut state = 0x4C41_4E45;
        for round in 0..crate::exact::testing::rounds(300) {
            let len = (next(&mut state) % 2048) as usize;
            let xs = multiset(&mut state, len);
            let xs = &xs[..len];
            let lsb = -1074 + 64 * (round % 20) as i32;
            let (p1, r1) = run::<1, 4, true>(xs, lsb);
            let (p2, r2) = run::<2, 4, true>(xs, lsb);
            let (p4, r4) = run::<4, 4, true>(xs, lsb);
            let (p8, r8) = run::<8, 4, true>(xs, lsb);
            let want = total(&p1, r1.as_slice(), lsb);
            assert_eq!(
                total(&p2, r2.as_slice(), lsb),
                want,
                "round {round}: 2 lanes"
            );
            assert_eq!(
                total(&p4, r4.as_slice(), lsb),
                want,
                "round {round}: 4 lanes"
            );
            assert_eq!(
                total(&p8, r8.as_slice(), lsb),
                want,
                "round {round}: 8 lanes"
            );
            // Every input went one way or the other, and the one-lane kernel keeps only specials
            // for the scalar path in truncating mode.
            assert_eq!(
                r1.len,
                xs.iter().filter(|x| !x.is_finite()).count(),
                "round {round}"
            );
            assert!(r8.len >= r1.len, "round {round}");
        }
    }

    #[test]
    fn simd_portable_fold_at_four_lanes_equals_the_limb_arithmetic_oracle_on_the_exact_window() {
        // The oracle is the tier's own arithmetic: set_shifted, negate and add_assign on four
        // limbs, over every input the kernel vouched for; the kernel's words never enter it.
        let mut state = 0x4F52_4143;
        for round in 0..crate::exact::testing::rounds(300) {
            let len = (next(&mut state) % 2048) as usize;
            let xs = multiset(&mut state, len);
            let xs = &xs[..len];
            let lsb = -60 + (round % 40) as i32;
            let (p, rest) = run::<4, 4, false>(xs, lsb);
            let mut want = [0u64; 4];
            let mut placed = 0usize;
            let rest = rest.as_slice();
            let mut r = 0;
            for &x in xs {
                let bits = x.to_bits();
                if r < rest.len() && rest[r] == bits {
                    r += 1;
                    continue;
                }
                let d = crate::ieee::decompose(x).expect("the kernel vouched for it: finite");
                let shift = d.exp - lsb;
                assert!(
                    shift >= 0 || d.mant == 0,
                    "round {round}: {x:e} is below the grid"
                );
                let mut term = [0u64; 4];
                if d.mant != 0 {
                    crate::limbs::set_shifted(&mut term, u128::from(d.mant), shift as u32);
                }
                if d.negative {
                    crate::limbs::negate(&mut term);
                }
                crate::limbs::add_assign(&mut want, &term);
                placed += 1;
            }
            assert_eq!(
                r,
                rest.len(),
                "round {round}: every leftover is an input, in order"
            );
            assert_eq!(placed + rest.len(), xs.len(), "round {round}");
            assert_eq!(p.twos_complement(), want, "round {round}: {placed} placed");
        }
    }

    #[test]
    fn simd_portable_lane_places_what_set_shifted_writes() {
        let mut state = 7;
        for _ in 0..crate::exact::testing::rounds(20_000) {
            let bits = next(&mut state);
            let lsb = -1200 + (next(&mut state) % 2400) as i32;
            let lane = Lane::decode::<4, true>(bits, lsb);
            if lane.suspect {
                assert!(!f64::from_bits(bits).is_finite());
                continue;
            }
            // The reference: the mantissa shifted by `shift` into a 256-bit window, bits below the
            // window dropped by shifting a wider word.
            let want: [u64; 4] = if lane.shift >= 0 {
                let mut w = [0u64; 6];
                if lane.shift < 320 {
                    crate::limbs::set_shifted(&mut w, u128::from(lane.mant), lane.shift as u32);
                }
                [w[0], w[1], w[2], w[3]]
            } else {
                let v = u128::from(lane.mant)
                    .checked_shr((-lane.shift) as u32)
                    .unwrap_or(0);
                [v as u64, (v >> 64) as u64, 0, 0]
            };
            let got = [lane.word(0), lane.word(1), lane.word(2), lane.word(3)];
            assert_eq!(got, want, "bits {bits:#x} at lsb {lsb}");
            assert_eq!(lane.neg != 0, bits >> 63 == 1);
        }
    }

    #[test]
    fn simd_portable_soundness_test_refuses_nothing_the_window_holds_and_vouches_for_nothing_else()
    {
        let mut state = 99;
        for _ in 0..crate::exact::testing::rounds(50_000) {
            let bits = next(&mut state);
            let x = f64::from_bits(bits);
            let lsb = -1100 + (next(&mut state) % 2200) as i32;
            let lane = Lane::decode::<4, false>(bits, lsb);
            if lane.suspect {
                continue;
            }
            // Vouched for: finite, on the grid, and under the ceiling, as Fixed<lsb, 4> judges.
            let d = crate::ieee::decompose(x).expect("a special is always suspect");
            if d.mant != 0 {
                assert!(d.exp >= lsb, "{x:e} at lsb {lsb}: below the grid");
                let top = i64::from(d.exp) + i64::from(d.mant.ilog2()) - i64::from(lsb);
                assert!(
                    top < super::super::ceiling_bits(4),
                    "{x:e} at lsb {lsb}: at the ceiling"
                );
            }
        }
        // And the other direction on normals: a normal the window holds is never suspect.
        for _ in 0..crate::exact::testing::rounds(50_000) {
            let bits = next(&mut state) & !(0x7FFu64 << 52) | ((1 + next(&mut state) % 2046) << 52);
            let x = f64::from_bits(bits);
            let d = crate::ieee::decompose(x).unwrap();
            let lsb = -1100 + (next(&mut state) % 2200) as i32;
            let top = i64::from(d.exp) + i64::from(d.mant.ilog2()) - i64::from(lsb);
            let held = d.exp >= lsb && top < super::super::ceiling_bits(4);
            let lane = Lane::decode::<4, false>(bits, lsb);
            assert_eq!(!lane.suspect, held, "{x:e} at lsb {lsb}");
        }
    }
}
