//! `Exact`'s codec, FORMAT.md. The compact form's body is the six fields little-endian, and the
//! decoder is where canonical form is checked, because a word with an even mantissa is not a
//! value and structural `Eq` must stay numeric equality. The checks run in one order, mantissa
//! then exponent then counts, so the first failing one names the error.
//!
//! Behind `alloc` the heap form has a body of its own: [`Exact::HEAP_TAG`], then the block as the
//! word holds it, four packed header limbs and the four runs of limbs. `encode` writes either
//! form, `from_bytes` reads either, and the heap decoder validates the body over the bytes, the
//! limb counts against the cap first, before it allocates the block, and refuses a body that is
//! not normalised or whose value fits the compact form, so one value still has one encoding.

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "alloc")]
use super::heap::{self, HEADER};
#[cfg(feature = "alloc")]
use super::EXACT_MAX_LIMBS;
use super::{Exact, EXACT_MAX_EXP};
use crate::wire::{self, WireError};

impl Exact {
    /// The first byte of every encoded compact `Exact`: this type, this form, this revision of
    /// its encoding.
    pub const WIRE_VERSION: u8 = 0x01;
    /// The length of [`Exact::to_bytes`]: the version byte and the 32-byte word.
    pub const BYTES: usize = 33;
    /// The first byte of every encoded heap `Exact`, which [`Exact::encode`] writes behind the
    /// `alloc` feature and [`Exact::from_bytes`] reads there; without `alloc` no value is in the
    /// heap form and the byte is refused as a [`WireError::Version`].
    pub const HEAP_TAG: u8 = 0x04;

    /// The canonical encoding of the compact form, FORMAT.md: [`Exact::WIRE_VERSION`], then the
    /// mantissa's low and high limbs, the exponent and the `[n, p, q]` counts, each
    /// little-endian. The body is the `repr(C)` bytes on a little-endian machine.
    ///
    /// # Panics
    ///
    /// On a value in the heap form, which has no 33-byte encoding; [`Exact::is_compact`] says
    /// which form a value is in, and [`Exact::encode`] writes either.
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

    /// The value [`Exact::to_bytes`] or [`Exact::encode`] encoded, if the body is a canonical
    /// form: a byte pattern with an even mantissa, or a heap body of a value the compact form
    /// holds, is a second spelling of a value, and a decoder that accepted one would break
    /// `Eq`-substitutability in an engine's consolidation. The first byte picks the form.
    ///
    /// # Errors
    ///
    /// On a compact body, [`WireError::Length`] unless `bytes` is exactly [`Exact::BYTES`] long,
    /// then [`WireError::Version`] unless its first byte is [`Exact::WIRE_VERSION`]. Then, on the
    /// body: [`WireError::EvenMantissa`] for a nonzero mantissa with its lowest bit clear,
    /// [`WireError::ZeroExponent`] for a zero mantissa with a nonzero exponent,
    /// [`WireError::Exponent`] for an exponent outside `±EXACT_MAX_EXP`, and [`WireError::Count`]
    /// for a count of `i32::MIN`. Every odd mantissa is inside the width, so there is no width
    /// refusal.
    ///
    /// On a heap body, behind `alloc`: [`WireError::Length`] under the tag and the header,
    /// [`WireError::Limbs`] for a run's limb count past [`EXACT_MAX_LIMBS`], refused before
    /// anything is allocated, [`WireError::Length`] unless the body is the length the header
    /// implies, then on each run in order [`WireError::Form`] for a sign or an exponent on an
    /// empty run, [`WireError::EvenMantissa`] for an even first limb, [`WireError::Form`] for a
    /// zero last limb and [`WireError::Exponent`] for the bound, and last [`WireError::Form`] for
    /// a value that fits the compact form.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, WireError> {
        #[cfg(feature = "alloc")]
        if let Some((&Self::HEAP_TAG, body)) = bytes.split_first() {
            return Self::decode_heap(body);
        }
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

    /// The word with these fields, if it is a canonical compact form: the one constructor every
    /// decoder of the compact form goes through, the DBSP adapter's archived form and the
    /// `Arbitrary` draw included.
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

/// The shortest heap encoding: the tag and the header, every run empty.
#[cfg(feature = "alloc")]
const LEAST: usize = 1 + 8 * HEADER;

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl Exact {
    /// Either form's canonical encoding, FORMAT.md: [`Exact::to_bytes`] for the compact form; for
    /// the heap form [`Exact::HEAP_TAG`], then the block as the word holds it, four header limbs
    /// (the sign in the top bit, the limb count in the next 31, the exponent as an `i32` in the
    /// low 32) for the finite part and the NaN, +∞ and −∞ amounts, then the four runs of limbs,
    /// each least significant first, every limb little-endian. One allocation, the `Vec`.
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        if self.is_compact() {
            return self.to_bytes().to_vec();
        }
        let used = self.parts().used();
        let mut out = alloc::vec![0u8; 1 + 8 * used.len()];
        out[0] = Self::HEAP_TAG;
        wire::put_u64s(used, &mut out[1..]);
        out
    }

    /// A heap body, the bytes behind the tag, validated over the bytes before the block is
    /// allocated: the header's counts against the cap, the length they imply, then
    /// [`heap::check_runs`]; a body that passes becomes the block of a word through the one gate
    /// every block from outside passes.
    fn decode_heap(body: &[u8]) -> Result<Self, WireError> {
        let found = 1 + body.len();
        if body.len() < 8 * HEADER {
            return Err(WireError::Length {
                expected: LEAST,
                found,
            });
        }
        let mut header = [0u64; HEADER];
        wire::get_u64s(&body[..8 * HEADER], &mut header);
        let mut limbs = 0usize;
        for &e in &header {
            let len = heap::unpack(e).2;
            if len > EXACT_MAX_LIMBS {
                return Err(WireError::Limbs);
            }
            limbs += len;
        }
        let expected = LEAST + 8 * limbs;
        if found != expected {
            return Err(WireError::Length { expected, found });
        }
        let runs = &body[8 * HEADER..];
        heap::check_runs(&header, limbs, |i| wire::u64_le(&runs[8 * i..8 * i + 8]))?;
        let mut block = alloc::vec![0u64; HEADER + limbs];
        wire::get_u64s(body, &mut block);
        Self::from_canonical_block(block)
    }
}
