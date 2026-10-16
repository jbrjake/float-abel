// Fixed<1, 4> has a grid of two, so no odd integer is on it: `Fixed<1, 4>: From<i8>`, which
// differential-dataflow's `count_core` and `distinct_core` put out as one, cannot be exact, and
// the const assertion on `(LSB, LIMBS)` refuses it when the impl is instantiated.
use float_abel::Fixed;

fn main() {
    let _one = Fixed::<1, 4>::from(1i8);
}
