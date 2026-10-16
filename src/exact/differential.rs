//! differential-dataflow's weight traits on [`Exact`], DESIGN.md § differential-dataflow, so a
//! collection takes it as a diff the way it takes `isize`. Each impl is the group operator it
//! names: `plus_equals` is `+=`, `negate` is `Neg`, `zero` is [`Exact::ZERO`], `is_zero` is
//! [`Exact::is_zero`], which is what deletes an update, and `Multiply` is `Mul` by `Exact`, `i64`
//! or `isize` on either side, so a `join`'s `R × R2` and an `explode`'s `R2 × R` reach the same
//! integer arithmetic. Past the cap the operators panic with the named invariant, because
//! `plus_equals` cannot return an error to the engine.
//!
//! [`BandedSum`] is the banded `SUM` idiom on top of them, docs/OVERFLOW.md § The banded idiom:
//! the amount's band ([`Exact::band`]) rides beside the key through `explode` and `count_total`,
//! so no diff the engine consolidates ever leaves the compact form, and a `reduce` folds each
//! key's few band totals into the one total the plain idiom would have emitted.

use core::hash::Hash;

use differential_dataflow::difference::{Abelian, IsZero, Monoid, Multiply, Semigroup};
use differential_dataflow::lattice::Lattice;
use differential_dataflow::operators::CountTotal;
use differential_dataflow::{ExchangeData, VecCollection};
use timely::order::TotalOrder;
use timely::progress::Timestamp;

use super::Exact;

impl IsZero for Exact {
    #[inline]
    fn is_zero(&self) -> bool {
        Exact::is_zero(self)
    }
}

impl Semigroup for Exact {
    #[inline]
    fn plus_equals(&mut self, rhs: &Self) {
        *self += rhs;
    }
}

impl Monoid for Exact {
    #[inline]
    fn zero() -> Self {
        Self::ZERO
    }
}

impl Abelian for Exact {
    #[inline]
    fn negate(&mut self) {
        self.negate_assign();
    }
}

impl Multiply<Exact> for Exact {
    type Output = Exact;
    #[inline]
    fn multiply(self, rhs: &Exact) -> Exact {
        self * rhs
    }
}

impl Multiply<i64> for Exact {
    type Output = Exact;
    #[inline]
    fn multiply(self, rhs: &i64) -> Exact {
        self * *rhs
    }
}

impl Multiply<isize> for Exact {
    type Output = Exact;
    #[inline]
    fn multiply(self, rhs: &isize) -> Exact {
        self * *rhs
    }
}

impl Multiply<Exact> for i64 {
    type Output = Exact;
    #[inline]
    fn multiply(self, rhs: &Exact) -> Exact {
        rhs * self
    }
}

impl Multiply<Exact> for isize {
    type Output = Exact;
    #[inline]
    fn multiply(self, rhs: &Exact) -> Exact {
        rhs * self
    }
}

/// The banded `SUM` idiom, docs/OVERFLOW.md § The banded idiom: a sum of doubles inside a
/// dataflow with no promotion on the way.
///
/// The plain idiom, `explode` the amount into the diff and `count_total`, is correct on every
/// input on the heap form, and on same-scale amounts its diffs stay compact. On amounts of mixed
/// magnitudes a key's running diff spans past the width, promotes, and every consolidation step
/// from then on adds limbs on the heap. This idiom puts the amount's magnitude band
/// ([`Exact::band`]) beside the key, so every diff differential forms is a sum inside one band
/// and spans at most `84 + log₂ n` bits, inside the compact form for any `n` under 2⁴³ live rows
/// per `(key, band)`; the one value that can promote is the cross-band sum a `reduce` forms once
/// per key change over the key's at most 66 band totals, and that value is emitted, never
/// consolidated. Reach for it when a key's amounts span magnitudes, for sums alone: a product has
/// a 106-bit mantissa and no band narrow enough, and a `join` of two weighted collections is the
/// plain idiom's.
pub trait BandedSum<'scope, T: Timestamp, K> {
    /// Each key's exact total as data with multiplicity one, as `explode` into the diff then
    /// `count_total` emit it, from rows of `(key, amount as the double's bits)`: a collection's
    /// data must be `Ord`, and `f64` is not. A row's multiplicity scales its amount, so a
    /// retraction is the row at `−1`. A key whose total is the zero has no row, as it has none
    /// under `count_total`, where `is_zero` deletes it. NaN and ±∞ are counted in band 255 and
    /// projected at read, as in any `Exact`.
    fn banded_sum(self) -> VecCollection<'scope, T, (K, Exact), isize>;
}

impl<'scope, T, K> BandedSum<'scope, T, K> for VecCollection<'scope, T, (K, u64), isize>
where
    T: Timestamp + TotalOrder + Lattice,
    K: ExchangeData + Hash,
{
    fn banded_sum(self) -> VecCollection<'scope, T, (K, Exact), isize> {
        self.explode(|(key, bits)| {
            let x = f64::from_bits(bits);
            Some(((key, Exact::band(x)), Exact::from_f64(x)))
        })
        .count_total()
        .map(|((key, band), total)| (key, (band, total)))
        .reduce(|_key, bands, output| {
            let mut total = Exact::ZERO;
            for ((_band, band_total), multiplicity) in bands {
                total += band_total * *multiplicity as i64;
            }
            if !total.is_zero() {
                output.push((total, 1));
            }
        })
    }
}
