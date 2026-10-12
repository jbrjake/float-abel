//! `Exact` and `Binned` on a build with no features: plain `Copy` words that work exactly as on
//! every other build, since nothing in the crate allocates. The operators panic by name past the
//! cap and the checked methods return it with the state untouched. Every test is named
//! `noalloc_*`.

#![allow(clippy::float_cmp)]

use float_abel::{Binned, Exact, PastCap, Wide, WireError};

const _: () = assert!(core::mem::size_of::<Exact>() == 32);
const _: () = assert!(core::mem::size_of::<Binned<3>>() == 80);

#[test]
fn noalloc_a_binned_fold_lands_in_wide_without_std() {
    let xs = [1e30, 1e-10, -1e30, 0.5, f64::NAN];
    let mut forward = Binned::<3>::ZERO;
    forward.extend_from_slice(&xs);
    let backward = xs
        .iter()
        .rev()
        .fold(Binned::<3>::ZERO, |acc, &x| acc + Binned::from(x));
    assert_eq!(forward, backward);
    assert_eq!(forward.specials(), (1, 0, 0));
    let w = forward.into_wide();
    assert!(w.to_f64().is_nan());
    // The history is gone from the Wide: its window starts where the survivors are, two limbs
    // below the 1e30 that fixed the fold's, so only the value round-trips, not the bits.
    assert_eq!(w.to_binned::<3>().into_wide(), w);
    assert_eq!(forward.index(), 18);
    assert_eq!(w.to_binned::<3>().index(), 16);
    let mut finite = forward;
    finite = finite + Binned::from(0.0);
    assert_eq!(finite, forward);
    assert_eq!(
        Wide::from(0.5).to_binned::<3>().into_wide(),
        Wide::from(0.5)
    );
}

#[test]
fn noalloc_sums_products_and_specials_work_without_std() {
    assert_eq!(core::mem::size_of::<Exact>(), 32);
    let sum = Exact::from(0.1) + Exact::from(0.2);
    assert_eq!(sum.to_f64(), 0.1 + 0.2);
    assert_eq!(sum.try_to_f64(), None);
    assert_eq!(Exact::from(0.5) + Exact::from(0.25), Exact::from(0.75));
    assert_eq!(sum - Exact::from(0.2), Exact::from(0.1));
    assert_eq!(Exact::from(1.5) - Exact::from(1.5), Exact::ZERO);
    let mut s = Exact::ZERO;
    s.deposit(2.0);
    s.deposit_product(3.0, 4.0);
    assert_eq!(s.try_to_f64(), Some(14.0));
    assert_eq!((s * 3i64).try_to_f64(), Some(42.0));
    assert_eq!(
        (Exact::from(1.5) * Exact::from(4.0)).try_to_f64(),
        Some(6.0)
    );
    let mut n = Exact::from(1.0);
    n += Exact::from(f64::NAN);
    assert!(n.to_f64().is_nan());
    n -= Exact::from(f64::NAN);
    assert_eq!(n, Exact::from(1.0));
    assert_eq!(Exact::from(f64::INFINITY).specials(), (0, 1, 0));
}

#[test]
#[should_panic(expected = "width")]
fn noalloc_a_sum_past_the_width_panics_by_name() {
    let _ = Exact::from(1e300) + Exact::from(1e-300);
}

#[test]
#[should_panic(expected = "count")]
fn noalloc_a_fractional_special_amount_panics_by_name() {
    let _ = Exact::from(2.5) * Exact::from(f64::NAN);
}

#[test]
fn noalloc_the_checked_methods_return_the_cap_and_leave_the_state() {
    let big = Exact::from(1e300);
    let small = Exact::from(1e-300);
    assert_eq!(big.checked_add(&small), Err(PastCap::Width));
    assert_eq!(small.checked_add(&big), Err(PastCap::Width));
    assert_eq!(big.checked_sub(&small), Err(PastCap::Width));
    assert_eq!(big, Exact::from(1e300));
    let mut s = Exact::from(1.5);
    assert_eq!(s.checked_deposit(2.5), Ok(()));
    assert_eq!(s.checked_deposit(1e-300), Err(PastCap::Width));
    assert_eq!(
        s.checked_deposit_product(1e-200, 1e-200),
        Err(PastCap::Width)
    );
    assert_eq!(s, Exact::from(4.0));
    let top = (0..30).fold(Exact::from(2.0), |x, _| x * x);
    assert_eq!(top.checked_mul(&Exact::from(2.0)), Err(PastCap::Exponent));
    assert_eq!(top.checked_mul_i64(2), Err(PastCap::Exponent));
    assert_eq!(
        Exact::from(0.5).checked_mul(&Exact::from(f64::NAN)),
        Err(PastCap::Count)
    );
    assert!(format!("{}", PastCap::Width).contains("width"));
    assert!(format!("{}", PastCap::Exponent).contains("exponent"));
    assert!(format!("{}", PastCap::Count).contains("count"));
}

#[test]
fn noalloc_the_wire_codec_needs_no_feature() {
    let x = Exact::from(1.5) + Exact::from(f64::NAN);
    let bytes = x.to_bytes();
    assert_eq!(bytes.len(), 33);
    assert_eq!(bytes[0], Exact::WIRE_VERSION);
    assert_eq!(Exact::from_bytes(&bytes), Ok(x));
    assert_eq!(
        Exact::from_bytes(&bytes[..32]),
        Err(WireError::Length {
            expected: 33,
            found: 32
        })
    );
    let mut even = bytes;
    even[1] = 2;
    assert_eq!(Exact::from_bytes(&even), Err(WireError::EvenMantissa));
    assert!(format!("{}", WireError::EvenMantissa).contains("even"));
}
