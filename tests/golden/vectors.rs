//! The two committed vector sets, decoded into the tiers that hold them the way tests/rounding.rs
//! and tests/wide.rs decode them. A rounding vector is `(-1)^negative · mant · 2^exp`, injected
//! exactly: its mantissa shifted onto a tier's grid and handed to `Wide::from_parts` or
//! `Fixed256::from_limbs`, and `Exact` taken from the `Wide` through the conversion, scaled back by
//! an exact power of two when the vector lies off `Wide`'s grid. A conformance case is a multiset
//! of doubles. The expected reads in the files are not consulted here: tests/main.rs asserts them,
//! and this crate hashes what the crate computes.

use float_abel::{Exact, Fixed256, Wide};

use crate::stream::pow2;

/// `(-1)^negative · mant · 2^exp`, the mantissa as limbs, low first, with no zero limb on top.
pub(crate) struct Dyadic {
    pub(crate) name: String,
    pub(crate) negative: bool,
    pub(crate) mant: Vec<u64>,
    pub(crate) exp: i32,
}

pub(crate) struct Quotient {
    pub(crate) v: Dyadic,
    pub(crate) n: u64,
}

fn hex_limbs(s: &str) -> Vec<u64> {
    let digits = s.strip_prefix("0x").unwrap();
    let mut limbs = Vec::new();
    let mut end = digits.len();
    while end > 0 {
        let start = end.saturating_sub(16);
        limbs.push(u64::from_str_radix(&digits[start..end], 16).unwrap());
        end = start;
    }
    while limbs.last() == Some(&0) {
        limbs.pop();
    }
    limbs
}

fn dyadic(v: &serde_json::Value) -> Dyadic {
    Dyadic {
        name: v["name"].as_str().unwrap().to_owned(),
        negative: v["negative"].as_bool().unwrap(),
        mant: hex_limbs(v["mant"].as_str().unwrap()),
        exp: v["exp"].as_i64().unwrap() as i32,
    }
}

/// The rounding set: every read vector, and every quotient vector with its divisor.
pub(crate) fn rounding() -> (Vec<Dyadic>, Vec<Quotient>) {
    let v: serde_json::Value =
        serde_json::from_str(include_str!("../data/rounding_vectors.json")).unwrap();
    assert_eq!(v["generator"], "scripts/gen_rounding_vectors.py");
    let reads = v["reads"].as_array().unwrap().iter().map(dyadic).collect();
    let quotients = v["quotients"]
        .as_array()
        .unwrap()
        .iter()
        .map(|q| Quotient {
            v: dyadic(q),
            n: q["n"].as_u64().unwrap(),
        })
        .collect();
    (reads, quotients)
}

/// bitrep's conformance set: each case's name and its inputs.
pub(crate) fn conformance() -> Vec<(String, Vec<f64>)> {
    let v: serde_json::Value =
        serde_json::from_str(include_str!("../data/bitrep/vectors.json")).unwrap();
    assert_eq!(
        v["format"], 1,
        "a new vector format: re-read bitrep's FORMAT.md"
    );
    v["cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            let inputs = c["input_bits"]
                .as_array()
                .unwrap()
                .iter()
                .map(|h| f64::from_bits(u64::from_str_radix(h.as_str().unwrap(), 16).unwrap()))
                .collect();
            (c["name"].as_str().unwrap().to_owned(), inputs)
        })
        .collect()
}

fn bit_length(mant: &[u64]) -> u64 {
    mant.last().map_or(0, |&top| {
        64 * (mant.len() as u64 - 1) + 64 - u64::from(top.leading_zeros())
    })
}

fn trailing_zeros(mant: &[u64]) -> u64 {
    mant.iter()
        .position(|&x| x != 0)
        .map_or(0, |i| 64 * i as u64 + u64::from(mant[i].trailing_zeros()))
}

/// The mantissa shifted `shift` bits up into `N` limbs, as a magnitude, or `None` when the shift is
/// negative (a set bit would fall below the grid) or the result needs more than `ceiling_bits`.
fn place<const N: usize>(mant: &[u64], shift: i64, ceiling_bits: u64) -> Option<[u64; N]> {
    let mut out = [0u64; N];
    if mant.is_empty() {
        return Some(out);
    }
    if shift < 0 || bit_length(mant) + shift as u64 > ceiling_bits {
        return None;
    }
    let (words, bits) = ((shift / 64) as usize, (shift % 64) as u32);
    for (i, &limb) in mant.iter().enumerate() {
        out[i + words] |= limb << bits;
        if bits != 0 && limb >> (64 - bits) != 0 {
            out[i + words + 1] |= limb >> (64 - bits);
        }
    }
    Some(out)
}

fn negate<const N: usize>(a: &mut [u64; N]) {
    let mut carry = 1u64;
    for x in a.iter_mut() {
        let (s, c) = (!*x).overflowing_add(carry);
        *x = s;
        carry = u64::from(c);
    }
}

pub(crate) fn wide_of(v: &Dyadic) -> Option<Wide> {
    let ceiling = (Wide::CEILING_EXP - Wide::LSB) as u64;
    let mut limbs = place::<{ Wide::LIMBS }>(&v.mant, i64::from(v.exp - Wide::LSB), ceiling)?;
    if v.negative {
        negate(&mut limbs);
    }
    Some(Wide::from_parts(limbs, [0; 3]))
}

pub(crate) fn fixed_of(v: &Dyadic) -> Option<Fixed256> {
    let ceiling = (Fixed256::CEILING_EXP - Fixed256::LSB) as u64;
    let mut limbs = place::<4>(&v.mant, i64::from(v.exp - Fixed256::LSB), ceiling)?;
    if v.negative {
        negate(&mut limbs);
    }
    Some(Fixed256::from_limbs(limbs))
}

/// `Exact` holds a vector iff the span from its lowest set bit to its highest fits 127 bits. One
/// inside `Wide`'s window comes through the conversion; one off the grid or past the ceiling is
/// built at a shifted exponent and scaled back by an exact power of two through the ring product.
pub(crate) fn exact_of(v: &Dyadic) -> Option<Exact> {
    if bit_length(&v.mant) - trailing_zeros(&v.mant) > 127 {
        return None;
    }
    if let Some(wide) = wide_of(v) {
        return Some(Exact::try_from(&wide).unwrap());
    }
    let shift = (-v.exp).clamp(-1023, 1074);
    let moved = Dyadic {
        name: v.name.clone(),
        negative: v.negative,
        mant: v.mant.clone(),
        exp: v.exp + shift,
    };
    let wide =
        wide_of(&moved).unwrap_or_else(|| panic!("{}: shifted by {shift} and still off", v.name));
    Some(Exact::try_from(&wide).unwrap() * Exact::from(pow2(-shift)))
}
