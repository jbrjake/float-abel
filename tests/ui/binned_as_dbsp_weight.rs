// Binned<K> is not a group, so it implements none of DBSP's weight traits, and a weighted batch
// cannot take it: `OrdIndexedWSet` bounds its weight on `DBWeight`, a monoid with the storage
// traits, and Binned has neither the monoid (`&Binned + &Binned`, `HasZero`) nor the storage
// traits (`SizeOf`, `IsNone`, `SupportsRoaring`, rkyv's `Serialize`). The same fragment with
// `Exact` is the pass case beside this one.
use dbsp::dynamic::DynWeightTyped;
use dbsp::utils::Tup2;
use dbsp::OrdIndexedWSet;
use float_abel::Binned;

fn main() {
    let _batch = OrdIndexedWSet::<u32, u32, Binned<3>, DynWeightTyped<Binned<3>>>::from_tuples(
        (),
        vec![Tup2(Tup2(1, 10), Binned::<3>::ZERO)],
    );
}
