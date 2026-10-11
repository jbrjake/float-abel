//! Rounding an exact `(-1)^s · limbs · 2^exp` to a binary floating-point format in integer
//! arithmetic: round to nearest, ties to even, with subnormals, overflow and a sticky bit. Every
//! tier's `to_f64` and `div_f64` end here, which is what makes their reads agree bit for bit.

use crate::limbs;

/// A binary interchange format.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Format {
    /// Significand bits, implicit one included.
    precision: u32,
    /// Exponent field width.
    exp_bits: u32,
}

impl Format {
    const fn frac_bits(self) -> u32 {
        self.precision - 1
    }
    const fn bias(self) -> i32 {
        (1 << (self.exp_bits - 1)) - 1
    }
    /// The largest finite exponent, 1023 for `f64`.
    const fn emax(self) -> i32 {
        self.bias()
    }
    /// The smallest normal exponent, -1022 for `f64`.
    const fn emin(self) -> i32 {
        1 - self.bias()
    }
    /// The weight of the lowest subnormal bit, 2^-1074 for `f64`.
    const fn min_weight(self) -> i32 {
        self.emin() - self.frac_bits() as i32
    }
}

/// IEEE binary64.
pub(crate) const F64: Format = Format {
    precision: 53,
    exp_bits: 11,
};

/// Rounds the magnitude `limbs · 2^exp` to nearest-even in `fmt` and applies the sign. `sticky`
/// says that nonzero bits lie strictly below bit 0 of `limbs`; a caller passing it guarantees
/// the limbs carry at least `precision + 2` bits, so the round bit is always in view. Returns the
/// format's bit pattern and whether the result differs from the exact value.
///
/// A magnitude that rounds to zero keeps its sign, as IEEE rounds it. A zero magnitude is the
/// caller's zero: signed as the caller says.
pub(crate) fn round(
    negative: bool,
    limbs: &[u64],
    exp: i32,
    sticky: bool,
    fmt: Format,
) -> (u64, bool) {
    let sign = u64::from(negative) << (fmt.frac_bits() + fmt.exp_bits);
    let Some(top) = limbs::top_bit(limbs) else {
        debug_assert!(
            !sticky,
            "a sticky bit with no limbs in view cannot be rounded"
        );
        return (sign, sticky);
    };
    // The result's lowest bit, in limb coordinates: `precision` bits below the top, or the
    // format's lowest subnormal bit, whichever is higher.
    let normal_lsb = i64::from(top as i32) - i64::from(fmt.frac_bits());
    let subnormal_lsb = i64::from(fmt.min_weight()) - i64::from(exp);
    let lsb = normal_lsb.max(subnormal_lsb);
    let (kept, inexact) = if lsb <= 0 {
        debug_assert!(
            !sticky,
            "a sticky bit below a short value cannot be rounded"
        );
        // Fewer than `precision` bits and nothing below bit 0: exact.
        (limbs::window(limbs, 0, 64) << (-lsb) as u32, false)
    } else {
        let lsb = lsb as u64;
        let q = limbs::window(limbs, lsb, fmt.precision);
        let round_bit = limbs::window(limbs, lsb - 1, 1) != 0;
        let below = sticky || limbs::any_below(limbs, lsb - 1);
        let up = round_bit && (below || q & 1 == 1);
        (q + u64::from(up), round_bit || below)
    };
    (
        sign | compose(kept, exp + lsb as i32, fmt).0,
        inexact || compose(kept, exp + lsb as i32, fmt).1,
    )
}

/// The bits of `kept · 2^weight` for `kept < 2^(precision + 1)`, and whether it overflowed. The
/// value is representable by construction unless it is past the largest finite value.
fn compose(kept: u64, weight: i32, fmt: Format) -> (u64, bool) {
    if kept == 0 {
        return (0, false);
    }
    let bits = 64 - kept.leading_zeros();
    let e = weight + bits as i32 - 1;
    if e > fmt.emax() {
        let inf = ((1u64 << fmt.exp_bits) - 1) << fmt.frac_bits();
        return (inf, true);
    }
    let frac_mask = (1u64 << fmt.frac_bits()) - 1;
    if e >= fmt.emin() {
        let sig = if bits > fmt.precision {
            kept >> (bits - fmt.precision)
        } else {
            kept << (fmt.precision - bits)
        };
        let biased = (e + fmt.bias()) as u64;
        ((biased << fmt.frac_bits()) | (sig & frac_mask), false)
    } else {
        // Subnormal: `kept` sits at `weight`, at or above the lowest bit, with no implicit one.
        let shift = (weight - fmt.min_weight()) as u32;
        (kept << shift, false)
    }
}

/// The three-limb quotient window of `mag / n` with its exponent and sticky bit, ready for
/// [`round`]: `(window, exp, sticky)` with `window · 2^exp` the truncated quotient and `sticky`
/// whether anything was truncated. `mag` is nonzero and `n` is nonzero.
///
/// Long division from the top limb down, continuing into virtual zero limbs below limb 0, stops
/// two limbs after the first nonzero quotient limb: 128 bits beyond the leading one is more than
/// any format needs, and whatever remains is sticky.
pub(crate) fn div_window(mag: &[u64], exp: i32, n: u64) -> ([u64; 3], i32, bool) {
    debug_assert!(n != 0 && !limbs::is_zero(mag));
    let step = |i: i64, rem: u64| -> (u64, u64) {
        let limb = if i >= 0 {
            mag.get(i as usize).copied().unwrap_or(0)
        } else {
            0
        };
        let cur = (u128::from(rem) << 64) | u128::from(limb);
        ((cur / u128::from(n)) as u64, (cur % u128::from(n)) as u64)
    };
    let top = mag.iter().rposition(|&x| x != 0).map_or(0, |i| i as i64);
    // The first nonzero quotient limb is at `top` or, since `rem · 2^64 ≥ 2^64 > n` once a limb
    // has been carried down, at `top - 1`.
    let (q2, rem, first) = {
        let (q, rem) = step(top, 0);
        if q != 0 {
            (q, rem, top)
        } else {
            let (q, rem) = step(top - 1, rem);
            (q, rem, top - 1)
        }
    };
    let (q1, rem) = step(first - 1, rem);
    let (q0, rem) = step(first - 2, rem);
    let lower_limbs = (first - 2).clamp(0, mag.len() as i64) as usize;
    let sticky = rem != 0 || mag[..lower_limbs].iter().any(|&x| x != 0);
    ([q0, q1, q2], exp + 64 * (first - 2) as i32, sticky)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(limbs: &[u64], exp: i32) -> (f64, bool) {
        let (bits, inexact) = round(false, limbs, exp, false, F64);
        (f64::from_bits(bits), inexact)
    }

    /// Doubles chosen to be exact: every cast below is of a value under 2^54 with a zero low bit.
    #[allow(clippy::cast_precision_loss)]
    fn exact(n: u64) -> f64 {
        n as f64
    }

    #[test]
    fn exact_values_come_back_exact() {
        assert_eq!(r(&[1], 0), (1.0, false));
        assert_eq!(r(&[3], -1), (1.5, false));
        assert_eq!(r(&[(1 << 53) - 1], 0), (9_007_199_254_740_991.0, false));
        assert_eq!(r(&[0, 1], 0), (2f64.powi(64), false));
        assert_eq!(r(&[1], -1074), (5e-324, false));
        assert_eq!(r(&[(1 << 53) - 1], 971), (f64::MAX, false));
        assert_eq!(r(&[], 0), (0.0, false));
        assert_eq!(r(&[0, 0], 5), (0.0, false));
    }

    #[test]
    fn ties_go_to_even_and_the_rest_to_nearest() {
        let two53 = 1u64 << 53;
        assert_eq!(r(&[two53 + 1], 0), (exact(two53), true));
        assert_eq!(r(&[two53 + 2], 0), (exact(two53 + 2), false));
        assert_eq!(r(&[two53 + 3], 0), (exact(two53 + 4), true));
        assert_eq!(r(&[two53 + 5], 0), (exact(two53 + 4), true));
        assert_eq!(r(&[two53 + 7], 0), (exact(two53 + 8), true));
        // A tie that carries into the next binade.
        assert_eq!(r(&[(1 << 54) - 1], 0), (2f64.powi(54), true));
    }

    #[test]
    fn subnormals_round_at_the_format_floor() {
        assert_eq!(r(&[1], -1075), (0.0, true));
        assert_eq!(r(&[3], -1076), (5e-324, true));
        assert_eq!(r(&[3], -1075), (1e-323, true));
        assert_eq!(r(&[1, 0, 0], -1076), (0.0, true));
        let (bits, _) = round(true, &[1], -1075, false, F64);
        assert_eq!(
            bits,
            (-0.0f64).to_bits(),
            "a negative value that rounds to zero is -0.0"
        );
        assert_eq!(
            r(&[(1 << 52) - 1], -1074),
            (f64::from_bits((1 << 52) - 1), false)
        );
        assert_eq!(r(&[1 << 52], -1074), (f64::MIN_POSITIVE, false));
    }

    #[test]
    fn overflow_is_infinity() {
        assert_eq!(r(&[1], 1024), (f64::INFINITY, true));
        assert_eq!(
            r(&[(1 << 53) - 1, 1 << 62], 971 - 64),
            (f64::INFINITY, true)
        );
        let (bits, inexact) = round(true, &[1], 1024, false, F64);
        assert_eq!((f64::from_bits(bits), inexact), (f64::NEG_INFINITY, true));
    }

    #[test]
    fn sticky_breaks_a_tie_upward() {
        let two53 = 1u64 << 53;
        // 2^53 + 1 exactly ties to 2^53; with sticky bits below it is above the tie.
        let (bits, inexact) = round(false, &[0, 0, two53 + 1], -128, true, F64);
        assert_eq!((f64::from_bits(bits), inexact), (exact(two53 + 2), true));
    }

    #[test]
    fn div_window_feeds_round_the_correctly_rounded_quotient() {
        let div = |mag: [u64; 2], n: u64| {
            let (w, exp, sticky) = div_window(&mag, 0, n);
            assert_ne!(w[2], 0, "the window leads with a nonzero limb");
            let (bits, inexact) = round(false, &w, exp, sticky, F64);
            (f64::from_bits(bits), inexact)
        };
        // Exact operands: IEEE division is the correctly rounded quotient.
        assert_eq!(div([7, 0], 1), (7.0, false));
        assert_eq!(div([1, 0], 3), (1.0 / 3.0, true));
        assert_eq!(div([5, 0], 7), (5.0 / 7.0, true));
        assert_eq!(div([0, 1], 3), (2f64.powi(64) / 3.0, true));
        assert_eq!(div([0, 1], 1 << 40), (2f64.powi(24), false));
        // (2^128 - 1) / (2^64 - 1) = 2^64 + 1, which rounds down to 2^64.
        assert_eq!(div([u64::MAX, u64::MAX], u64::MAX), (2f64.powi(64), true));
        // (2^128 - 1) / 2 = 2^127 - 1/2, which rounds up to 2^127.
        assert_eq!(div([u64::MAX, u64::MAX], 2), (2f64.powi(127), true));
        // A quotient below one: the window is entirely fractional.
        assert_eq!(div([1, 0], u64::MAX), (2f64.powi(-64), true));
        assert_eq!(div([3, 0], 4), (0.75, false));
    }
}
