// Binned<K> keeps between 1 and 33 bins: at least one to hold a value, and at most 33 so that the
// top bin's carry still lies inside Wide's 34 limbs. The const assertion on K refuses the rest
// when the zero is instantiated.
use float_abel::Binned;

fn main() {
    let _b = Binned::<0>::ZERO;
}
