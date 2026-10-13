//! The exponent bound on the compact tier: reached only by a chain of products, reported as
//! [`Overflow::Exponent`] by plain code in every build, and out of reach of any sum or any product
//! of two doubles. Every test is named `compact_exponent_*` so the task verify can filter on it.

use super::{Compact, Overflow};
use crate::exact::testing::{add, c, mul, Rng};
use crate::exact::EXACT_MAX_EXP;

/// `x` squared `k` times: `(1, e)` becomes `(1, e · 2^k)`.
fn squared(x: Compact, k: u32) -> Compact {
    (0..k).fold(x, |y, _| mul(y, y))
}

#[test]
fn compact_exponent_bound_is_two_to_the_thirty() {
    assert_eq!(EXACT_MAX_EXP, 1 << 30);
    assert_eq!(EXACT_MAX_EXP, 1_073_741_824);
}

#[test]
fn compact_exponent_at_the_bound_holds_and_one_past_it_is_refused_by_name() {
    let top = squared(c(2.0), 30);
    let bottom = squared(c(0.5), 30);
    assert_eq!(top.parts(), (1, 0, EXACT_MAX_EXP));
    assert_eq!(bottom.parts(), (1, 0, -EXACT_MAX_EXP));
    // One past the bound, by a product on either side and in either operand order.
    assert_eq!(top.checked_mul(c(2.0)), Err(Overflow::Exponent));
    assert_eq!(c(2.0).checked_mul(top), Err(Overflow::Exponent));
    assert_eq!(bottom.checked_mul(c(0.5)), Err(Overflow::Exponent));
    assert_eq!((-top).checked_mul(c(2.0)), Err(Overflow::Exponent));
    // And by a sum whose canonical form carries past it: 2^N + 2^N is 2^(N + 1).
    assert_eq!(top.checked_add(top), Err(Overflow::Exponent));
    assert_eq!((-top).checked_add(-top), Err(Overflow::Exponent));
    // Squaring the top overflows the exponent long before the mantissa.
    assert_eq!(top.checked_mul(top), Err(Overflow::Exponent));
    // The same shapes one step inside the bound hold.
    assert_eq!(mul(top, c(0.5)).parts(), (1, 0, EXACT_MAX_EXP - 1));
    assert_eq!(add(bottom, bottom).parts(), (1, 0, -EXACT_MAX_EXP + 1));
    assert_eq!(mul(top, bottom), c(1.0));
    // A sum across the whole bound is a width overflow, not an exponent one: the two are told
    // apart.
    assert_eq!(top.checked_add(bottom), Err(Overflow::NeedsWide));
    assert_eq!(bottom.checked_add(c(1.0)), Err(Overflow::NeedsWide));
    // The value at the bound reads as the overflow it is, and compares like any other.
    assert_eq!(top.read(), (f64::INFINITY, true));
    assert_eq!(bottom.read(), (0.0, true));
    assert!(bottom < c(5e-324));
    assert!(top > c(f64::MAX));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "a million products reach the bound, and their count is the assertion"
)]
fn compact_exponent_chain_of_smallest_subnormals_stops_at_the_bound() {
    // Each factor lowers the exponent by 1074; the chain stops on the product that would pass
    // the bound, which names the exponent and not the width, since the mantissa stays one bit.
    let mut x = c(5e-324);
    let mut factors = 1;
    loop {
        match x.checked_mul(c(5e-324)) {
            Ok(next) => {
                x = next;
                factors += 1;
            }
            Err(e) => {
                assert_eq!(e, Overflow::Exponent);
                break;
            }
        }
    }
    assert_eq!(factors, EXACT_MAX_EXP / 1074);
    assert_eq!(x.parts(), (1, 0, -1074 * factors));
    assert!(x.parts().2 >= -EXACT_MAX_EXP);
    assert!(x.parts().2 - 1074 < -EXACT_MAX_EXP);
}

#[test]
fn compact_exponent_never_limits_sums_or_two_factor_products() {
    let mut rng = Rng(28);
    for _ in 0..crate::exact::testing::rounds(100_000) {
        let (a, b) = (rng.finite(), rng.finite());
        assert_ne!(
            c(a).checked_add(c(b)),
            Err(Overflow::Exponent),
            "{a:e} + {b:e}"
        );
        assert!(Compact::from_product(a, b).is_ok(), "{a:e} * {b:e}");
        assert_ne!(
            c(a).checked_mul(c(b)),
            Err(Overflow::Exponent),
            "{a:e} * {b:e}"
        );
    }
    // The extremes any two doubles reach, each a small fraction of the bound.
    assert_eq!(mul(c(f64::MAX), c(f64::MAX)).parts().2, 1942);
    assert_eq!(mul(c(5e-324), c(5e-324)).parts().2, -2148);
    assert_eq!(add(c(f64::MAX), c(f64::MAX)).parts().2, 972);
}

/// The furthest any sum or two-factor product reaches is a small fraction of the bound.
const _: () = assert!(2148 + 127 < EXACT_MAX_EXP / 100_000);
