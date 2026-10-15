//! The canonical wire format, FORMAT.md: one version byte, then the type's `repr(C)` body
//! little-endian. `to_bytes` and `from_bytes` on every exact type, [`WireError`] as the one
//! refusal, and the `serde` feature carrying the same bytes. This file holds the error, the byte
//! helpers and the [`Wide`] and [`Fixed`] codecs, which are total apart from the version byte and
//! the length; `Exact`'s codec lives beside its canonical form, in `exact::wire`.

use core::fmt;

use crate::fixed::Fixed;
use crate::wide::Wide;

#[cfg(feature = "serde")]
mod serde_impls;

/// Why a decoder refused bytes. [`Exact::from_bytes`](crate::Exact::from_bytes),
/// [`Wide::from_bytes`] and [`Fixed::from_bytes`] return it, and the `serde` deserialisers carry
/// its message, so a word that is not a value never becomes one. Every variant is a refusal of the
/// input; none is a panic.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WireError {
    /// The first byte is not the type's [`WIRE_VERSION`](crate::Exact::WIRE_VERSION), so the body
    /// is another type's, another revision's, or not this format at all.
    Version(u8),
    /// The byte count is not the type's [`BYTES`](crate::Exact::BYTES).
    Length {
        /// The type's encoded length.
        expected: usize,
        /// The length offered.
        found: usize,
    },
    /// `Exact`: a nonzero mantissa with its lowest bit clear. Canonical form moves trailing
    /// zeros into the exponent, so an even mantissa is a second spelling of a value.
    EvenMantissa,
    /// `Exact`: a zero mantissa with a nonzero exponent. The zero form is `(0, 0, 0)`.
    ZeroExponent,
    /// `Exact`: an exponent outside `-EXACT_MAX_EXP..=EXACT_MAX_EXP`.
    Exponent,
    /// `Exact`: a count equal to `i32::MIN`, which no operation produces and negation cannot take.
    Count,
    /// `Exact`, heap form: a run's limb count past `EXACT_MAX_LIMBS`, refused before anything is
    /// allocated, because an untrusted size gets a ceiling.
    Limbs,
    /// `Exact`, heap form: a body in a form no operation writes, so a second spelling of a value:
    /// a run that is not normalised (a leading zero limb, or a sign or an exponent on an empty
    /// run), or a value that fits the compact form and so belongs in its encoding.
    Form,
}

impl fmt::Display for WireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Version(found) => write!(f, "unknown version byte {found:#04x}"),
            Self::Length { expected, found } => {
                write!(f, "wrong length: {expected} bytes expected, {found} found")
            }
            Self::EvenMantissa => {
                f.write_str("even mantissa: trailing zeros belong in the exponent")
            }
            Self::ZeroExponent => f.write_str("a zero mantissa with a nonzero exponent"),
            Self::Exponent => f.write_str("the exponent lies outside ±EXACT_MAX_EXP"),
            Self::Count => f.write_str("a count of i32::MIN, which negation cannot take"),
            Self::Limbs => f.write_str("a heap run's limb count lies past EXACT_MAX_LIMBS"),
            Self::Form => f.write_str(
                "a heap body that is not normalised, or holds a value that fits the compact form",
            ),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for WireError {}

/// Refuses a first byte that is not the type's.
pub(crate) fn check_version(found: u8, expected: u8) -> Result<(), WireError> {
    if found == expected {
        Ok(())
    } else {
        Err(WireError::Version(found))
    }
}

/// The body behind the version byte, once the length and then the byte are the type's.
pub(crate) fn body(bytes: &[u8], version: u8, len: usize) -> Result<&[u8], WireError> {
    if bytes.len() != len {
        return Err(WireError::Length {
            expected: len,
            found: bytes.len(),
        });
    }
    check_version(bytes[0], version)?;
    Ok(&bytes[1..])
}

/// Limbs to little-endian bytes, eight per limb, into a slice of exactly that length.
pub(crate) fn put_u64s(limbs: &[u64], out: &mut [u8]) {
    debug_assert_eq!(out.len(), 8 * limbs.len());
    for (limb, chunk) in limbs.iter().zip(out.chunks_exact_mut(8)) {
        chunk.copy_from_slice(&limb.to_le_bytes());
    }
}

/// Little-endian bytes to limbs, eight per limb, from a slice of exactly that length.
pub(crate) fn get_u64s(bytes: &[u8], limbs: &mut [u64]) {
    debug_assert_eq!(bytes.len(), 8 * limbs.len());
    for (limb, chunk) in limbs.iter_mut().zip(bytes.chunks_exact(8)) {
        *limb = u64_le(chunk);
    }
}

/// The little-endian `u64` in an 8-byte slice.
pub(crate) fn u64_le(chunk: &[u8]) -> u64 {
    let mut raw = [0u8; 8];
    raw.copy_from_slice(chunk);
    u64::from_le_bytes(raw)
}

/// The little-endian `i32` in a 4-byte slice.
pub(crate) fn i32_le(chunk: &[u8]) -> i32 {
    let mut raw = [0u8; 4];
    raw.copy_from_slice(chunk);
    i32::from_le_bytes(raw)
}

impl Wide {
    /// The first byte of every encoded `Wide`: this type, this revision of its encoding.
    pub const WIRE_VERSION: u8 = 0x02;
    /// The length of [`Wide::to_bytes`]: the version byte, 34 limbs and 3 counters.
    pub const BYTES: usize = 1 + 8 * (Self::LIMBS + 3);

    /// The canonical encoding, FORMAT.md: [`Wide::WIRE_VERSION`], then the limbs least
    /// significant first and the `[n, p, q]` counters, each little-endian. The body is the
    /// `repr(C)` bytes on a little-endian machine.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; Self::BYTES] {
        let mut out = [0u8; Self::BYTES];
        out[0] = Self::WIRE_VERSION;
        let limb_bytes = 8 * Self::LIMBS;
        put_u64s(self.limbs(), &mut out[1..=limb_bytes]);
        let (n, p, q) = self.specials();
        for (chunk, count) in out[1 + limb_bytes..].chunks_exact_mut(8).zip([n, p, q]) {
            chunk.copy_from_slice(&count.to_le_bytes());
        }
        out
    }

    /// The value [`Wide::to_bytes`] encoded. Every body is a value, so the only refusals are the
    /// framing's.
    ///
    /// # Errors
    ///
    /// [`WireError::Length`] unless `bytes` is exactly [`Wide::BYTES`] long, then
    /// [`WireError::Version`] unless its first byte is [`Wide::WIRE_VERSION`].
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, WireError> {
        let body = body(bytes, Self::WIRE_VERSION, Self::BYTES)?;
        let (limb_bytes, counter_bytes) = body.split_at(8 * Self::LIMBS);
        let mut limbs = [0u64; Self::LIMBS];
        get_u64s(limb_bytes, &mut limbs);
        let mut specials = [0i64; 3];
        for (count, chunk) in specials.iter_mut().zip(counter_bytes.chunks_exact(8)) {
            *count = u64_le(chunk) as i64;
        }
        Ok(Self::from_parts(limbs, specials))
    }
}

/// Pins `N`, the array [`Fixed::to_bytes`] returns, to the type's `BYTES`, so a buffer can never
/// be the wrong size at run time.
struct Encoded<const LIMBS: usize, const N: usize>;

impl<const LIMBS: usize, const N: usize> Encoded<LIMBS, N> {
    const OK: () = assert!(
        N == 1 + 8 * LIMBS,
        "Fixed: to_bytes returns [u8; BYTES], the version byte then eight bytes per limb"
    );
}

impl<const LSB: i32, const LIMBS: usize> Fixed<LSB, LIMBS> {
    /// The first byte of every encoded `Fixed`: this type, this revision of its encoding. The
    /// grid and the width are the type's, not the encoding's, so every `Fixed` shares the byte
    /// and a wrong width is a [`WireError::Length`].
    pub const WIRE_VERSION: u8 = 0x03;
    /// The length of [`Fixed::to_bytes`]: the version byte and the limbs.
    pub const BYTES: usize = 1 + 8 * LIMBS;

    /// The canonical encoding, FORMAT.md: [`Fixed::WIRE_VERSION`], then the limbs least
    /// significant first, each little-endian. The body is the `repr(C)` bytes on a little-endian
    /// machine.
    ///
    /// `N` is [`Fixed::BYTES`], pinned by a `const` assertion rather than inferred, because
    /// stable Rust cannot spell `8 * LIMBS + 1` in a return type; write the binding's type and
    /// the compiler fills it in: `let bytes: [u8; Fixed256::BYTES] = f.to_bytes();`. Any other
    /// length does not build.
    #[must_use]
    pub fn to_bytes<const N: usize>(&self) -> [u8; N] {
        #[allow(clippy::let_unit_value)] // evaluates the const assertion on the length
        let () = Encoded::<LIMBS, N>::OK;
        let mut out = [0u8; N];
        out[0] = Self::WIRE_VERSION;
        put_u64s(self.limbs(), &mut out[1..]);
        out
    }

    /// The value [`Fixed::to_bytes`] encoded. Every body is a value, so the only refusals are the
    /// framing's.
    ///
    /// # Errors
    ///
    /// [`WireError::Length`] unless `bytes` is exactly [`Fixed::BYTES`] long, then
    /// [`WireError::Version`] unless its first byte is [`Fixed::WIRE_VERSION`].
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, WireError> {
        let body = body(bytes, Self::WIRE_VERSION, Self::BYTES)?;
        let mut limbs = [0u64; LIMBS];
        get_u64s(body, &mut limbs);
        Ok(Self::from_limbs(limbs))
    }
}
