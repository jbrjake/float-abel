// Fixed256's encoding is 33 bytes: the version byte then four limbs. The array `to_bytes` returns
// is pinned to that length by a const assertion, so a buffer of any other size is refused at
// compile time rather than filled short or long at run time.
use float_abel::Fixed256;

fn main() {
    let f = Fixed256::ZERO;
    let _bytes: [u8; 32] = f.to_bytes();
}
