use float_abel::{Exact, EXACT_MAX_EXP};

fn e(x: f64) -> Exact {
    Exact::from(x)
}

fn add(xs: &[f64]) -> Result<Exact, float_abel::PastCap> {
    let mut acc = Exact::ZERO;
    for &x in xs {
        acc = acc.checked_add(&e(x))?;
    }
    Ok(acc)
}

fn show(label: &str, xs: &[f64]) {
    let f: f64 = xs.iter().sum();
    match add(xs) {
        Ok(s) => println!("{label:<44} f64 = {f:<24e} Exact ok, reads {:e}", s.to_f64()),
        Err(c) => println!("{label:<44} f64 = {f:<24e} Exact CAP {c:?}"),
    }
}

fn main() {
    println!("EXACT_MAX_EXP = {EXACT_MAX_EXP}");

    println!("\n-- every single f64 round-trips --");
    for x in [
        f64::MAX,
        -f64::MAX,
        f64::MIN_POSITIVE,
        f64::from_bits(1),
        0.1,
        -0.0,
    ] {
        let r = e(x).to_f64();
        println!("{x:<26e} -> {r:<26e} bits equal: {}", r.to_bits() == x.to_bits());
    }
    for x in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let w = e(x);
        println!("{x:<26} specials {:?} reads {}", w.specials(), w.to_f64());
    }

    println!("\n-- past f64's top: exact inside, +inf at read, retract restores --");
    let two_max = e(f64::MAX).checked_add(&e(f64::MAX)).unwrap();
    println!("Exact MAX+MAX reads {:e}", two_max.to_f64());
    let back = two_max.checked_sub(&e(f64::MAX)).unwrap();
    println!("Exact MAX+MAX-MAX reads {:e}, == MAX: {}", back.to_f64(), back == e(f64::MAX));
    let ff = f64::MAX + f64::MAX - f64::MAX;
    println!("f64   MAX+MAX-MAX = {ff:e}");
    let scaled = e(f64::MAX).checked_mul_i64(i64::MAX).unwrap();
    println!("Exact MAX * i64::MAX ok, reads {:e}", scaled.to_f64());

    println!("\n-- the 127-bit width on sums --");
    let p126 = 2f64.powi(126);
    let p127 = 2f64.powi(127);
    show("2^126 + 1  (span 127 bits)", &[p126, 1.0]);
    show("2^127 + 1  (span 128 bits)", &[p127, 1.0]);
    show("1e16 + 1 - 1e16", &[1e16, 1.0, -1e16]);
    show("1 + f64::EPSILON", &[1.0, f64::EPSILON]);
    for k in [6, 8, 10, 11, 12, 20] {
        let big = 10f64.powi(k);
        let small = 10f64.powi(-k);
        show(&format!("1e{k} + 1e-{k}"), &[big, small]);
    }
    show("1 + 1e-22", &[1.0, 1e-22]);
    show("1 + 1e-23", &[1.0, 1e-23]);
    show("1 + 2^-73 (one set bit)", &[1.0, 2f64.powi(-73)]);
    show("1 + 2^-126 (one set bit)", &[1.0, 2f64.powi(-126)]);
    show("1 + 2^-127 (one set bit)", &[1.0, 2f64.powi(-127)]);
    show("1 + smallest subnormal", &[1.0, f64::from_bits(1)]);
    show("MAX + smallest subnormal", &[f64::MAX, f64::from_bits(1)]);

    println!("\n-- the same multiset, two orders --");
    let big = 2f64.powi(1000);
    let tiny = 2f64.powi(-1000);
    show("(2^1000 - 2^1000) + 2^-1000", &[big, -big, tiny]);
    show("(2^1000 + 2^-1000) - 2^1000", &[big, tiny, -big]);
}
