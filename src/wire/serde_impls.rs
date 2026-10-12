//! `Serialize` and `Deserialize` as the wire bytes. A value is a fixed-length tuple of the bytes
//! `to_bytes` gives, which is how a `[u8; N]` serialises, so a format frames it as it frames an
//! array: one that writes a tuple's elements back to back, postcard or bincode 1's fixed-int
//! encoding, emits `to_bytes` byte for byte with no length prefix, and `serde_json` carries it as
//! an array of numbers. A decoder gathers the bytes and runs `from_bytes`, so canonical form is
//! validated in one place and a refusal is the decoder's message. Nothing here needs `std` or an
//! allocator.

use core::fmt;

use serde::de::{self, Deserialize, Deserializer, Expected, SeqAccess, Visitor};
use serde::ser::{Serialize, SerializeTuple, Serializer};

use crate::{Exact, Fixed, Wide};

/// The bytes as a tuple of `u8`, one element each.
fn serialize_bytes<S: Serializer>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
    let mut tuple = serializer.serialize_tuple(bytes.len())?;
    for b in bytes {
        tuple.serialize_element(b)?;
    }
    tuple.end()
}

/// Fills `buf` from the sequence, one byte per element, where `buf` starts at element `from`.
/// A sequence that ends first is `invalid_length` at the element it lacks.
fn fill<'de, A: SeqAccess<'de>>(
    seq: &mut A,
    buf: &mut [u8],
    from: usize,
    expecting: &dyn Expected,
) -> Result<(), A::Error> {
    for (i, b) in buf.iter_mut().enumerate() {
        *b = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(from + i, expecting))?;
    }
    Ok(())
}

#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl Serialize for Exact {
    /// [`Exact::to_bytes`] as a 33-element tuple of `u8`.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize_bytes(&self.to_bytes(), serializer)
    }
}

struct ExactVisitor;

impl<'de> Visitor<'de> for ExactVisitor {
    type Value = Exact;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "the {}-byte wire encoding of an Exact", Exact::BYTES)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Exact, A::Error> {
        let mut buf = [0u8; Exact::BYTES];
        fill(&mut seq, &mut buf, 0, &self)?;
        Exact::from_bytes(&buf).map_err(de::Error::custom)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl<'de> Deserialize<'de> for Exact {
    /// The 33-element tuple through [`Exact::from_bytes`], so a non-canonical word is refused
    /// with the decoder's reason.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_tuple(Exact::BYTES, ExactVisitor)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl Serialize for Wide {
    /// [`Wide::to_bytes`] as a 297-element tuple of `u8`.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize_bytes(&self.to_bytes(), serializer)
    }
}

struct WideVisitor;

impl<'de> Visitor<'de> for WideVisitor {
    type Value = Wide;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "the {}-byte wire encoding of a Wide", Wide::BYTES)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Wide, A::Error> {
        let mut buf = [0u8; Wide::BYTES];
        fill(&mut seq, &mut buf, 0, &self)?;
        Wide::from_bytes(&buf).map_err(de::Error::custom)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl<'de> Deserialize<'de> for Wide {
    /// The 297-element tuple through [`Wide::from_bytes`].
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_tuple(Wide::BYTES, WideVisitor)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl<const LSB: i32, const LIMBS: usize> Serialize for Fixed<LSB, LIMBS> {
    /// [`Fixed::to_bytes`] as a `BYTES`-element tuple of `u8`: the version byte, then eight per
    /// limb. Streamed limb by limb, because the array's length cannot be spelled here.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut tuple = serializer.serialize_tuple(Self::BYTES)?;
        tuple.serialize_element(&Self::WIRE_VERSION)?;
        for limb in self.limbs() {
            for b in limb.to_le_bytes() {
                tuple.serialize_element(&b)?;
            }
        }
        tuple.end()
    }
}

struct FixedVisitor<const LSB: i32, const LIMBS: usize>;

impl<'de, const LSB: i32, const LIMBS: usize> Visitor<'de> for FixedVisitor<LSB, LIMBS> {
    type Value = Fixed<LSB, LIMBS>;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "the {}-byte wire encoding of a Fixed<{LSB}, {LIMBS}>",
            Fixed::<LSB, LIMBS>::BYTES
        )
    }

    /// The version byte through the same check as [`Fixed::from_bytes`], then the limbs, each
    /// gathered from eight elements; the tuple's arity is the length check.
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut version = [0u8; 1];
        fill(&mut seq, &mut version, 0, &self)?;
        super::check_version(version[0], Fixed::<LSB, LIMBS>::WIRE_VERSION)
            .map_err(de::Error::custom)?;
        let mut limbs = [0u64; LIMBS];
        let mut chunk = [0u8; 8];
        for (i, limb) in limbs.iter_mut().enumerate() {
            fill(&mut seq, &mut chunk, 1 + 8 * i, &self)?;
            *limb = u64::from_le_bytes(chunk);
        }
        Ok(Fixed::from_limbs(limbs))
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl<'de, const LSB: i32, const LIMBS: usize> Deserialize<'de> for Fixed<LSB, LIMBS> {
    /// The `BYTES`-element tuple, validated as [`Fixed::from_bytes`] validates it.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_tuple(Self::BYTES, FixedVisitor::<LSB, LIMBS>)
    }
}
