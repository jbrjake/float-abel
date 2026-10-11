//! The product of two [`Exact`]s: the monoid algebra D[M] of DESIGN.md § NaN and ±∞ on
//! `(f, n, p, q)`, with the special amounts kept integer so that the value stays a 32-byte word.
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
//! stop being one: that is [`PastCap::Count`].

use super::compact::Compact;
use super::{Exact, PastCap};

/// `f · k` for a count `k`: zero when `k` is, else `f` as an integer times `k`, in `i128`.
fn scaled(f: Compact, k: i32) -> Result<i128, PastCap> {
    if k == 0 {
        return Ok(0);
    }
    let f = f.to_i128().ok_or(PastCap::Count)?;
    f.checked_mul(i128::from(k)).ok_or(PastCap::Count)
}

pub(super) fn product(a: Exact, b: Exact) -> Result<Exact, PastCap> {
    let (fa, fb) = (a.finite(), b.finite());
    let finite = fa.checked_mul(fb).map_err(Exact::cap)?;
    let [na, pa, qa] = a.counts().map(i128::from);
    let [nb, pb, qb] = b.counts().map(i128::from);
    let nan = scaled(fa, b.nan)? + scaled(fb, a.nan)? + na * (nb + pb + qb) + nb * (pa + qa);
    let pos = scaled(fa, b.pos_inf)? + scaled(fb, a.pos_inf)? + pa * pb + qa * qb;
    let neg = scaled(fa, b.neg_inf)? + scaled(fb, a.neg_inf)? + pa * qb + qa * pb;
    Ok(Exact::assemble(
        finite,
        [
            Exact::narrow(nan)?,
            Exact::narrow(pos)?,
            Exact::narrow(neg)?,
        ],
    ))
}
