//! Where the serialized exchange's cost is, BENCH.md § The wire form. timely's `ProcessBinary`
//! exchange encodes every batch of updates through its `Bytesable` impl on `Message`: bincode 1
//! sizes the batch in one pass, writes it in a second into the reserved slab, and the receiver
//! decodes it in a third, so a diff's `Serialize` runs twice and its `Deserialize` once per
//! crossing. This target runs that path alone, each pass its own row, on a batch of
//! `((u32, ()), u64, R)` updates, the tuple `count_total`'s `arrange_by_self` exchanges in
//! `benches/dd_sum.rs`, at the batch size timely's exchange buffers hold, so a row is the wire
//! form's share of the end-to-end gap and nothing else's. Four diffs: `isize`, the engine's own;
//! `Exact` as the crate serialises it, one byte string behind bincode's 8-byte length; and two
//! other forms built here as newtypes over its bytes, a 33-element tuple of `u8`, as a `[u8; 33]`
//! serialises, and a 7-element tuple of the fields, which bincode's fixed-int encoding frames
//! byte for byte as the 33-tuple. Each form round-trips against the input before anything is
//! timed. Advisory, never a gate: `cargo bench --bench wire`, and the numbers go into BENCH.md
//! with the machine and the commit.

// The `isize` diff is the fixture's count of units, cast on purpose as the SUM casts it.
#![allow(clippy::cast_possible_truncation)]

use core::fmt;
use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use float_abel::Exact;
use serde::de::{self, Deserialize, Deserializer, Expected, SeqAccess, Visitor};
use serde::ser::{Serialize, SerializeTuple, Serializer};
use timely::bytes::arc::{Bytes, BytesMut};
use timely::communication::Bytesable;
use timely::container::buffer::default_capacity;
use timely::dataflow::channels::Message;

mod support;
use support::{amount, units};

/// One exchanged update: the key and its unit value, the epoch, the diff.
type Update<R> = ((u32, ()), u64, R);
/// A batch as the exchange channel frames it: the message's time, then the updates.
type Batch<R> = Message<u64, Vec<Update<R>>>;

/// The next `u8` element of a sequence, or `invalid_length` at `at` when the sequence ends.
fn next_byte<'de, A: SeqAccess<'de>>(
    seq: &mut A,
    at: usize,
    expecting: &dyn Expected,
) -> Result<u8, A::Error> {
    seq.next_element()?
        .ok_or_else(|| de::Error::invalid_length(at, expecting))
}

/// `Exact` as a 33-element tuple of `u8`, one `serialize_element` call per byte, as a `[u8; 33]`
/// serialises: 33 bytes on bincode with no length, 33 element calls each way.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct AsTuple(Exact);

impl Serialize for AsTuple {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut tuple = serializer.serialize_tuple(Exact::BYTES)?;
        for b in self.0.to_bytes() {
            tuple.serialize_element(&b)?;
        }
        tuple.end()
    }
}

struct TupleVisitor;

impl<'de> Visitor<'de> for TupleVisitor {
    type Value = AsTuple;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("the 33 wire bytes of an Exact, one element each")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<AsTuple, A::Error> {
        let mut b = [0u8; Exact::BYTES];
        for (at, byte) in b.iter_mut().enumerate() {
            *byte = next_byte(&mut seq, at, &self)?;
        }
        Exact::from_bytes(&b)
            .map(AsTuple)
            .map_err(de::Error::custom)
    }
}

impl<'de> Deserialize<'de> for AsTuple {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_tuple(Exact::BYTES, TupleVisitor)
    }
}

/// `Exact` as a 7-element tuple of its fields, `(u8, u64, u64, i32, i32, i32, i32)`: the version
/// byte, the mantissa's limbs, the exponent and the three counts, each read off `to_bytes`. On
/// bincode 1's fixed-int encoding that is the 33-tuple's bytes exactly, in seven element calls
/// instead of 33; on a varint format such as postcard it is not.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct AsFields(Exact);

/// The little-endian `u64` at `at` in a wire encoding.
fn u64_at(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(bytes[at..at + 8].try_into().expect("eight bytes"))
}

/// The little-endian `i32` at `at` in a wire encoding.
fn i32_at(bytes: &[u8], at: usize) -> i32 {
    i32::from_le_bytes(bytes[at..at + 4].try_into().expect("four bytes"))
}

impl Serialize for AsFields {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let b = self.0.to_bytes();
        let mut tuple = serializer.serialize_tuple(7)?;
        tuple.serialize_element(&b[0])?;
        tuple.serialize_element(&u64_at(&b, 1))?;
        tuple.serialize_element(&u64_at(&b, 9))?;
        tuple.serialize_element(&i32_at(&b, 17))?;
        tuple.serialize_element(&i32_at(&b, 21))?;
        tuple.serialize_element(&i32_at(&b, 25))?;
        tuple.serialize_element(&i32_at(&b, 29))?;
        tuple.end()
    }
}

struct FieldsVisitor;

impl<'de> Visitor<'de> for FieldsVisitor {
    type Value = AsFields;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("the seven fields of an Exact's wire encoding")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<AsFields, A::Error> {
        let mut b = [0u8; Exact::BYTES];
        b[0] = next_byte(&mut seq, 0, &self)?;
        let mut at = 1;
        for _ in 0..2 {
            let limb: u64 = seq
                .next_element()?
                .ok_or_else(|| de::Error::invalid_length(at, &self))?;
            b[at..at + 8].copy_from_slice(&limb.to_le_bytes());
            at += 8;
        }
        for _ in 0..4 {
            let field: i32 = seq
                .next_element()?
                .ok_or_else(|| de::Error::invalid_length(at, &self))?;
            b[at..at + 4].copy_from_slice(&field.to_le_bytes());
            at += 4;
        }
        Exact::from_bytes(&b)
            .map(AsFields)
            .map_err(de::Error::custom)
    }
}

impl<'de> Deserialize<'de> for AsFields {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_tuple(7, FieldsVisitor)
    }
}

/// A batch of the fixture's rows at timely's buffer capacity for this update, 8 KiB of elements,
/// every update in epoch 0 as the SUM's one input epoch is.
fn batch<R>(diff: impl Fn(i64) -> R) -> Batch<R> {
    let n = default_capacity::<Update<R>>();
    let data = units(n as u64)
        .into_iter()
        .map(|(account, units)| ((account, ()), 0, diff(units)))
        .collect();
    Message::new(0, data)
}

/// The batch as the exchange writes it: sized, then written into a slab of exactly that length.
fn encode<R>(m: &Batch<R>) -> Vec<u8>
where
    Batch<R>: Bytesable,
{
    let mut buf = vec![0u8; m.length_in_bytes()];
    let mut writer = &mut buf[..];
    m.into_bytes(&mut writer);
    assert!(writer.is_empty(), "the slab is filled exactly");
    buf
}

/// The three passes of one crossing, each its own row at the batch's element count: `size` is
/// `length_in_bytes`, the serializer run to count; `write` is `into_bytes` into a slab;
/// `read` is `from_bytes` on the received bytes. Before timing, the batch round-trips.
fn passes<R>(c: &mut Criterion, form: &str, m: &Batch<R>)
where
    R: PartialEq + fmt::Debug,
    Batch<R>: Bytesable,
{
    let encoded = encode(m);
    let received = BytesMut::from(encoded.clone()).freeze();
    let back = Batch::<R>::from_bytes(received.clone());
    assert_eq!(back.data.len(), m.data.len());
    assert_eq!(back.data, m.data, "{form}: the batch round-trips");
    assert_eq!(back.time, m.time);

    let mut g = c.benchmark_group("wire");
    g.throughput(Throughput::Elements(m.data.len() as u64));
    g.bench_function(format!("{form}/size"), |b| {
        b.iter(|| black_box(m).length_in_bytes());
    });
    let mut slab = vec![0u8; encoded.len()];
    g.bench_function(format!("{form}/write"), |b| {
        b.iter(|| {
            let mut writer = &mut slab[..];
            black_box(m).into_bytes(&mut writer);
        });
    });
    g.bench_function(format!("{form}/read"), |b| {
        b.iter(|| {
            let bytes: Bytes = received.clone();
            black_box(Batch::<R>::from_bytes(black_box(bytes)))
        });
    });
    g.finish();
}

fn wire(c: &mut Criterion) {
    let exact = batch(|units| Exact::from(amount(units)));
    let tuple = batch(|units| AsTuple(Exact::from(amount(units))));
    let fields = batch(|units| AsFields(Exact::from(amount(units))));
    // The framings, checked once: the fields form is the tuple form byte for byte on bincode, and
    // the crate's byte string carries an 8-byte length per update over the tuple, each batch
    // padded to a multiple of eight on its own.
    let tuple_bytes = encode(&tuple);
    assert_eq!(encode(&fields), tuple_bytes);
    let n = exact.data.len();
    assert_eq!(tuple.data.len(), n);
    assert_eq!(fields.data.len(), n);
    let prefixes = encode(&exact).len() - tuple_bytes.len();
    assert!(
        prefixes.abs_diff(8 * n) < 8,
        "{prefixes} bytes of prefix for {n} updates"
    );

    passes(c, "isize", &batch(|units| units as isize));
    passes(c, "exact", &exact);
    passes(c, "tuple", &tuple);
    passes(c, "fields", &fields);
}

criterion_group!(benches, wire);
criterion_main!(benches);
