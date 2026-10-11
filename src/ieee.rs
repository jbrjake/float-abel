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

/// Which non-finite value a double is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Special {
    /// Any NaN, whatever its sign or payload.
    Nan,
    PosInf,
    NegInf,
}

/// The special a non-finite double is, by its bits. The caller has seen [`decompose`] return
/// `None` for it.
pub(crate) fn special(x: f64) -> Special {
    let bits = x.to_bits();
    debug_assert_eq!((bits >> FRAC_BITS) & EXP_MASK, EXP_MASK, "{x} is finite");
    if bits & FRAC_MASK != 0 {
        Special::Nan
    } else if bits >> 63 != 0 {
        Special::NegInf
    } else {
        Special::PosInf
    }
}

/// The special that signed counts of NaN, +∞ and −∞ inputs project to, or `None` when they are
/// all zero: a NaN count, or both infinities present, is NaN; one infinity reads by the sign of its
/// count. Every tier that counts specials reads through this.
pub(crate) fn project(n: i64, p: i64, q: i64) -> Option<f64> {
    if n != 0 || (p != 0 && q != 0) {
        return Some(f64::NAN);
    }
    match (p.signum(), q.signum()) {
        (0, 0) => None,
        (1, _) | (_, -1) => Some(f64::INFINITY),
        _ => Some(f64::NEG_INFINITY),
    }
}

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

/// Positions of the NaN, +∞ and −∞ counts in a counter array, shared by every tier that counts.
pub(crate) const NAN: usize = 0;
pub(crate) const POS_INF: usize = 1;
pub(crate) const NEG_INF: usize = 2;

/// One factor of a product, taken apart.
pub(crate) enum Factor {
    Finite(Dyadic),
    Special(Special),
}

pub(crate) fn factor(x: f64) -> Factor {
    match decompose(x) {
        Some(d) => Factor::Finite(d),
        None => Factor::Special(special(x)),
    }
}

pub(crate) const fn counter(s: Special) -> usize {
    match s {
        Special::Nan => NAN,
        Special::PosInf => POS_INF,
        Special::NegInf => NEG_INF,
    }
}

/// Which counter a special times a nonzero finite factor lands in, and by how much: `g · P` is
/// `sign(g)` of `P`, and NaN absorbs.
pub(crate) const fn special_times_finite(s: Special, finite_negative: bool) -> (usize, i64) {
    match s {
        Special::Nan => (NAN, 1),
        _ => (counter(s), if finite_negative { -1 } else { 1 }),
    }
}

/// Which counter the product of two specials lands in: NaN absorbs, and the infinities follow the
/// sign rule, `(+∞)(+∞) = (−∞)(−∞) = +∞` and `(+∞)(−∞) = −∞`.
pub(crate) const fn special_times_special(s: Special, t: Special) -> usize {
    match (s, t) {
        (Special::Nan, _) | (_, Special::Nan) => NAN,
        (Special::PosInf, Special::PosInf) | (Special::NegInf, Special::NegInf) => POS_INF,
        _ => NEG_INF,
    }
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
    fn specials_are_none_and_classified_by_their_bits() {
        assert_eq!(decompose(f64::NAN), None);
        assert_eq!(decompose(f64::INFINITY), None);
        assert_eq!(decompose(f64::NEG_INFINITY), None);
        assert_eq!(special(f64::NAN), Special::Nan);
        assert_eq!(special(-f64::NAN), Special::Nan);
        assert_eq!(special(f64::from_bits(0x7FF0_0000_0000_0001)), Special::Nan);
        assert_eq!(special(f64::INFINITY), Special::PosInf);
        assert_eq!(special(f64::NEG_INFINITY), Special::NegInf);
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
