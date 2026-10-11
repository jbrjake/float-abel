// Fixed256's grid, 2^-96, is above Wide's unit, and its 256 bits land 978 bits up Wide's 2176, so
// `Wide: From<Fixed256>` is exact and the const assertion on `(LSB, LIMBS)` lets it compile.
use float_abel::{Fixed256, Wide};

fn main() {
    let mut f = Fixed256::ZERO;
    f.try_deposit(-1.5).unwrap();
    let w = Wide::from(f);
    assert_eq!(w.to_f64(), -1.5);
}
