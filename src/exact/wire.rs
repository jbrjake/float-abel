//! `Exact`'s codec, FORMAT.md: the body is the six fields little-endian, and the decoder is where
//! canonical form is checked, because a word with an even mantissa is not a value and structural
//! `Eq` must stay numeric equality. The checks run in one order, mantissa then exponent then
//! counts, so the first failing one names the error.

use super::{Exact, EXACT_MAX_EXP};
use crate::wire::{self, WireError};

impl Exact {
    /// The first byte of every encoded `Exact`: this type, this revision of its encoding.
    pub const WIRE_VERSION: u8 = 0x01;
    /// The length of [`Exact::to_bytes`]: the version byte and the 32-byte word.
    pub const BYTES: usize = 33;

    /// The canonical encoding, FORMAT.md: [`Exact::WIRE_VERSION`], then the mantissa's low and
    /// high limbs, the exponent and the `[n, p, q]` counts, each little-endian. The body is the
    /// `repr(C)` bytes on a little-endian machine.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; Self::BYTES] {
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
