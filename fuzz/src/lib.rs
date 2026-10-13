//! The fuzz targets' oracle, DESIGN.md § Hardening. Every finite double is an exact count of
//! `2^-1074` units, so a multiset's sum is a `num-bigint` integer in those units beside three
//! signed counts of its specials. A target holds a tier to that integer bit for bit, through
//! `Wide`'s limbs, and to the IEEE reading of it, rounded here in integer arithmetic that shares
//! nothing with the crate's. The unit tests hold the rounding to the committed vectors.

// Limb arithmetic moves bits between widths on purpose, as the crate's does.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]

use float_abel::{Fixed256, OutOfWindow, Wide};
use num_bigint::{BigInt, BigUint};
use num_traits::{One, Zero};

/// `Fixed256`'s grid in units: `2^-96` is `2^978` of them.
const FIXED256_GRID: u64 = 978;
/// `Fixed256` accepts `|x| < 2^95`: a magnitude of at most `95 + 1074` bits in units.
const FIXED256_CEILING_BITS: u64 = 1169;

/// A binary floating-point format: the precision with its leading one, the exponent of its least
/// subnormal, the biased exponent that encodes infinity, and the width.
#[derive(Debug)]
pub struct Format {
    precision: u32,
    unit: i64,
    infinite: u64,
    width: u32,
}

/// IEEE binary64.
pub const F64: Format = Format {
    precision: 53,
    unit: -1074,
    infinite: 0x7FF,
    width: 64,
};

/// IEEE binary32.
pub const F32: Format = Format {
    precision: 24,
    unit: -149,
    infinite: 0xFF,
    width: 32,
};

/// `(-1)^negative · mag · 2^exp` rounded to nearest, ties to even, into the format's bits:
/// subnormal below the normal range, infinite above it, a zero magnitude the exact `+0`. The flag
/// says whether anything was lost.
///
/// # Panics
///
/// On a magnitude or exponent far outside anything a `Wide` holds.
#[must_use]
pub fn round(negative: bool, mag: &BigUint, exp: i64, f: &Format) -> (u64, bool) {
    if mag.is_zero() {
        return (0, false);
    }
    let sign = if negative { 1u64 << (f.width - 1) } else { 0 };
    let top = exp + i64::try_from(mag.bits()).expect("a bit count") - 1;
    // The grid the result lies on: `precision` significant bits, or the subnormal unit.
    let mut grid = (top - i64::from(f.precision) + 1).max(f.unit);
    let shift = grid - exp;
    let (mut q, inexact) = if shift <= 0 {
        (mag << u64::try_from(-shift).expect("non-negative"), false)
    } else {
        let shift = u64::try_from(shift).expect("positive");
        let q = mag >> shift;
        let rem = mag & ((BigUint::one() << shift) - BigUint::one());
        let half = BigUint::one() << (shift - 1);
        let up = rem > half || (rem == half && q.bit(0));
        (if up { q + BigUint::one() } else { q }, !rem.is_zero())
    };
    // A carry out of the precision moves the leading one up one binade.
    if q.bits() > u64::from(f.precision) {
        q >>= 1u32;
        grid += 1;
    }
    let q = q.to_u64_digits().first().copied().unwrap_or(0);
    let bits = if grid == f.unit {
        // Subnormal, or the first normal binade, whose field is the integer itself.
        q
    } else {
        let e = u64::try_from(grid - f.unit + 1).expect("above the unit");
        if e >= f.infinite {
            return (sign | f.infinite << (f.precision - 1), true);
        }
        (e << (f.precision - 1)) | (q - (1 << (f.precision - 1)))
    };
    (sign | bits, inexact)
}

/// A finite double as a count of `2^-1074` units, by its bits: `None` for NaN and ±∞.
#[must_use]
pub fn units(x: f64) -> Option<BigInt> {
    let bits = x.to_bits();
    let biased = (bits >> 52) & 0x7FF;
    let frac = bits & ((1 << 52) - 1);
    if biased == 0x7FF {
        return None;
    }
    let (mant, shift) = if biased == 0 {
        (frac, 0)
    } else {
        (frac | 1 << 52, biased - 1)
    };
    let mag = BigUint::from(mant) << shift;
    Some(if bits >> 63 == 1 {
        -BigInt::from(mag)
    } else {
        BigInt::from(mag)
    })
}

/// Which counter a special bumps: NaN, +∞, −∞.
#[must_use]
pub fn special(x: f64) -> usize {
    if x.is_nan() {
        0
    } else if x > 0.0 {
        1
    } else {
        2
    }
}

/// `n` as `N` two's-complement limbs, least significant first.
///
/// # Panics
///
/// If `n` needs more than `N` limbs.
#[must_use]
pub fn limbs<const N: usize>(n: &BigInt) -> [u64; N] {
    let bytes = n.to_signed_bytes_le();
    assert!(bytes.len() <= 8 * N, "{n} needs more than {N} limbs");
    let fill = if n.sign() == num_bigint::Sign::Minus {
        0xFF
    } else {
        0
    };
    let mut out = [0u64; N];
    for (i, limb) in out.iter_mut().enumerate() {
        let mut raw = [fill; 8];
        for (j, b) in raw.iter_mut().enumerate() {
            if let Some(&v) = bytes.get(8 * i + j) {
                *b = v;
            }
        }
        *limb = u64::from_le_bytes(raw);
    }
    out
}

/// The exact multiset: the finite sum in units and the three signed counts, `[n, p, q]`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Oracle {
    /// The finite sum, in units of `2^-1074`.
    pub sum: BigInt,
    /// How many NaN, +∞ and −∞ inputs, each negative when more were retracted than inserted.
    pub counts: [i64; 3],
}

impl Oracle {
    /// Adds `x` with multiplicity `by`: `1` inserts, `-1` retracts.
    pub fn add(&mut self, x: f64, by: i64) {
        match units(x) {
            Some(n) => self.sum += n * by,
            None => self.counts[special(x)] += by,
        }
    }

    /// Whether nothing is held.
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.sum.is_zero() && self.counts == [0; 3]
    }

    /// The projection, DESIGN.md § NaN and ±∞: NaN with any NaN or both infinities present, else
    /// one infinity by the sign of its count, else `None` for the finite part.
    #[must_use]
    pub fn projection(&self) -> Option<f64> {
        let [n, p, q] = self.counts;
        if n != 0 || (p != 0 && q != 0) {
            return Some(f64::NAN);
        }
        match (p.signum(), q.signum()) {
            (0, 0) => None,
            (1, _) | (_, -1) => Some(f64::INFINITY),
            _ => Some(f64::NEG_INFINITY),
        }
    }

    /// The reading as a double and whether it is exact: a projection passes through, and the
    /// finite sum rounds to nearest-even.
    #[must_use]
    pub fn to_f64(&self) -> (f64, bool) {
        if let Some(special) = self.projection() {
            return (special, true);
        }
        let (bits, inexact) = round(
            self.sum.sign() == num_bigint::Sign::Minus,
            self.sum.magnitude(),
            -1074,
            &F64,
        );
        (f64::from_bits(bits), !inexact)
    }

    /// The reading as a single, rounded once from the exact sum.
    #[must_use]
    pub fn to_f32(&self) -> f32 {
        if let Some(special) = self.projection() {
            return special as f32;
        }
        let (bits, _) = round(
            self.sum.sign() == num_bigint::Sign::Minus,
            self.sum.magnitude(),
            -1074,
            &F32,
        );
        f32::from_bits(bits as u32)
    }

    /// The `Wide` holding exactly this multiset: the sum as 34 two's-complement limbs, then the
    /// counts.
    #[must_use]
    pub fn to_wide(&self) -> Wide {
        Wide::from_parts(limbs::<34>(&self.sum), self.counts)
    }
}

/// `Fixed256`'s verdict on `x`, from the window alone: finite, on the `2^-96` grid, under `2^95`.
///
/// # Errors
///
/// The refusal by name, in the crate's order.
pub fn fixed256_verdict(x: f64) -> Result<(), OutOfWindow> {
    let n = units(x).ok_or(OutOfWindow::NotFinite)?;
    if n.is_zero() {
        return Ok(());
    }
    if n.trailing_zeros().unwrap_or(0) < FIXED256_GRID {
        return Err(OutOfWindow::OffGrid);
    }
    if n.bits() > FIXED256_CEILING_BITS {
        return Err(OutOfWindow::AboveCeiling);
    }
    Ok(())
}

/// The `Fixed256` holding exactly `sum` units, which must lie on its grid.
///
/// # Panics
///
/// If `sum` has a set bit below the grid.
#[must_use]
pub fn to_fixed256(sum: &BigInt) -> Fixed256 {
    assert!(
        sum.is_zero() || sum.trailing_zeros().unwrap_or(0) >= FIXED256_GRID,
        "{sum} is off Fixed256's grid"
    );
    Fixed256::from_limbs(limbs::<4>(&(sum >> FIXED256_GRID)))
}

/// `Binned<K>`'s definition, DESIGN.md § Binned, on a fold of `xs`: the `Wide` limb at the
/// window's bottom (the largest magnitude's top limb, never below `K - 1`, less `K - 1`), and
/// the sum of the finite inputs with each magnitude cut below that limb, specials counted.
#[must_use]
pub fn binned_truncated<const K: usize>(xs: &[f64]) -> (u64, Oracle) {
    let top = xs
        .iter()
        .filter_map(|&x| units(x))
        .filter(|n| !n.is_zero())
        .map(|n| (n.bits() - 1) / 64)
        .max()
        .unwrap_or(0);
    let k = K as u64;
    let lo = top.max(k - 1) + 1 - k;
    let mut oracle = Oracle::default();
    for &x in xs {
        match units(x) {
            Some(n) => {
                let kept = (n.magnitude() >> (64 * lo)) << (64 * lo);
                oracle.sum += BigInt::from_biguint(n.sign(), kept);
            }
            None => oracle.counts[special(x)] += 1,
        }
    }
    (lo, oracle)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex_mag(s: &str) -> BigUint {
        BigUint::parse_bytes(s.strip_prefix("0x").unwrap().as_bytes(), 16).unwrap()
    }

    fn hex_u64(s: &str) -> u64 {
        u64::from_str_radix(s.strip_prefix("0x").unwrap(), 16).unwrap()
    }

    /// 2^e as a double, by its bits.
    fn pow2(e: i32) -> f64 {
        assert!((-1074..=1023).contains(&e));
        if e >= -1022 {
            f64::from_bits(((e + 1023) as u64) << 52)
        } else {
            f64::from_bits(1u64 << (e + 1074))
        }
    }

    /// splitmix64.
    fn next(state: &mut u64) -> u64 {
        *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = *state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// The oracle's rounding against the vectors `scripts/gen_rounding_vectors.py` generated in
    /// Python, every read vector in both formats.
    #[test]
    fn rounding_matches_the_committed_vectors() {
        let v: serde_json::Value =
            serde_json::from_str(include_str!("../../tests/data/rounding_vectors.json")).unwrap();
        let reads = v["reads"].as_array().unwrap();
        assert_eq!(reads.len(), 173);
        for r in reads {
            let name = r["name"].as_str().unwrap();
            let negative = r["negative"].as_bool().unwrap();
            let mag = hex_mag(r["mant"].as_str().unwrap());
            let exp = r["exp"].as_i64().unwrap();
            let f64_bits = hex_u64(r["f64"].as_str().unwrap());
            let f32_bits = hex_u64(r["f32"].as_str().unwrap());
            let exact = r["f64_exact"].as_bool().unwrap();
            assert_eq!(
                round(negative, &mag, exp, &F64),
                (f64_bits, !exact),
                "{name}"
            );
            assert_eq!(round(negative, &mag, exp, &F32).0, f32_bits, "{name}: f32");
        }
    }

    #[test]
    fn a_double_is_its_units_and_a_one_element_multiset_reads_it_back() {
        let mut state = 1;
        let fixed = [
            0.0,
            -0.0,
            1.0,
            -1.5,
            0.1,
            5e-324,
            -5e-324,
            f64::MIN_POSITIVE,
            f64::MAX,
            -f64::MAX,
            1e-300,
            1e300,
            f64::from_bits(0x000F_FFFF_FFFF_FFFF),
        ];
        let random = (0..2000).map(|_| f64::from_bits(next(&mut state)));
        let mut seen = 0;
        for x in fixed.into_iter().chain(random).filter(|x| x.is_finite()) {
            seen += 1;
            let mut o = Oracle::default();
            o.add(x, 1);
            let (v, exact) = o.to_f64();
            assert!(exact, "{x:e}");
            // Either zero is the one zero state, which reads +0 in both formats.
            let want = if x == 0.0 { 0 } else { x.to_bits() };
            assert_eq!(v.to_bits(), want, "{x:e}");
            assert_eq!(o.to_wide(), Wide::from(x), "{x:e}");
            let want32 = if x == 0.0 { 0 } else { (x as f32).to_bits() };
            assert_eq!(o.to_f32().to_bits(), want32, "{x:e} as f32");
            let mut back = o.clone();
            back.add(x, -1);
            assert!(back.is_zero());
        }
        assert!(seen > 1500, "{seen} finite draws");
        // Specials are counted and projected: NaN first, then both infinities, then one by the
        // sign of its count.
        assert_eq!(units(f64::NAN), None);
        let mut o = Oracle::default();
        o.add(f64::INFINITY, 1);
        assert_eq!(o.to_f64(), (f64::INFINITY, true));
        o.add(f64::NEG_INFINITY, 1);
        assert!(o.to_f64().0.is_nan());
        o.add(f64::NEG_INFINITY, -1);
        o.add(f64::INFINITY, -2);
        assert_eq!(o.counts, [0, -1, 0]);
        assert_eq!(o.to_f64(), (f64::NEG_INFINITY, true));
        o.add(f64::NAN, 1);
        assert!(o.to_f64().0.is_nan());
        assert!(o.to_f32().is_nan());
    }

    #[test]
    fn limbs_are_twos_complement_and_fixed256_sees_its_window() {
        assert_eq!(limbs::<2>(&BigInt::from(-1)), [u64::MAX; 2]);
        assert_eq!(limbs::<2>(&(BigInt::one() << 64u32)), [0, 1]);
        assert_eq!(limbs::<2>(&-(BigInt::one() << 64u32)), [0, u64::MAX]);
        assert_eq!(limbs::<3>(&BigInt::from(5)), [5, 0, 0]);
        for (x, verdict) in [
            (1.0, Ok(())),
            (0.0, Ok(())),
            (pow2(-44), Ok(())),
            (-pow2(-96), Ok(())),
            (pow2(95) * (1.0 - f64::EPSILON / 2.0), Ok(())),
            (pow2(-97), Err(OutOfWindow::OffGrid)),
            (5e-324, Err(OutOfWindow::OffGrid)),
            (pow2(95), Err(OutOfWindow::AboveCeiling)),
            (f64::MAX, Err(OutOfWindow::AboveCeiling)),
            (f64::NAN, Err(OutOfWindow::NotFinite)),
            (f64::NEG_INFINITY, Err(OutOfWindow::NotFinite)),
        ] {
            assert_eq!(fixed256_verdict(x), verdict, "{x:e}");
            assert_eq!(
                Fixed256::try_from(x).map(|_| ()),
                verdict,
                "{x:e}: the crate"
            );
        }
        for x in [1.0, -1.0, 0.75, pow2(-44), -pow2(94)] {
            assert_eq!(
                to_fixed256(&units(x).unwrap()),
                Fixed256::try_from(x).unwrap(),
                "{x:e}"
            );
        }
    }

    /// The design's not-a-group history at the widths tests/binned.rs pins: at `K = 3` the window
    /// starts at limb 16 and 16 bits of `1e-10` survive; at `K = 1` nothing does.
    #[test]
    fn binned_truncation_is_the_documented_history() {
        let history = [1e30, 1e-10, -1e30];
        let (lo, o) = binned_truncated::<3>(&history);
        assert_eq!(lo, 16);
        assert_eq!(o.to_f64().0, 112_589.0 * pow2(-50));
        let (lo, o) = binned_truncated::<1>(&history);
        assert_eq!(lo, 18);
        assert!(o.is_zero());
        let (lo, o) = binned_truncated::<3>(&[f64::NAN, 2.5]);
        assert_eq!(lo, 14);
        assert_eq!(o.counts, [1, 0, 0]);
        assert_eq!(binned_truncated::<3>(&[]).0, 0);
    }
}
