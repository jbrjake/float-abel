//! Probe of dbsp 0.364.0's weight contract on rustc 1.99.0, with a word shaped like `Exact`.
//!
//! `Word` is `Exact`'s six fields with rkyv's derive on them, converted through the wire bytes.
//! `W` is the weight with every trait the design lists; `NoNone`, `NoOrdRepr` and `NoRing` each
//! drop one group and are compiled only under a `break_*` feature, where the build must fail on
//! the predicted bound.
#![deny(unsafe_code)]

use core::cmp::Ordering;
use core::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

use dbsp::algebra::{HasOne, HasZero, MulByRef};
use dbsp::dynamic::OrdRepr;
use float_abel::Exact;
use rkyv::{Archive, Deserialize, Serialize};
use size_of::{Context, SizeOf};

/// `Exact`'s layout, as the integers rkyv's derive archives as themselves.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Archive, Serialize, Deserialize)]
#[repr(C)]
pub struct Word {
    lo: u64,
    hi: u64,
    exp: i32,
    nan: i32,
    pos_inf: i32,
    neg_inf: i32,
}

fn u64_at(b: &[u8], i: usize) -> u64 {
    u64::from_le_bytes(b[i..i + 8].try_into().unwrap())
}

fn i32_at(b: &[u8], i: usize) -> i32 {
    i32::from_le_bytes(b[i..i + 4].try_into().unwrap())
}

impl Word {
    pub fn from_exact(x: Exact) -> Self {
        let b = x.to_bytes();
        Self {
            lo: u64_at(&b, 1),
            hi: u64_at(&b, 9),
            exp: i32_at(&b, 17),
            nan: i32_at(&b, 21),
            pos_inf: i32_at(&b, 25),
            neg_inf: i32_at(&b, 29),
        }
    }

    pub fn exact(&self) -> Exact {
        let mut b = [0u8; Exact::BYTES];
        b[0] = Exact::WIRE_VERSION;
        b[1..9].copy_from_slice(&self.lo.to_le_bytes());
        b[9..17].copy_from_slice(&self.hi.to_le_bytes());
        b[17..21].copy_from_slice(&self.exp.to_le_bytes());
        b[21..25].copy_from_slice(&self.nan.to_le_bytes());
        b[25..29].copy_from_slice(&self.pos_inf.to_le_bytes());
        b[29..33].copy_from_slice(&self.neg_inf.to_le_bytes());
        Exact::from_bytes(&b).expect("a Word is a canonical Exact")
    }
}

impl ArchivedWord {
    pub fn to_word(&self) -> Word {
        Word {
            lo: self.lo,
            hi: self.hi,
            exp: self.exp,
            nan: self.nan,
            pos_inf: self.pos_inf,
            neg_inf: self.neg_inf,
        }
    }
}

/// The group, the monoid, the integer scaling, the storage traits and the ordering: everything
/// but the three groups a break removes.
macro_rules! group_weight {
    ($name:ident) => {
        #[derive(
            Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
        )]
        #[repr(transparent)]
        pub struct $name(pub Word);

        impl $name {
            pub fn from_exact(x: Exact) -> Self {
                Self(Word::from_exact(x))
            }
            pub fn exact(&self) -> Exact {
                self.0.exact()
            }
        }

        impl PartialOrd for $name {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }
        impl Ord for $name {
            fn cmp(&self, other: &Self) -> Ordering {
                self.exact().cmp(&other.exact())
            }
        }

        impl SizeOf for $name {
            fn size_of_children(&self, _: &mut Context) {}
        }

        impl Add<&$name> for &$name {
            type Output = $name;
            fn add(self, rhs: &$name) -> $name {
                $name::from_exact(self.exact() + rhs.exact())
            }
        }
        impl Add for $name {
            type Output = Self;
            fn add(self, rhs: Self) -> Self {
                &self + &rhs
            }
        }
        impl AddAssign<&$name> for $name {
            fn add_assign(&mut self, rhs: &$name) {
                *self = &*self + rhs;
            }
        }
        impl AddAssign for $name {
            fn add_assign(&mut self, rhs: Self) {
                *self += &rhs;
            }
        }
        impl Sub<&$name> for &$name {
            type Output = $name;
            fn sub(self, rhs: &$name) -> $name {
                $name::from_exact(self.exact() - rhs.exact())
            }
        }
        impl Sub for $name {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self {
                &self - &rhs
            }
        }
        impl SubAssign<&$name> for $name {
            fn sub_assign(&mut self, rhs: &$name) {
                *self = &*self - rhs;
            }
        }
        impl SubAssign for $name {
            fn sub_assign(&mut self, rhs: Self) {
                *self -= &rhs;
            }
        }

        impl HasZero for $name {
            fn is_zero(&self) -> bool {
                self.exact().is_zero()
            }
            fn zero() -> Self {
                Self::default()
            }
        }

        impl MulByRef<i64> for $name {
            type Output = Self;
            fn mul_by_ref(&self, k: &i64) -> Self {
                Self::from_exact(self.exact() * *k)
            }
        }
        impl MulByRef<isize> for $name {
            type Output = Self;
            fn mul_by_ref(&self, k: &isize) -> Self {
                Self::from_exact(self.exact() * *k)
            }
        }

        paste_archived!($name);
    };
}

/// `PartialEq`/`Eq`/`PartialOrd`/`Ord` on the archived form, by the same numeric compare.
macro_rules! paste_archived {
    ($name:ident) => {
        impl PartialEq for <$name as Archive>::Archived {
            fn eq(&self, other: &Self) -> bool {
                self.0.to_word() == other.0.to_word()
            }
        }
        impl Eq for <$name as Archive>::Archived {}
        impl PartialOrd for <$name as Archive>::Archived {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }
        impl Ord for <$name as Archive>::Archived {
            fn cmp(&self, other: &Self) -> Ordering {
                self.0.to_word().exact().cmp(&other.0.to_word().exact())
            }
        }
    };
}

macro_rules! ord_repr_weight {
    ($name:ident) => {
        impl OrdRepr<$name> for <$name as Archive>::Archived {
            fn ord_cmp(&self, other: &$name) -> Ordering {
                self.0.to_word().exact().cmp(&other.exact())
            }
        }
    };
}

macro_rules! ring_weight {
    ($name:ident) => {
        impl Neg for &$name {
            type Output = $name;
            fn neg(self) -> $name {
                $name::from_exact(-self.exact())
            }
        }
        impl Neg for $name {
            type Output = Self;
            fn neg(self) -> Self {
                -&self
            }
        }
        impl Mul<&$name> for &$name {
            type Output = $name;
            fn mul(self, rhs: &$name) -> $name {
                $name::from_exact(self.exact() * rhs.exact())
            }
        }
        impl Mul for $name {
            type Output = Self;
            fn mul(self, rhs: Self) -> Self {
                &self * &rhs
            }
        }
        impl HasOne for $name {
            fn one() -> Self {
                Self::from_exact(Exact::from(1.0))
            }
        }
    };
}

/// The weight with everything.
group_weight!(W);
ord_repr_weight!(W);
ring_weight!(W);
dbsp::never_none!(W);
dbsp::never_roaring_filter!(W);

#[cfg(feature = "break_none")]
pub mod break_none {
    use super::*;
    group_weight!(NoNone);
    ord_repr_weight!(NoNone);
    ring_weight!(NoNone);

    fn needs_weight<T: dbsp::DBWeight>() {}
    pub fn bound() {
        needs_weight::<NoNone>();
    }
}

#[cfg(feature = "break_ordrepr")]
pub mod break_ordrepr {
    use super::*;
    group_weight!(NoOrdRepr);
    ring_weight!(NoOrdRepr);
    dbsp::never_none!(NoOrdRepr);
    dbsp::never_roaring_filter!(NoOrdRepr);

    fn needs_weight<T: dbsp::DBWeight>() {}
    pub fn bound() {
        needs_weight::<NoOrdRepr>();
    }
}

#[cfg(feature = "break_ring")]
pub mod break_ring {
    use super::*;
    use dbsp::dynamic::DynWeightTyped;
    use dbsp::{OrdIndexedWSet, RootCircuit};
    group_weight!(NoRing);
    ord_repr_weight!(NoRing);
    dbsp::never_none!(NoRing);
    dbsp::never_roaring_filter!(NoRing);

    pub fn bound() {
        let _ = RootCircuit::build(|circuit| {
            let (stream, _handle) = circuit
                .add_input_stream::<OrdIndexedWSet<u32, u32, NoRing, DynWeightTyped<NoRing>>>();
            let _negated = stream.neg();
            Ok(())
        });
    }
}

#[cfg(test)]
#[allow(unsafe_code)]
mod tests {
    use super::*;
    use dbsp::dynamic::{DowncastTrait, DynWeightTyped};
    use dbsp::trace::{BatchReader as _, Cursor as _};
    use dbsp::typed_batch::BatchReader as _;
    use dbsp::utils::Tup2;
    use dbsp::{IndexedZSetReader, OrdIndexedWSet, Runtime};

    type Batch = OrdIndexedWSet<u32, u32, W, DynWeightTyped<W>>;

    fn e(x: f64) -> Exact {
        Exact::from(x)
    }

    fn w(x: f64) -> W {
        W::from_exact(e(x))
    }

    fn tuples(b: &Batch) -> Vec<(u32, u32, Exact)> {
        let mut out = Vec::new();
        let mut c = b.inner().cursor();
        while c.key_valid() {
            while c.val_valid() {
                let k = *c.key().downcast_checked::<u32>();
                let v = *c.val().downcast_checked::<u32>();
                let r: W = **c.weight();
                out.push((k, v, r.exact()));
                c.step_val();
            }
            c.step_key();
        }
        out
    }

    #[test]
    fn word_round_trips_through_exact() {
        for x in [0.0, 0.1, -0.2, 1e300, 5e-324, f64::NAN, f64::INFINITY] {
            let word = Word::from_exact(e(x));
            assert_eq!(word.exact(), e(x));
        }
    }

    #[test]
    fn archived_form_agrees_with_ord() {
        let values: Vec<W> = [0.0, 0.1, -0.2, 0.3, 1e300, -1e300, f64::NAN, f64::INFINITY, f64::NEG_INFINITY]
            .into_iter()
            .map(w)
            .collect();
        for a in &values {
            let bytes = rkyv::to_bytes::<_, 64>(a).unwrap();
            let archived = unsafe { rkyv::archived_root::<W>(&bytes) };
            for b in &values {
                assert_eq!(archived.ord_cmp(b), a.cmp(b), "{a:?} vs {b:?}");
                let bytes_b = rkyv::to_bytes::<_, 64>(b).unwrap();
                let archived_b = unsafe { rkyv::archived_root::<W>(&bytes_b) };
                assert_eq!(archived.cmp(archived_b), a.cmp(b));
            }
        }
    }

    #[test]
    fn consolidation_adds_exactly_and_cancels_on_merge() {
        let b = Batch::from_tuples(
            (),
            vec![Tup2(Tup2(1, 10), w(0.1)), Tup2(Tup2(1, 10), w(0.2))],
        );
        assert_eq!(tuples(&b), vec![(1, 10, e(0.1) + e(0.2))]);
        assert_ne!(0.1 + 0.2 - 0.2 - 0.1, 0.0, "the f64 control must leave residue");
        let n = Batch::from_tuples(
            (),
            vec![Tup2(Tup2(1, 10), w(-0.2)), Tup2(Tup2(1, 10), w(-0.1))],
        );
        let merged = b.merge(&n);
        assert_eq!(tuples(&merged), vec![]);
    }

    #[test]
    fn aggregate_linear_scales_by_zweight_and_retracts_on_exact_zero() {
        let (mut dbsp, (input, output)) = Runtime::init_circuit(1, |circuit| {
            let (stream, handle) = circuit.add_input_indexed_zset::<u32, u64>();
            let totals = stream.aggregate_linear(|bits: &u64| w(f64::from_bits(*bits)));
            Ok((handle, totals.output()))
        })
        .unwrap();

        input.push(1, (0.1f64.to_bits(), 1));
        input.push(1, (0.2f64.to_bits(), 3));
        input.push(2, (0.5f64.to_bits(), 1));
        dbsp.transaction().unwrap();
        let out: Vec<(u32, W, i64)> = output.consolidate().iter().collect();
        let total1 = W::from_exact(e(0.1) + e(0.2) * 3i64);
        assert_eq!(out, vec![(1, total1, 1), (2, w(0.5), 1)]);

        input.push(1, (0.1f64.to_bits(), -1));
        input.push(1, (0.2f64.to_bits(), -3));
        dbsp.transaction().unwrap();
        let out: Vec<(u32, W, i64)> = output.consolidate().iter().collect();
        assert_eq!(out, vec![(1, total1, -1)], "the zero group is retracted, not re-emitted");

        dbsp.kill().unwrap();
    }
}
