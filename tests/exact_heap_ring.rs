//! The ring on the heap arm of `Exact`, DESIGN.md § NaN and ±∞: products of promoted values
//! against a `num-bigint` oracle, a join chain of depth three, the dyadic special amounts the heap
//! form carries, the laws kit on the 2^±300 band, a promoted total crossing differential-dataflow's
//! serialized exchange, and the serde body a heap value crosses as. Every test is named
//! `exact_heap_*` so the task verify can filter on it.

// The tests name their operands as the algebra does.
#![allow(clippy::many_single_char_names)]

use float_abel::laws::proptest::prelude::*;
use float_abel::laws::{self, Weight};
use float_abel::{Exact, OutOfWindow, Wide};
use num_bigint::BigInt;

use crate::differential::{netted, sum_dataflow, Row};
use crate::exact_heap::{two_to, units, wide_units};
use crate::support::{hash, rounds, Rng};

fn e(x: f64) -> Exact {
    Exact::from(x)
}

/// `x`'s finite part in units of `2^-2148`, the grid of a product of two doubles, through `Wide`.
fn product_units(x: &Exact) -> BigInt {
    wide_units(&Wide::try_from(x).expect("a product fold of this band lies inside Wide's window"))
        << 1074
}

#[test]
fn exact_heap_ring_a_dot_product_of_mixed_pairs_matches_the_oracle_both_ways() {
    let mut rng = Rng(60);
    let n = rounds(100_000);
    let pairs: Vec<(f64, f64)> = (0..n)
        .map(|_| (rng.on_grid(-300, 300), rng.on_grid(-300, 300)))
        .collect();
    let mut deposited = Exact::ZERO;
    let mut folded = Exact::ZERO;
    let mut checked = Exact::ZERO;
    let mut oracle = BigInt::from(0);
    for &(a, b) in &pairs {
        deposited.deposit_product(a, b);
        folded += Exact::product(a, b);
        checked.checked_deposit_product(a, b).unwrap();
        oracle += units(a) * units(b);
    }
    assert!(!deposited.is_compact());
    assert_eq!(deposited, folded);
    assert_eq!(deposited, checked);
    assert_eq!(hash(&deposited), hash(&folded));
    assert_eq!(product_units(&deposited), oracle);
    // Retract every other row's product and the oracle follows; retract the rest and the slot is
    // the zero word.
    for &(a, b) in pairs.iter().step_by(2) {
        deposited -= Exact::product(a, b);
        oracle -= units(a) * units(b);
    }
    assert_eq!(product_units(&deposited), oracle);
    for &(a, b) in pairs.iter().skip(1).step_by(2) {
        deposited -= Exact::product(a, b);
    }
    assert!(deposited.is_zero());
    assert!(deposited.is_compact());
    // The bulk fold over the whole band as one `Sum`.
    let total: Exact = pairs.iter().map(|&(a, b)| Exact::product(a, b)).sum();
    assert_eq!(total, folded);
}

#[test]
fn exact_heap_ring_a_join_chain_of_depth_three_multiplies_and_distributes_bit_for_bit() {
    let mut rng = Rng(61);
    let mut heap_products = 0;
    let draws = rounds(2_000);
    for _ in 0..draws {
        let mut draw = || e(rng.on_grid(-300, 300)) + e(rng.on_grid(-300, 300));
        let (a, b, c, d) = (draw(), draw(), draw(), draw());
        let abc = &(&a * &b) * &c;
        heap_products += usize::from(!abc.is_compact());
        assert_eq!(abc, &a * &(&b * &c));
        assert_eq!(&a * &b, &b * &a);
        assert_eq!(&(&a + &b) * &c, &(&a * &c) + &(&b * &c));
        assert_eq!(&(&a - &b) * &c, &(&a * &c) - &(&b * &c));
        assert_eq!(&abc * &d, &a * &(&b * &(&c * &d)));
        assert!((&abc - &abc).is_zero());
        assert_eq!(&a * &Exact::ZERO, Exact::ZERO);
        assert_eq!(&a * &e(1.0), a);
        assert_eq!(&a * &e(-1.0), -&a);
        assert_eq!(&(&a * &b) * 3i64, &(&a * 3i64) * &b);
        assert_eq!(&(&a * &b) - &(&a * &b), Exact::ZERO);
    }
    assert!(heap_products > draws * 9 / 10, "{heap_products} of {draws}");
    // A depth-3 chain of doubles against the oracle, in units of 2^-3222.
    for _ in 0..rounds(2_000) {
        let (a, b, c) = (
            rng.on_grid(-300, 300),
            rng.on_grid(-300, 300),
            rng.on_grid(-300, 300),
        );
        let x = &Exact::product(a, b) * &e(c);
        let w = Wide::try_from(&x).expect("three factors of this band lie inside Wide's window");
        assert_eq!(
            wide_units(&w) << 2148,
            units(a) * units(b) * units(c),
            "{a:e} {b:e} {c:e}"
        );
        assert_eq!(x, &(&e(a) * &e(b)) * &e(c));
        assert_eq!(x, &e(a) * &(&e(b) * &e(c)));
    }
}

#[test]
fn exact_heap_ring_dyadic_special_amounts_promote_and_project() {
    let nan = e(f64::NAN);
    let inf = e(f64::INFINITY);
    let half_nan = &e(2.5) * &nan;
    assert!(!half_nan.is_compact());
    assert!(half_nan.to_f64().is_nan());
    assert_eq!(half_nan.specials(), None);
    assert!(!half_nan.is_finite());
    assert!(!half_nan.is_zero());
    assert_eq!(&nan * &e(2.5), half_nan);
    assert_eq!(e(2.5).checked_mul(&nan), Ok(half_nan.clone()));
    // Two halves make an integer count, which is the compact form again.
    assert_eq!(&half_nan * &e(2.0), &nan * 5i64);
    assert!((&half_nan * &e(2.0)).is_compact());
    assert_eq!(&half_nan + &half_nan, &nan * 5i64);
    assert!((&half_nan - &half_nan).is_zero());
    assert_eq!(&half_nan * 2i64, &nan * 5i64);
    let half_inf = &e(0.5) * &inf;
    assert!(!half_inf.is_compact());
    assert_eq!(half_inf.to_f64(), f64::INFINITY);
    assert_eq!(half_inf.try_to_f64(), Some(f64::INFINITY));
    assert_eq!(&half_inf + &half_inf, inf);
    assert!((&half_inf + &half_inf).is_compact());
    assert_eq!((&e(-1.0) * &inf).to_f64(), f64::NEG_INFINITY);
    assert_eq!((&e(-0.5) * &inf).to_f64(), f64::NEG_INFINITY);
    assert_eq!(&(&e(-0.5) * &inf) * &e(-2.0), inf);
    assert_eq!(&e(2.5) * &(&nan * 2i64), &nan * 5i64);
    assert_eq!(
        &(&e(0.5) * &inf) * &(&e(0.5) * &e(f64::NEG_INFINITY)),
        &e(0.25) * &e(f64::NEG_INFINITY)
    );
    // An exact zero annihilates on the heap form too.
    assert_eq!(&Exact::ZERO * &half_nan, Exact::ZERO);
    assert_eq!(&half_nan * &Exact::ZERO, Exact::ZERO);
    // An integer amount past `i64` is held but is not a count.
    let big = &two_to(300) * &inf;
    assert!(!big.is_compact());
    assert_eq!(big.specials(), None);
    assert_eq!(big.to_f64(), f64::INFINITY);
    assert_eq!(&big * &two_to(-300), inf);
    assert!((&big * &two_to(-300)).is_compact());
    // Amounts mix with a promoted finite part and come apart again.
    let raised = e(1e300) + e(1e-300);
    let mixed = &half_nan + &raised;
    assert!(mixed.to_f64().is_nan());
    assert_eq!(&mixed - &half_nan, raised);
    assert_eq!(&mixed - &raised, half_nan);
    assert_eq!(-&mixed, &-&half_nan + &-&raised);
    // `Wide` holds integer counts only: a dyadic amount is off its counters' grid, and an amount
    // past `i64` is above their ceiling, each refused by name with nothing built.
    assert_eq!(Wide::try_from(&half_nan), Err(OutOfWindow::OffGrid));
    assert_eq!(Wide::try_from(&big), Err(OutOfWindow::AboveCeiling));
    // A row's product still counts once per row, whatever the finite factor.
    assert_eq!(Exact::product(2.5, f64::NAN).specials(), Some((1, 0, 0)));
    assert_eq!(
        Exact::product(f64::INFINITY, -2.5).specials(),
        Some((0, -1, 0))
    );
    // Amounts order numerically: 2.5 NaN lies between 2 and 3 of them.
    assert!(&nan * 2i64 < half_nan);
    assert!(half_nan < &nan * 3i64);
    assert!(-&half_nan < -&(&nan * 2i64));
    assert_ne!(hash(&half_nan), hash(&(&nan * 2i64)));
}

/// A normal double with its leading bit at `2^e` for `e` in `lo..hi`, a full fraction and either
/// sign: what `tests/support.rs` draws, as a strategy.
fn double(lo: i32, hi: i32) -> impl Strategy<Value = f64> {
    (lo..hi, any::<u64>(), any::<bool>()).prop_map(|(e, frac, neg)| {
        let x = f64::from_bits(((e + 1023) as u64) << 52 | frac >> 12);
        if neg {
            -x
        } else {
            x
        }
    })
}

fn special() -> impl Strategy<Value = f64> {
    prop_oneof![Just(f64::NAN), Just(f64::INFINITY), Just(f64::NEG_INFINITY)]
}

/// The adversary of DESIGN.md § Exactness bounds: magnitudes uniform over 2^±300, one in sixteen a
/// special, so most sums and every product of two promote.
fn mixed() -> impl Strategy<Value = Exact> {
    prop_oneof![1 => special(), 15 => double(-300, 300)].prop_map(Exact::from)
}

/// Row products of that band, as a join deposits them.
fn products() -> impl Strategy<Value = Exact> {
    (double(-300, 300), double(-300, 300)).prop_map(|(a, b)| Exact::product(a, b))
}

#[test]
#[cfg(not(miri))]
fn exact_heap_ring_laws_hold_on_the_mixed_band_through_the_kit() {
    let mut r = laws::runner(256, 1);
    assert_eq!(laws::check_abelian_group(&mut r, mixed()), Ok(()));
    assert_eq!(laws::check_ring(&mut r, mixed(), -1024i64..1024), Ok(()));
    assert_eq!(laws::check_ring(&mut r, mixed(), mixed()), Ok(()));
    assert_eq!(laws::check_ring(&mut r, products(), mixed()), Ok(()));
    assert_eq!(laws::check_canonical(&mut r, mixed()), Ok(()));
    assert_eq!(
        laws::check_consolidation_invariance(&mut r, mixed()),
        Ok(())
    );
    assert!(<Exact as Weight>::zero().is_compact());
}

#[test]
fn exact_heap_ring_a_promoted_total_crosses_serialized_exchange_on_three_workers() {
    let mut rng = Rng(62);
    let rows: Vec<Row> = (0..rounds(3_000))
        .map(|i| ((i % 5) as u32, rng.on_grid(-300, 300).to_bits()))
        .collect();
    let retracts: Vec<Row> = rows.iter().copied().step_by(3).collect();
    let (inserts_for_oracle, retracts_for_oracle) = (rows.clone(), retracts.clone());
    let got = netted(3, move |worker| sum_dataflow(worker, &rows, &retracts));
    // The oracle: each key's total after epoch 0 and after epoch 1, through `Wide`.
    let total = |keep: &dyn Fn(usize) -> bool| -> Vec<Exact> {
        (0..5u32)
            .map(|key| {
                let mut w = Wide::ZERO;
                for (i, &(k, bits)) in inserts_for_oracle.iter().enumerate() {
                    if k == key && keep(i) {
                        w.deposit(f64::from_bits(bits));
                    }
                }
                Exact::try_from(&w).unwrap()
            })
            .collect()
    };
    let after0 = total(&|_| true);
    let after1 = total(&|i| i % 3 != 0);
    assert_eq!(
        retracts_for_oracle.len(),
        inserts_for_oracle.len().div_ceil(3)
    );
    let mut expected = Vec::new();
    for key in 0..5u32 {
        let (t0, t1) = (after0[key as usize].clone(), after1[key as usize].clone());
        assert!(!t0.is_compact(), "key {key} did not promote");
        expected.push(((key, t0.clone()), 0u64, 1isize));
        expected.push(((key, t0), 1, -1));
        expected.push(((key, t1), 1, 1));
    }
    differential_dataflow::consolidation::consolidate_updates(&mut expected);
    assert_eq!(got, expected);
}

/// The serde body a heap value crosses as on day one: a tag byte, then for the finite part and
/// the three amounts a sign byte, an `i32` exponent and a `u32` limb count, then the limbs. The
/// wire task owns the final encoding; this pins what the exchange relies on now.
const HEAP_TAG: u8 = 0x04;

fn heap_body(runs: &[(bool, i32, &[u64])]) -> Vec<u8> {
    assert_eq!(runs.len(), 4);
    let mut out = vec![HEAP_TAG];
    for &(neg, exp, limbs) in runs {
        out.push(u8::from(neg));
        out.extend_from_slice(&exp.to_le_bytes());
        out.extend_from_slice(&(limbs.len() as u32).to_le_bytes());
    }
    for &(_, _, limbs) in runs {
        for l in limbs {
            out.extend_from_slice(&l.to_le_bytes());
        }
    }
    out
}

/// The byte string behind postcard's varint length prefix.
fn unframed(bytes: &[u8]) -> &[u8] {
    let prefix = bytes.iter().position(|b| b & 0x80 == 0).unwrap() + 1;
    &bytes[prefix..]
}

#[test]
fn exact_heap_ring_serde_carries_a_heap_value_and_refuses_a_body_that_fits_the_compact_form() {
    let x = e(1e300) + e(1e-300);
    let bytes = postcard::to_allocvec(&x).unwrap();
    let body = unframed(&bytes);
    assert_eq!(body[0], HEAP_TAG);
    // Four run headers of nine bytes, then the finite part's 32 limbs: 2,046 bits from 2^-1049
    // up to 2^996, and no amounts.
    assert_eq!(body.len(), 1 + 4 * 9 + 8 * 32);
    assert_eq!(postcard::from_bytes::<Exact>(&bytes).unwrap(), x);
    let y = &(&e(2.5) * &e(f64::NAN)) + &x;
    let bytes = postcard::to_allocvec(&y).unwrap();
    assert_eq!(postcard::from_bytes::<Exact>(&bytes).unwrap(), y);
    let json = serde_json::to_string(&y).unwrap();
    assert_eq!(serde_json::from_str::<Exact>(&json).unwrap(), y);
    // The compact encoding is untouched: 33 bytes behind a one-byte varint.
    let c = e(1.5);
    let bytes = postcard::to_allocvec(&c).unwrap();
    assert_eq!(bytes.len(), 34);
    assert_eq!(unframed(&bytes), &c.to_bytes());
    // Hand-built bodies: 1.5 in the heap form fits the compact form, so it is refused; so is a
    // leading zero limb, an even low limb, a sign on an empty run, and a count past the cap.
    let refused = |body: Vec<u8>, why: &str| {
        let err = serde_json::from_str::<Exact>(&format!("{body:?}")).unwrap_err();
        assert!(err.to_string().contains(why), "{err}: {body:?}");
        assert!(postcard::from_bytes::<Exact>(
            &postcard::to_allocvec(&serde_bytes_of(&body)).unwrap()
        )
        .is_err());
    };
    let none: &[u64] = &[];
    refused(
        heap_body(&[
            (false, -1, &[3]),
            (false, 0, none),
            (false, 0, none),
            (false, 0, none),
        ]),
        "compact",
    );
    refused(
        heap_body(&[
            (false, 0, &[1, 1, 0]),
            (false, 0, none),
            (false, 0, none),
            (false, 0, none),
        ]),
        "normalised",
    );
    refused(
        heap_body(&[
            (false, 0, &[2, 1, 1]),
            (false, 0, none),
            (false, 0, none),
            (false, 0, none),
        ]),
        "even",
    );
    refused(
        heap_body(&[
            (false, 0, &[1, 0, 0, 1]),
            (true, 0, none),
            (false, 0, none),
            (false, 0, none),
        ]),
        "normalised",
    );
    let mut past_cap = heap_body(&[
        (false, 0, &[1, 0, 0, 1]),
        (false, 0, none),
        (false, 0, none),
        (false, 0, none),
    ]);
    past_cap[6..10].copy_from_slice(&(float_abel::EXACT_MAX_LIMBS as u32 + 1).to_le_bytes());
    refused(past_cap, "limb");
    // A heap value that holds a count past `i32` round-trips with its amounts.
    let z = &(&e(f64::NAN) * i64::from(i32::MAX)) + &e(f64::NAN);
    assert!(!z.is_compact());
    let bytes = postcard::to_allocvec(&z).unwrap();
    assert_eq!(postcard::from_bytes::<Exact>(&bytes).unwrap(), z);
}

/// The byte string serde sees for `body`, so a hand-built body can be framed the way postcard
/// frames an `Exact`.
fn serde_bytes_of(body: &[u8]) -> serde_bytes_shim::Bytes<'_> {
    serde_bytes_shim::Bytes(body)
}

mod serde_bytes_shim {
    use serde::{Serialize, Serializer};

    pub(super) struct Bytes<'a>(pub(super) &'a [u8]);

    impl Serialize for Bytes<'_> {
        fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            s.serialize_bytes(self.0)
        }
    }
}
