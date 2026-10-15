//! The product of two compact [`Exact`]s: the monoid algebra D[M] of DESIGN.md § NaN and ±∞ on
//! `(f, n, p, q)`, with the special amounts kept integer so that the result is the compact word.
//!
//! ```text
//! (f, n, p, q) · (g, n′, p′, q′) =
//!   ( f·g,
//!     f·n′ + g·n + n·(n′ + p′ + q′) + n′·(p + q),   -- anything times NaN is NaN
//!     f·p′ + g·p + p·p′ + q·q′,                      -- (+∞)(+∞) = (−∞)(−∞) = +∞
//!     f·q′ + g·q + p·q′ + q·p′ )                     -- (+∞)(−∞) = −∞
//! ```
//!
//! `f·n′` is a dyadic times a count. When the count is zero the term is zero whatever `f` is, which
//! is how an exact zero annihilates (`0 · ∞ = 0`) and how the common case, no specials on either
//! side, costs one multiply. When the count is nonzero, `f` must be an integer, or the amount would
//! stop being one: that is [`Overflow::Count`], which the heap form takes over with `alloc` and
//! which is `PastCap::Count` without it.

use super::compact::{Compact, Overflow};
use super::Exact;

/// `f · k` for a count `k`: zero when `k` is, else `f` as an integer times `k`, in `i128`.
fn scaled(f: Compact, k: i32) -> Result<i128, Overflow> {
    if k == 0 {
        return Ok(0);
    }
    let f = f.to_i128().ok_or(Overflow::Count)?;
    f.checked_mul(i128::from(k)).ok_or(Overflow::Count)
}

pub(super) fn product(a: &Exact, b: &Exact) -> Result<Exact, Overflow> {
    let (fa, fb) = (a.finite(), b.finite());
    let finite = fa.checked_mul(fb)?;
    let [na, pa, qa] = a.counts().map(i128::from);
    let [nb, pb, qb] = b.counts().map(i128::from);
    let nan =
        scaled(fa, nb as i32)? + scaled(fb, na as i32)? + na * (nb + pb + qb) + nb * (pa + qa);
    let pos = scaled(fa, pb as i32)? + scaled(fb, pa as i32)? + pa * pb + qa * qb;
    let neg = scaled(fa, qb as i32)? + scaled(fb, qa as i32)? + pa * qb + qa * pb;
    let counts = [
        Exact::narrow(nan).ok_or(Overflow::Count)?,
        Exact::narrow(pos).ok_or(Overflow::Count)?,
        Exact::narrow(neg).ok_or(Overflow::Count)?,
    ];
    Ok(Exact::assemble(finite, counts))
}
