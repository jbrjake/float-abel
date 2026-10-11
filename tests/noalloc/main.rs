//! `Exact` on a build with no features: the compact tier alone, where the inline width is the
//! cap and a special input is past it. The operators panic by name, the checked methods return
//! the cap and leave the state as it was, and everything inside the width works as it does on
//! every other build. Every test is named `noalloc_*`.

#![allow(clippy::float_cmp)]

use float_abel::{Exact, PastCap};

const _: () = assert!(core::mem::size_of::<Exact>() == 24);

#[test]
fn noalloc_sums_inside_the_width_are_exact_and_read_like_ieee() {
    assert_eq!(core::mem::size_of::<Exact>(), 24);
    let sum = Exact::from(0.1) + Exact::from(0.2);
    assert_eq!(sum.to_f64(), 0.1 + 0.2);
    assert_eq!(sum.try_to_f64(), None);
    assert!(sum.is_compact());
    assert!(sum.is_finite());
    assert_eq!(Exact::from(0.5) + Exact::from(0.25), Exact::from(0.75));
    assert_eq!(sum.clone() - Exact::from(0.2), Exact::from(0.1));
    assert_eq!(Exact::from(1.5) - Exact::from(1.5), Exact::ZERO);
    let mut s = Exact::ZERO;
    s.deposit(2.0);
    s.deposit_product(3.0, 4.0);
    assert_eq!(s.try_to_f64(), Some(14.0));
    assert_eq!((s * 3i64).try_to_f64(), Some(42.0));
}

#[test]
#[should_panic(expected = "width")]
fn noalloc_a_sum_past_the_inline_width_panics_by_name() {
    let _ = Exact::from(1e300) + Exact::from(1e-300);
}

#[test]
#[should_panic(expected = "NaN or ±∞")]
fn noalloc_a_special_input_panics_by_name() {
    let _ = Exact::from(f64::NAN);
}

#[test]
#[should_panic(expected = "NaN or ±∞")]
fn noalloc_a_special_factor_panics_by_name() {
    let mut s = Exact::from(1.0);
    s.deposit_product(f64::INFINITY, 2.0);
}

#[test]
fn noalloc_the_checked_methods_return_the_cap_and_leave_the_state() {
    let big = Exact::from(1e300);
    let small = Exact::from(1e-300);
    assert_eq!(big.checked_add(&small), Err(PastCap::Width));
    assert_eq!(small.checked_add(&big), Err(PastCap::Width));
    assert_eq!(big.checked_sub(&small), Err(PastCap::Width));
    assert_eq!(big, Exact::from(1e300));
    assert_eq!(Exact::try_from_f64(f64::NAN), Err(PastCap::Special));
    assert_eq!(Exact::try_from_f64(f64::INFINITY), Err(PastCap::Special));
    assert_eq!(
        Exact::try_from_f64(f64::NEG_INFINITY),
        Err(PastCap::Special)
    );
    assert_eq!(Exact::checked_product(f64::NAN, 0.0), Err(PastCap::Special));
    assert_eq!(
        Exact::checked_product(2.0, f64::NEG_INFINITY),
        Err(PastCap::Special)
    );
    let mut s = Exact::from(1.5);
    assert_eq!(s.checked_deposit(f64::NAN), Err(PastCap::Special));
    assert_eq!(s.checked_deposit(2.5), Ok(()));
    assert_eq!(s.checked_deposit(1e-300), Err(PastCap::Width));
    assert_eq!(
        s.checked_deposit_product(1e-200, 1e-200),
        Err(PastCap::Width)
    );
    assert_eq!(
        s.checked_deposit_product(f64::INFINITY, 1.0),
        Err(PastCap::Special)
    );
    assert_eq!(s, Exact::from(4.0));
    // The exponent bound is the same cap on every build.
    let top = (0..30).fold(Exact::from(2.0), |x, _| x.clone() * x);
    assert_eq!(top.checked_mul(&Exact::from(2.0)), Err(PastCap::Exponent));
    assert_eq!(top.checked_mul_i64(2), Err(PastCap::Exponent));
    assert_eq!(top.checked_mul_i64(3).map(|v| v.is_compact()), Ok(true));
    // The cap names itself.
    assert!(format!("{}", PastCap::Width).contains("width"));
    assert!(format!("{}", PastCap::Exponent).contains("exponent"));
    assert!(format!("{}", PastCap::Special).contains("NaN or ±∞"));
}
