//! [`Exact`]: the engine weight, a dyadic held in the smallest tier that represents it. The
//! compact tier is 127 bits of mantissa inline, in `compact`; the heap tier is the `alloc`
//! feature's reason to exist.
//!
//! The tier enum sits behind a newtype because an enum's variant fields are as visible as the
//! enum, and a hand-built `Compact { lo: 2, .. }` would be a non-canonical value, which structural
//! `Eq` cannot afford. The layout is load-bearing: a struct variant lets the discriminant share the
//! word that `exp: i32` half fills, so the whole is 24 bytes where a nested struct or an `i128`
//! would make it 32.

// No public item reaches the exact tiers: Exact has no public surface without its heap arm.
#![cfg_attr(not(test), allow(dead_code))]

mod compact;
#[cfg(test)]
pub(crate) mod testing;

pub(crate) use compact::Compact;

/// The exponent bound: every held value is `m · 2^exp` with `|exp| ≤ EXACT_MAX_EXP`, checked
/// after every operation that moves an exponent, in every build. A sum's exponent stays inside its
/// inputs' range plus the mantissa width, and a product of two doubles lies within `±2148`, so only
/// a chain of nested products moves an exponent far: roughly a million products of the smallest
/// subnormal reach the bound. Past it is the cap. The number keeps every intermediate inside `i64`
/// and every stored exponent inside `i32`, with room to spare on both sides.
pub(crate) const EXACT_MAX_EXP: i32 = 1 << 30;

/// An exact abelian group of every `f64`, specials included, with a ring on top, in 24 bytes.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Exact(Tier);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Tier {
    /// `m = (hi:lo)` as a signed 128-bit integer and the value `m · 2^exp`, with `m` odd, or the
    /// zero `(0, 0, 0)`.
    Compact { lo: u64, hi: u64, exp: i32 },
}

const _: () = assert!(
    core::mem::size_of::<Exact>() == 24,
    "Exact's layout moved: a tier moved"
);

impl Exact {
    /// The zero, which is the only zero.
    pub(crate) const ZERO: Self = Self(Tier::Compact {
        lo: 0,
        hi: 0,
        exp: 0,
    });

    pub(crate) fn from_compact(c: Compact) -> Self {
        let (lo, hi, exp) = c.parts();
        Self(Tier::Compact { lo, hi, exp })
    }

    /// The value in its working form.
    pub(crate) fn compact(&self) -> Compact {
        let Tier::Compact { lo, hi, exp } = self.0;
        Compact::from_parts(lo, hi, exp)
    }
}
