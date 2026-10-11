// Fixed<-96, 20> is on a grid Wide holds, but its 1280 bits shifted 978 up need 2258, past Wide's
// 2176: a state near its ceiling has no Wide image, so the const assertion refuses the impl.
use float_abel::{Fixed, Wide};

fn main() {
    let f = Fixed::<-96, 20>::ZERO;
    let _w = Wide::from(f);
}
