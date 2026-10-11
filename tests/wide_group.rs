//! `Wide` as a group: the laws with specials in the multiset, order-independent folds, the special
//! counters and their projection at read, and the numeric-then-lexicographic order. Every test is
//! named `wide_*` so the task verify can filter on it; `tests/wide.rs` holds the carrier and its
//! reads.

use core::cmp::Ordering;
use float_abel::Wide;

use crate::support::{hash, Rng};

const NAN: f64 = f64::NAN;
const INF: f64 = f64::INFINITY;
const NINF: f64 = f64::NEG_INFINITY;

fn w(x: f64) -> Wide {
    Wide::from(x)
}

#[test]
fn wide_specials_are_counted_and_projected_at_read() {
    // A NaN inserted then retracted leaves the finite sum, and on its own leaves the zero.
    let mut s = w(1.5);
    s += w(NAN);
    assert!(s.to_f64().is_nan());
    assert!(!s.is_finite());
    s -= w(NAN);
    assert_eq!(s, w(1.5));
    assert!(s.is_finite());
    assert_eq!(s.to_f64(), 1.5);
    let mut z = w(NAN);
    z -= w(NAN);
    assert!(z.is_zero());
    assert_eq!(z, Wide::ZERO);
    // Projection order: NaN first, then both infinities, then one infinity by the sign of its
    // count, then the finite part.
    assert!((w(NAN) + w(INF) + w(1.0)).to_f64().is_nan());
    assert!((w(INF) + w(NINF)).to_f64().is_nan());
    assert_eq!((w(INF) + w(NINF)).specials(), (0, 1, 1));
    assert!((w(INF) * 2i64 + w(NINF)).to_f64().is_nan());
    assert!(
        (-w(INF) + w(NINF)).to_f64().is_nan(),
        "(0, -1, 1) is both present"
    );
    assert_eq!((w(INF) + w(INF) + w(-1e300)).to_f64(), INF);
    assert_eq!((w(NINF) + w(1e300)).to_f64(), NINF);
    assert_eq!((-w(INF)).specials(), (0, -1, 0));
    assert_eq!((-w(INF)).to_f64(), NINF);
    assert_eq!((-w(NINF)).to_f64(), INF);
    assert_eq!((w(INF) * -3i64).to_f64(), NINF);
    let mut t = w(INF);
    t -= w(INF);
    t -= w(INF);
    assert_eq!(t.specials(), (0, -1, 0));
    assert_eq!(t.to_f64(), NINF);
    // Projection is a read: the finite part is kept exactly underneath a special.
    let mut u = w(0.1) + w(0.2) + w(INF);
    assert_eq!(u.to_f64(), INF);
    u -= w(INF);
    assert_eq!(u, w(0.1) + w(0.2));
    assert_eq!(u.to_f64(), 0.1 + 0.2);
    // A finite part that overflows a double reads ∞ while `is_finite` stays true: `is_finite`
    // is about the counters, and the state is still an exact finite sum.
    let big = w(f64::MAX) + w(f64::MAX);
    assert!(big.is_finite());
    assert_eq!(big.to_f64(), INF);
    assert_eq!(big.try_to_f64(), None);
    // `try_to_f64` passes a special projection through: nothing was rounded.
    assert!(w(NAN).try_to_f64().unwrap().is_nan());
    assert_eq!(w(INF).try_to_f64(), Some(INF));
    assert_eq!((w(NINF) + w(0.1)).try_to_f64(), Some(NINF));
}

#[test]
#[allow(clippy::erasing_op)] // scaling by a literal zero is one of the cases under test
fn wide_group_laws_hold_with_specials_in_the_multiset() {
    let mut rng = Rng(14);
    let mut specials_seen = 0;
    for _ in 0..5_000 {
        let (a, b, c) = (w(rng.any()), w(rng.any()), w(rng.any()));
        specials_seen += usize::from(!a.is_finite());
        assert_eq!(a + b, b + a);
        assert_eq!((a + b) + c, a + (b + c));
        assert_eq!(a + Wide::ZERO, a);
        assert_eq!(a + (-a), Wide::ZERO);
        assert_eq!(a + b - b, a);
        assert_eq!(-(-a), a);
        assert_eq!(a - b, a + (-b));
        let mut aa = a;
        aa += b;
        aa -= b;
        assert_eq!(aa, a);
        // Integer scaling is repeated addition, on the limbs and the counters alike.
        assert_eq!(a * 3i64, a + a + a);
        assert_eq!(a * -1i64, -a);
        assert_eq!(a * 0i64, Wide::ZERO);
        assert_eq!(a * 2isize, a + a);
        assert_eq!((a + b) * 5i64, a * 5i64 + b * 5i64);
    }
    assert!(
        specials_seen > 200,
        "{specials_seen} specials in 5000 draws"
    );
}

#[test]
fn wide_folds_are_order_independent() {
    let mut rng = Rng(15);
    let xs: Vec<f64> = (0..2_000).map(|_| rng.any()).collect();
    let forward: Wide = xs.iter().map(|&x| w(x)).sum();
    let backward: Wide = xs.iter().rev().map(|&x| w(x)).sum();
    assert_eq!(forward, backward);
    let states: Vec<Wide> = xs.iter().map(|&x| w(x)).collect();
    let by_ref: Wide = states.iter().sum();
    assert_eq!(by_ref, forward);
    // Random partitions, merged in random order.
    for round in 0..50 {
        let mut parts = vec![Wide::ZERO; 1 + round % 7];
        for &x in &xs {
            let i = (rng.next() % parts.len() as u64) as usize;
            parts[i].deposit(x);
        }
        let mut merged = Wide::ZERO;
        while !parts.is_empty() {
            let i = (rng.next() % parts.len() as u64) as usize;
            merged += parts.swap_remove(i);
        }
        assert_eq!(merged, forward);
    }
    // Retracting every input lands on the zero, specials included.
    assert!(!forward.is_finite(), "2000 draws with no special; reseed");
    let mut back = forward;
    for &x in &xs {
        back -= w(x);
    }
    assert!(back.is_zero());
    // The f64 fold is not order independent on the same finite data, which is the point.
    let fs: Vec<f64> = (0..1_000).map(|_| rng.on_grid(-60, 60)).collect();
    let f: f64 = fs.iter().sum();
    let r: f64 = fs.iter().rev().sum();
    assert_ne!(
        f.to_bits(),
        r.to_bits(),
        "a 1000-term f64 fold happened to commute; reseed"
    );
    let fw: Wide = fs.iter().map(|&x| w(x)).sum();
    let rw: Wide = fs.iter().rev().map(|&x| w(x)).sum();
    assert_eq!(fw, rw);
}

#[test]
fn wide_order_is_numeric_then_lexicographic_on_the_counters() {
    let mut rng = Rng(16);
    for _ in 0..50_000 {
        let (a, b) = (rng.finite(), rng.finite());
        assert_eq!(
            w(a).cmp(&w(b)),
            a.partial_cmp(&b).unwrap(),
            "{a:e} vs {b:e}"
        );
        assert_eq!(w(a).partial_cmp(&w(b)), Some(w(a).cmp(&w(b))));
    }
    assert!(w(-5e-324) < Wide::ZERO);
    assert!(Wide::ZERO < w(5e-324));
    assert!(w(-f64::MAX) < w(5e-324));
    // The finite part decides first; equal finite parts fall to (n, p, q).
    assert!(w(NAN) < w(5e-324));
    assert!(w(-5e-324) < w(NAN));
    assert!(w(INF) < w(NAN));
    assert!(w(NINF) < w(INF));
    assert!(Wide::ZERO < w(NINF));
    assert!(-w(INF) < Wide::ZERO);
    assert_eq!(w(INF).cmp(&w(INF)), Ordering::Equal);
    // Structural Eq is numeric equality, and Hash agrees: two routes to one value are one state.
    assert_eq!(w(0.5) + w(0.25), w(0.75));
    assert_eq!(w(3.0) - w(1.0), w(1.0) + w(1.0));
    assert_eq!(hash(&(w(0.5) + w(0.25))), hash(&w(0.75)));
    assert_eq!(w(INF) + w(1.0) - w(1.0), w(INF));
    assert_eq!(hash(&(w(INF) + w(1.0) - w(1.0))), hash(&w(INF)));
    // Total: a sort of mixed states is consistent with cmp.
    let mut xs: Vec<Wide> = (0..200).map(|_| w(rng.any())).collect();
    xs.sort();
    for pair in xs.windows(2) {
        assert_ne!(pair[0].cmp(&pair[1]), Ordering::Greater);
    }
}

#[test]
#[should_panic(expected = "counter")]
fn wide_overflowing_a_special_counter_is_a_named_panic_in_every_build() {
    // An i64 counter at its maximum plus one more NaN: a named panic in every build, release
    // included, because a wrapped counter would project the wrong special.
    let _ = w(NAN) * i64::MAX + w(NAN);
}
