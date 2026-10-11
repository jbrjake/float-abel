// Fixed<-1076, 20> has a grid two bits below Wide's unit of 2^-1074, so a value with its lowest
// bit set has no Wide image: `Wide: From<Fixed<-1076, 20>>` cannot be exact, and the const
// assertion on `(LSB, LIMBS)` refuses it when the impl is instantiated.
use float_abel::{Fixed, Wide};

fn main() {
    let f = Fixed::<-1076, 20>::ZERO;
    let _w = Wide::from(f);
}
