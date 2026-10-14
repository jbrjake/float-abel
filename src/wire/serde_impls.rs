//! `Serialize` and `Deserialize` as the wire bytes. `Exact` and `Wide` serialise as one byte
//! string, the bytes `to_bytes` gives handed to `serialize_bytes`, so a format frames them with its
//! own length (a `u64` on bincode 1, a varint on postcard, an array of numbers on `serde_json`) and
//! the serializer runs about once per value rather than once per byte, which is what an engine's
//! exchange pays per update, three passes over. `Fixed` serialises as a fixed-length tuple of its
//! bytes, one element each, because its length is `1 + 8 * LIMBS`, which stable Rust cannot spell
//! as an array type inside the impl, and a byte string needs one slice; a format that writes a
//! tuple's elements back to back emits its `to_bytes` byte for byte. Every decoder gathers the
//! bytes, from a byte string or from a sequence of numbers, and runs `from_bytes`, so canonical
//! form is validated in one place and a refusal is the decoder's message. Nothing here needs `std`
//! or an allocator.

use core::fmt;

use serde::de::{self, Deserialize, Deserializer, Expected, SeqAccess, Visitor};
use serde::ser::{Serialize, SerializeTuple, Serializer};

use crate::{Exact, Fixed, Wide};

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
    /// [`Exact::to_bytes`] as one byte string of 33 bytes.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(&self.to_bytes())
    }
}

struct ExactVisitor;

impl<'de> Visitor<'de> for ExactVisitor {
    type Value = Exact;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "the {}-byte wire encoding of an Exact", Exact::BYTES)
    }

    /// The byte string, whatever its length: [`Exact::from_bytes`] refuses the wrong one by name.
    fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<Exact, E> {
        Exact::from_bytes(v).map_err(de::Error::custom)
    }

    /// A human-readable format's array of numbers, one byte each.
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Exact, A::Error> {
        let mut buf = [0u8; Exact::BYTES];
        fill(&mut seq, &mut buf, 0, &self)?;
        Exact::from_bytes(&buf).map_err(de::Error::custom)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl<'de> Deserialize<'de> for Exact {
    /// The byte string through [`Exact::from_bytes`], so a non-canonical word is refused with
    /// the decoder's reason.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_bytes(ExactVisitor)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl Serialize for Wide {
    /// [`Wide::to_bytes`] as one byte string of 297 bytes.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(&self.to_bytes())
    }
}

struct WideVisitor;

impl<'de> Visitor<'de> for WideVisitor {
    type Value = Wide;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "the {}-byte wire encoding of a Wide", Wide::BYTES)
    }

    /// The byte string, whatever its length: [`Wide::from_bytes`] refuses the wrong one by name.
    fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<Wide, E> {
        Wide::from_bytes(v).map_err(de::Error::custom)
    }

    /// A human-readable format's array of numbers, one byte each.
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Wide, A::Error> {
        let mut buf = [0u8; Wide::BYTES];
        fill(&mut seq, &mut buf, 0, &self)?;
        Wide::from_bytes(&buf).map_err(de::Error::custom)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl<'de> Deserialize<'de> for Wide {
    /// The byte string through [`Wide::from_bytes`].
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_bytes(WideVisitor)
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
