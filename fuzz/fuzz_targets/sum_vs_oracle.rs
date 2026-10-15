//! A multiset of doubles, each inserted or retracted, through every tier against the oracle,
//! DESIGN.md § Hardening: `Wide`'s limbs and counts are the oracle's bit for bit and its reads are
//! the oracle rounded; `Fixed256` accepts exactly its window and holds the same integer; `Exact`
//! agrees with both wherever the width holds the running sum; `Binned` lands the truncated sum the
//! design defines; and the bulk entries land the one-at-a-time bits through whatever kernel this
//! machine dispatches.

#![no_main]

use float_abel::{Binned, Exact, Fixed256, PastCap, Wide};
use float_abel_fuzz::{binned_truncated, fixed256_verdict, to_fixed256, units, Oracle};
use libfuzzer_sys::fuzz_target;
use num_bigint::BigInt;

/// Bit-equal doubles, with every NaN one NaN.
fn same(got: f64, want: f64, what: &str) {
    if want.is_nan() {
        assert!(got.is_nan(), "{what}: {got:e} is not NaN");
    } else {
        assert_eq!(got.to_bits(), want.to_bits(), "{what}: {got:e} vs {want:e}");
    }
}

fn check_binned<const K: usize>(xs: &[f64]) {
    let (lo, truncated) = binned_truncated::<K>(xs);
    let mut fold = Binned::<K>::ZERO;
    for &x in xs {
        fold.deposit(x);
    }
    assert_eq!(
        u64::from(fold.index().unsigned_abs()),
        lo + K as u64 - 1,
        "K={K}: index"
    );
    assert_eq!(
        fold.into_wide(),
        truncated.to_wide(),
        "K={K}: the truncated sum"
    );
    let mut bulk = Binned::<K>::ZERO;
    bulk.extend_from_slice(xs);
    assert_eq!(bulk, fold, "K={K}: the bulk entry");
}

fuzz_target!(|ops: Vec<(f64, bool)>| {
    let mut oracle = Oracle::default();
    let mut wide = Wide::ZERO;
    let mut exact = Some(Exact::ZERO);
    let mut fixed = Some(Fixed256::ZERO);
    let mut fixed_units = BigInt::default();
    for &(x, retract) in &ops {
        let by = if retract { -1 } else { 1 };
        oracle.add(x, by);
        if retract {
            wide -= Wide::from(x);
        } else {
            wide.deposit(x);
        }
        exact = exact.and_then(|e| {
            let term = Exact::from(x);
            let next = if retract {
                e.checked_sub(&term)
            } else {
                e.checked_add(&term)
            };
            match next {
                Ok(v) => Some(v),
                Err(cap) => {
                    assert_eq!(
                        cap,
                        PastCap::Width,
                        "{x:e}: a sum of doubles caps on the width alone"
                    );
                    None
                }
            }
        });
        if let Some(f) = fixed.as_mut() {
            let term = Fixed256::try_from(x);
            assert_eq!(
                term.map(|_| ()),
                fixed256_verdict(x),
                "{x:e}: Fixed256's window"
            );
            match term {
                Ok(t) => {
                    if retract {
                        *f -= t;
                    } else {
                        *f += t;
                    }
                    fixed_units += units(x).expect("finite") * by;
                }
                Err(_) => fixed = None,
            }
        }
    }

    // Wide is the oracle bit for bit, and reads it rounded.
    assert_eq!(wide, oracle.to_wide());
    let (value, exact_read) = oracle.to_f64();
    same(wide.to_f64(), value, "Wide::to_f64");
    assert_eq!(wide.try_to_f64().is_some(), exact_read, "Wide::try_to_f64");
    if let Some(v) = wide.try_to_f64() {
        same(v, value, "Wide::try_to_f64");
    }
    same(
        f64::from(wide.to_f32()),
        f64::from(oracle.to_f32()),
        "Wide::to_f32",
    );
    assert_eq!(wide.is_finite(), oracle.counts == [0; 3]);
    assert_eq!(wide.is_zero(), oracle.is_zero());

    // Fixed256 holds the same integer on its grid, and embeds in Wide exactly.
    if let Some(f) = fixed {
        assert_eq!(f, to_fixed256(&fixed_units));
        assert_eq!(Wide::from(f), wide, "Fixed256 into Wide");
        same(f.to_f64(), value, "Fixed256::to_f64");
        assert_eq!(f.try_to_f64().is_some(), exact_read, "Fixed256::try_to_f64");
        same(
            f64::from(f.to_f32()),
            f64::from(oracle.to_f32()),
            "Fixed256::to_f32",
        );
        assert_eq!(f.is_zero(), oracle.is_zero());
    }

    // Exact agrees wherever the width held every running sum, and converts both ways.
    if let Some(e) = exact {
        same(e.to_f64(), value, "Exact::to_f64");
        assert_eq!(e.try_to_f64().is_some(), exact_read, "Exact::try_to_f64");
        same(
            f64::from(e.to_f32()),
            f64::from(oracle.to_f32()),
            "Exact::to_f32",
        );
        let (n, p, q) = e.specials().expect("a sum's amounts are counts");
        assert_eq!([n, p, q], oracle.counts);
        assert_eq!(Exact::try_from(&wide), Ok(e.clone()), "Wide into Exact");
        assert_eq!(Wide::try_from(&e), Ok(wide), "Exact into Wide");
        assert_eq!(e.is_zero(), oracle.is_zero());
    }

    // The deposits alone: Binned's truncated sum, and Wide's bulk entry against its loop.
    let deposits: Vec<f64> = ops.iter().filter(|(_, r)| !r).map(|(x, _)| *x).collect();
    let mut loop_fold = Wide::ZERO;
    for &x in &deposits {
        loop_fold.deposit(x);
    }
    let mut bulk = Wide::ZERO;
    bulk.extend_from_slice(&deposits);
    assert_eq!(bulk, loop_fold, "Wide::extend_from_slice");
    check_binned::<1>(&deposits);
    check_binned::<3>(&deposits);
});
