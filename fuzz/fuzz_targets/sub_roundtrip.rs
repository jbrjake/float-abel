//! `a + b − b == a` bit for bit on every exact tier, with the specials in, DESIGN.md § Hardening:
//! `Exact` over any two canonical words through the checked operators, `Wide` and `Fixed256` over
//! two multisets of doubles through the operators, and every value's bytes round-tripping.

#![no_main]

use arbitrary::Arbitrary;
use float_abel::{Exact, Fixed256, Wide};
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct Input {
    a: Exact,
    b: Exact,
    xs: Vec<f64>,
    ys: Vec<f64>,
}

fn wide(xs: &[f64]) -> Wide {
    let mut w = Wide::ZERO;
    for &x in xs {
        w.deposit(x);
    }
    w
}

/// The inputs `Fixed256` holds, folded; the rest skipped.
fn fixed(xs: &[f64]) -> Fixed256 {
    let mut f = Fixed256::ZERO;
    for &x in xs {
        let _ = f.try_deposit(x);
    }
    f
}

fuzz_target!(|input: Input| {
    let Input { a, b, xs, ys } = input;

    // Exact: the checked operators agree with each other and with the operators where they hold.
    assert_eq!(
        a.checked_add(&b),
        b.checked_add(&a),
        "commutative, cap included"
    );
    if let Ok(s) = a.checked_add(&b) {
        assert_eq!(s.checked_sub(&b), Ok(a.clone()), "{a:?} + {b:?} - {b:?}");
        assert_eq!(s.checked_sub(&a), Ok(b.clone()), "{a:?} + {b:?} - {a:?}");
        assert_eq!(&s - &b, a);
        assert_eq!(&s + &Exact::ZERO, s);
        // A sum past the width is the heap form, which crosses as its serde bytes; the compact
        // encoding is the compact form's.
        if s.is_compact() {
            assert_eq!(Exact::from_bytes(&s.to_bytes()), Ok(s.clone()));
        }
        assert_eq!(
            postcard::from_bytes::<Exact>(&postcard::to_allocvec(&s).unwrap()),
            Ok(s.clone())
        );
    }
    assert_eq!(-(-&a), a);
    assert_eq!(a.checked_sub(&a), Ok(Exact::ZERO));
    assert_eq!(&a + &Exact::ZERO, a);
    assert_eq!(a.checked_mul_i64(1), Ok(a.clone()));
    assert_eq!(a.checked_mul_i64(-1), Ok(-&a));
    assert_eq!(Exact::from_bytes(&a.to_bytes()), Ok(a.clone()));
    assert_eq!(a.cmp(&b).reverse(), b.cmp(&a));
    assert_eq!(
        a == b,
        a.to_bytes() == b.to_bytes(),
        "structural Eq is the bytes"
    );

    // Wide over the two multisets.
    let (wa, wb) = (wide(&xs), wide(&ys));
    assert_eq!(wa + wb - wb, wa);
    assert_eq!(wa + wb, wb + wa);
    assert_eq!(-(-wa), wa);
    assert_eq!(wa + -wa, Wide::ZERO, "the inverse");
    assert_eq!(wa + Wide::ZERO, wa);
    assert_eq!(wa * -1i64, -wa);
    assert_eq!(Wide::from_bytes(&(wa + wb).to_bytes()), Ok(wa + wb));

    // Fixed256 over the inputs it holds.
    let (fa, fb) = (fixed(&xs), fixed(&ys));
    assert_eq!(fa + fb - fb, fa);
    assert_eq!(fa + fb, fb + fa);
    assert_eq!(-(-fa), fa);
    assert_eq!(fa + -fa, Fixed256::ZERO, "the inverse");
    assert_eq!(fa * -1i64, -fa);
    let bytes: [u8; Fixed256::BYTES] = (fa + fb).to_bytes();
    assert_eq!(Fixed256::from_bytes(&bytes), Ok(fa + fb));
});
