//! Taking a finite `f64` apart into an odd dyadic, with no floating-point instruction.

/// A finite double as `(-1)^negative · mant · 2^exp`, with `mant` odd, or zero as `mant == 0`
/// (then `exp == 0`, and `negative` keeps the zero's sign for the callers that want it).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Dyadic {
    pub(crate) negative: bool,
    pub(crate) mant: u64,
    pub(crate) exp: i32,
}

/// The weight of a fraction bit at biased exponent 0: 2^-1074, the smallest subnormal.
const SUBNORMAL_EXP: i32 = -1074;
/// The weight of fraction bit 0 at biased exponent `e` is 2^(e - 1075).
const NORMAL_BIAS: i32 = 1075;
const FRAC_BITS: u32 = 52;
const FRAC_MASK: u64 = (1 << FRAC_BITS) - 1;
const EXP_MASK: u64 = 0x7FF;

/// The dyadic a finite double denotes, or `None` for NaN and ±∞.
pub(crate) fn decompose(x: f64) -> Option<Dyadic> {
    let bits = x.to_bits();
    let negative = (bits >> 63) != 0;
    let biased = (bits >> FRAC_BITS) & EXP_MASK;
    let frac = bits & FRAC_MASK;
    if biased == EXP_MASK {
        return None;
    }
    let (mant, exp) = if biased == 0 {
        (frac, SUBNORMAL_EXP)
    } else {
        (frac | (1 << FRAC_BITS), biased as i32 - NORMAL_BIAS)
    };
    if mant == 0 {
        return Some(Dyadic {
            negative,
            mant: 0,
            exp: 0,
        });
    }
    let tz = mant.trailing_zeros();
    Some(Dyadic {
        negative,
        mant: mant >> tz,
        exp: exp + tz as i32,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `powi` of a negative exponent reciprocates the positive power, and 2^1049 is already
    /// infinite, so the scaling goes in two exact halves.
    #[allow(clippy::cast_precision_loss)] // a mantissa has at most 53 bits: the cast is exact
    fn recompose(d: Dyadic) -> f64 {
        let half = d.exp / 2;
        let m = d.mant as f64 * 2f64.powi(half) * 2f64.powi(d.exp - half);
        if d.negative {
            -m
        } else {
            m
        }
    }

    #[test]
    fn specials_are_none() {
        assert_eq!(decompose(f64::NAN), None);
        assert_eq!(decompose(f64::INFINITY), None);
        assert_eq!(decompose(f64::NEG_INFINITY), None);
    }

    #[test]
    fn zeros_keep_their_sign_and_have_no_exponent() {
        let z = decompose(0.0).unwrap();
        assert_eq!((z.negative, z.mant, z.exp), (false, 0, 0));
        let nz = decompose(-0.0).unwrap();
        assert_eq!((nz.negative, nz.mant, nz.exp), (true, 0, 0));
    }

    #[test]
    fn mantissa_is_odd_and_the_value_round_trips() {
        for x in [
            1.0,
            -1.0,
            0.1,
            0.5,
            3.0,
            1e300,
            -1e-300,
            f64::MAX,
            f64::MIN_POSITIVE,
            5e-324,
            f64::from_bits(2), // 2^-1073: `powi` would reciprocate an infinite 2^1073
            1.0 + f64::EPSILON,
        ] {
            let d = decompose(x).unwrap();
            assert_eq!(d.mant & 1, 1, "{x:e}");
            assert_eq!(recompose(d).to_bits(), x.to_bits(), "{x:e}");
        }
        assert_eq!(
            decompose(1.0).unwrap(),
            Dyadic {
                negative: false,
                mant: 1,
                exp: 0
            }
        );
        assert_eq!(
            decompose(5e-324).unwrap(),
            Dyadic {
                negative: false,
                mant: 1,
                exp: -1074
            }
        );
        assert_eq!(decompose(f64::MAX).unwrap().exp, 971);
        assert_eq!(decompose(f64::MAX).unwrap().mant, (1 << 53) - 1);
    }
}
