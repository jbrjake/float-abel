//! Fixed256's per-value check at the edges DESIGN.md § Exactness bounds states: |x| < 2^95, and
//! on the 2^-96 grid, which every double at or above 2^-44 is.

use float_abel::Fixed256;

fn main() {
    let cases = [
        ("0.1", 0.1),
        ("1e-12", 1e-12),
        ("2^-44", 2f64.powi(-44)),
        ("2^-44 next below", f64::from_bits(2f64.powi(-44).to_bits() - 1)),
        ("1e-14", 1e-14),
        ("2^-96", 2f64.powi(-96)),
        ("2^-97", 2f64.powi(-97)),
        ("3.9e28", 3.9e28),
        ("2^95 next below", f64::from_bits(2f64.powi(95).to_bits() - 1)),
        ("2^95", 2f64.powi(95)),
        ("1e30", 1e30),
        ("NaN", f64::NAN),
        ("inf", f64::INFINITY),
    ];
    for (label, x) in cases {
        match Fixed256::try_from(x) {
            Ok(f) => println!("{label:<18} {x:<24e} fits, reads {:e}", f.to_f64()),
            Err(e) => println!("{label:<18} {x:<24e} refused: {e:?}"),
        }
        let q = Fixed256::quantize(x);
        if q.to_bits() != x.to_bits() {
            println!("{:<18} {:<24} quantize -> {q:e}", "", "");
        }
    }
}
