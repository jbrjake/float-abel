// The fragment the Binned<3> case fails on, with the engine weight in its place: `Exact` is a
// `DBWeight` behind the `dbsp` feature, so a weighted batch takes it. This case also proves that
// the harness builds with that feature on, so the Binned case fails for the reason it names and
// not for a missing feature.
use dbsp::dynamic::DynWeightTyped;
use dbsp::utils::Tup2;
use dbsp::OrdIndexedWSet;
use float_abel::Exact;

fn main() {
    let _batch = OrdIndexedWSet::<u32, u32, Exact, DynWeightTyped<Exact>>::from_tuples(
        (),
        vec![Tup2(Tup2(1, 10), Exact::from(0.1))],
    );
}
