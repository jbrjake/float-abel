//! `Exact`'s codec, FORMAT.md: the body is the six fields little-endian, and the decoder is where
//! canonical form is checked, because a word with an even mantissa is not a value and structural
//! `Eq` must stay numeric equality. The checks run in one order, mantissa then exponent then
//! counts, so the first failing one names the error.
//!
//! Behind `alloc` the heap form has a body of its own for an engine's exchange, [`HEAP_TAG`]
//! then the four runs' headers and limbs, which the serde impls emit and accept beside the compact
//! encoding; its decoder refuses a body that is not normalised or whose value fits the compact
//! form, so one value still has one encoding. `to_bytes` and `from_bytes` are the compact form's.

#[cfg(all(feature = "alloc", feature = "serde"))]
use alloc::vec::Vec;

use super::{Exact, EXACT_MAX_EXP};
use crate::wire::{self, WireError};

impl Exact {
    /// The first byte of every encoded compact `Exact`: this type, this revision of its encoding.
    pub const WIRE_VERSION: u8 = 0x01;
    /// The length of [`Exact::to_bytes`]: the version byte and the 32-byte word.
    pub const BYTES: usize = 33;

    /// The canonical encoding of the compact form, FORMAT.md: [`Exact::WIRE_VERSION`], then the
    /// mantissa's low and high limbs, the exponent and the `[n, p, q]` counts, each
    /// little-endian. The body is the `repr(C)` bytes on a little-endian machine.
    ///
    /// # Panics
    ///
    /// On a value in the heap form, which has no 33-byte encoding; [`Exact::is_compact`] says
    /// which form a value is in, and a heap value crosses an engine's exchange as its serde bytes.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; Self::BYTES] {
        assert!(
            self.is_compact(),
            "Exact: to_bytes encodes the compact form, and this value is in the heap form"
        );
        let mut out = [0u8; Self::BYTES];
        out[0] = Self::WIRE_VERSION;
        out[1..9].copy_from_slice(&self.lo.to_le_bytes());
        out[9..17].copy_from_slice(&self.hi.to_le_bytes());
        out[17..21].copy_from_slice(&self.exp.to_le_bytes());
        out[21..25].copy_from_slice(&self.nan.to_le_bytes());
        out[25..29].copy_from_slice(&self.pos_inf.to_le_bytes());
        out[29..33].copy_from_slice(&self.neg_inf.to_le_bytes());
        out
    }

    /// The value [`Exact::to_bytes`] encoded, if the body is a canonical form: a byte pattern
    /// with an even mantissa is a second spelling of a value, and a decoder that accepted one
    /// would break `Eq`-substitutability in an engine's consolidation.
    ///
    /// # Errors
    ///
    /// [`WireError::Length`] unless `bytes` is exactly [`Exact::BYTES`] long, then
    /// [`WireError::Version`] unless its first byte is [`Exact::WIRE_VERSION`]. Then, on the
    /// body: [`WireError::EvenMantissa`] for a nonzero mantissa with its lowest bit clear,
    /// [`WireError::ZeroExponent`] for a zero mantissa with a nonzero exponent,
    /// [`WireError::Exponent`] for an exponent outside `±EXACT_MAX_EXP`, and [`WireError::Count`]
    /// for a count of `i32::MIN`. Every odd mantissa is inside the width, so there is no width
    /// refusal.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, WireError> {
        let body = wire::body(bytes, Self::WIRE_VERSION, Self::BYTES)?;
        let lo = wire::u64_le(&body[0..8]);
        let hi = wire::u64_le(&body[8..16]);
        let exp = wire::i32_le(&body[16..20]);
        let counts = [
            wire::i32_le(&body[20..24]),
            wire::i32_le(&body[24..28]),
            wire::i32_le(&body[28..32]),
        ];
        Self::canonical(lo, hi, exp, counts)
    }

    /// The word with these fields, if it is a canonical form: the one constructor every decoder
    /// goes through, the DBSP adapter's archived form and the `Arbitrary` draw included.
    pub(crate) fn canonical(
        lo: u64,
        hi: u64,
        exp: i32,
        counts: [i32; 3],
    ) -> Result<Self, WireError> {
        if lo | hi == 0 {
            if exp != 0 {
                return Err(WireError::ZeroExponent);
            }
        } else if lo & 1 == 0 {
            return Err(WireError::EvenMantissa);
        }
        if !(-EXACT_MAX_EXP..=EXACT_MAX_EXP).contains(&exp) {
            return Err(WireError::Exponent);
        }
        if counts.contains(&i32::MIN) {
            return Err(WireError::Count);
        }
        Ok(Self {
            lo,
            hi,
            exp,
            nan: counts[0],
            pos_inf: counts[1],
            neg_inf: counts[2],
        })
    }
}

/// The first byte of a heap body: the tag an engine's exchange carries a heap value under.
#[cfg(all(feature = "alloc", feature = "serde"))]
pub(crate) const HEAP_TAG: u8 = 0x04;

/// A run's header on the wire: a sign byte, an `i32` exponent and a `u32` limb count.
#[cfg(all(feature = "alloc", feature = "serde"))]
const RUN_HEADER: usize = 9;

#[cfg(all(feature = "alloc", feature = "serde"))]
impl Exact {
    /// Either form's encoding: [`Exact::to_bytes`] for the compact form, the heap body for the
    /// heap form.
    pub(crate) fn encode(&self) -> Vec<u8> {
        if self.is_compact() {
            return self.to_bytes().to_vec();
        }
        let p = self.parts();
        let runs = [p.finite(), p.amount(0), p.amount(1), p.amount(2)];
        let limbs: usize = runs.iter().map(|r| r.mag.len()).sum();
        let mut out = Vec::with_capacity(1 + 4 * RUN_HEADER + 8 * limbs);
        out.push(HEAP_TAG);
        for r in &runs {
            out.push(u8::from(r.neg));
            out.extend_from_slice(&(r.exp as i32).to_le_bytes());
            out.extend_from_slice(&(r.mag.len() as u32).to_le_bytes());
        }
        for r in &runs {
            for limb in r.mag {
                out.extend_from_slice(&limb.to_le_bytes());
            }
        }
        out
    }

    /// Either form's decoder: a body under [`Exact::WIRE_VERSION`] through [`Exact::from_bytes`],
    /// one under [`HEAP_TAG`] through the heap decoder, and anything else refused by its version.
    pub(crate) fn decode(bytes: &[u8]) -> Result<Self, WireError> {
        match bytes.first() {
            Some(&HEAP_TAG) => Self::decode_heap(&bytes[1..]),
            _ => Self::from_bytes(bytes),
        }
    }

    /// A heap body, validated before anything is allocated: four run headers, each count under
    /// the cap, the length they imply, then each run normalised, then a value that does not fit
    /// the compact form.
    fn decode_heap(body: &[u8]) -> Result<Self, WireError> {
        use super::big::Owned;
        use super::value::Value;
        if body.len() < 4 * RUN_HEADER {
            return Err(WireError::Length {
                expected: 1 + 4 * RUN_HEADER,
                found: 1 + body.len(),
            });
        }
        let mut headers = [(false, 0i32, 0usize); 4];
        for (i, h) in headers.iter_mut().enumerate() {
            let at = i * RUN_HEADER;
            let len = u32::from_le_bytes([body[at + 5], body[at + 6], body[at + 7], body[at + 8]]);
            if len as usize > super::EXACT_MAX_LIMBS {
                return Err(WireError::Limbs);
            }
            if body[at] > 1 {
                return Err(WireError::Form);
            }
            *h = (
                body[at] != 0,
                wire::i32_le(&body[at + 1..at + 5]),
                len as usize,
            );
        }
        let limbs: usize = headers.iter().map(|h| h.2).sum();
        let expected = 1 + 4 * RUN_HEADER + 8 * limbs;
        if 1 + body.len() != expected {
            return Err(WireError::Length {
                expected,
                found: 1 + body.len(),
            });
        }
        let mut runs = [Owned::ZERO, Owned::ZERO, Owned::ZERO, Owned::ZERO];
        let mut at = 4 * RUN_HEADER;
        for (run, &(neg, exp, len)) in runs.iter_mut().zip(&headers) {
            let mut mag = Vec::with_capacity(len + super::heap::HEADER);
            for _ in 0..len {
                mag.push(wire::u64_le(&body[at..at + 8]));
                at += 8;
            }
            if len == 0 {
                if neg || exp != 0 {
                    return Err(WireError::Form);
                }
            } else {
                if mag[0] & 1 == 0 {
                    return Err(WireError::EvenMantissa);
                }
                if mag[len - 1] == 0 {
                    return Err(WireError::Form);
                }
                if !(-EXACT_MAX_EXP..=EXACT_MAX_EXP).contains(&exp) {
                    return Err(WireError::Exponent);
                }
            }
            *run = Owned {
                neg,
                exp: i64::from(exp),
                mag,
            };
        }
        let [finite, nan, pos, neg] = runs;
        let value = Value {
            finite,
            amounts: [nan, pos, neg],
        };
        match Self::land(value) {
            Ok(v) if v.is_compact() => Err(WireError::Form),
            Ok(v) => Ok(v),
            Err(_) => Err(WireError::Exponent),
        }
    }
}
