//! The heap arm's working value: the finite part and the three special amounts as normalised
//! dyadics, borrowed from either form as an [`Operand`] and owned by an operation's result as a
//! [`Value`], with the group and the ring of DESIGN.md § NaN and ±∞ on them. The product is the
//! monoid algebra ring.rs computes on integer counts, here over dyadic amounts, so `2.5 · NaN` is
//! an amount rather than a cap, and a special's amount in a product chain is as wide as the
//! finite part.

use core::cmp::Ordering;

use super::big::{self, Owned, Small, View};
use super::compact::Overflow;
use super::heap::Parts;
use super::Exact;

/// Either form of a value, viewable as four dyadics without allocating.
pub(crate) enum Operand<'a> {
    Compact { finite: Small, counts: [Small; 3] },
    Heap(Parts<'a>),
}

impl Operand<'_> {
    pub(crate) fn finite(&self) -> View<'_> {
        match self {
            Self::Compact { finite, .. } => finite.view(),
            Self::Heap(p) => p.finite(),
        }
    }

    pub(crate) fn amount(&self, i: usize) -> View<'_> {
        match self {
            Self::Compact { counts, .. } => counts[i].view(),
            Self::Heap(p) => p.amount(i),
        }
    }
}

impl Exact {
    pub(crate) fn operand(&self) -> Operand<'_> {
        if self.is_heap() {
            Operand::Heap(self.parts())
        } else {
            Operand::Compact {
                finite: Small::from_compact(self.finite()),
                counts: self.counts().map(|c| Small::from_i64(i64::from(c))),
            }
        }
    }
}

/// An operation's result: the finite part and the `[n, p, q]` amounts, owned.
pub(crate) struct Value {
    pub(crate) finite: Owned,
    pub(crate) amounts: [Owned; 3],
}

impl Value {
    /// The sum, componentwise.
    pub(crate) fn add(a: &Operand<'_>, b: &Operand<'_>) -> Result<Self, Overflow> {
        Ok(Self {
            finite: big::add(a.finite(), b.finite())?,
            amounts: [
                big::add(a.amount(0), b.amount(0))?,
                big::add(a.amount(1), b.amount(1))?,
                big::add(a.amount(2), b.amount(2))?,
            ],
        })
    }

    /// The product: `(f, n, p, q) · (g, n′, p′, q′)` as the design's monoid algebra, every term a
    /// dyadic product, so nothing is refused but the caps.
    pub(crate) fn mul(lhs: &Operand<'_>, rhs: &Operand<'_>) -> Result<Self, Overflow> {
        let (fa, fb) = (lhs.finite(), rhs.finite());
        let (na, pa, qa) = (lhs.amount(0), lhs.amount(1), lhs.amount(2));
        let (nb, pb, qb) = (rhs.amount(0), rhs.amount(1), rhs.amount(2));
        let finite = big::mul(fa, fb)?;
        // f·n′ + g·n + n·(n′ + p′ + q′) + n′·(p + q): anything times NaN is NaN.
        let b_specials = big::add(big::add(nb, pb)?.view(), qb)?;
        let a_infinities = big::add(pa, qa)?;
        let nan = sum([
            big::mul(fa, nb)?,
            big::mul(fb, na)?,
            big::mul(na, b_specials.view())?,
            big::mul(nb, a_infinities.view())?,
        ])?;
        // f·p′ + g·p + p·p′ + q·q′: (+∞)(+∞) = (−∞)(−∞) = +∞.
        let pos = sum([
            big::mul(fa, pb)?,
            big::mul(fb, pa)?,
            big::mul(pa, pb)?,
            big::mul(qa, qb)?,
        ])?;
        // f·q′ + g·q + p·q′ + q·p′: (+∞)(−∞) = −∞.
        let neg = sum([
            big::mul(fa, qb)?,
            big::mul(fb, qa)?,
            big::mul(pa, qb)?,
            big::mul(qa, pb)?,
        ])?;
        Ok(Self {
            finite,
            amounts: [nan, pos, neg],
        })
    }

    /// Integer scaling of the finite part and the amounts alike.
    pub(crate) fn mul_i64(a: &Operand<'_>, k: i64) -> Result<Self, Overflow> {
        let k = Small::from_i64(k);
        let k = k.view();
        Ok(Self {
            finite: big::mul(a.finite(), k)?,
            amounts: [
                big::mul(a.amount(0), k)?,
                big::mul(a.amount(1), k)?,
                big::mul(a.amount(2), k)?,
            ],
        })
    }
}

fn sum(terms: [Owned; 4]) -> Result<Owned, Overflow> {
    let [a, b, c, d] = terms;
    let ab = big::add(a.view(), b.view())?;
    let cd = big::add(c.view(), d.view())?;
    big::add(ab.view(), cd.view())
}

/// Numeric on the finite part, then `(n, p, q)` lexicographic: the order of DESIGN.md § Exact
/// across both forms.
pub(crate) fn cmp(a: &Operand<'_>, b: &Operand<'_>) -> Ordering {
    big::cmp(a.finite(), b.finite())
        .then_with(|| big::cmp(a.amount(0), b.amount(0)))
        .then_with(|| big::cmp(a.amount(1), b.amount(1)))
        .then_with(|| big::cmp(a.amount(2), b.amount(2)))
}
