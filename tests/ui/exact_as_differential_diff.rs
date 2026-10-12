// The fragment the Binned<3> case fails on, with the engine weight in its place: `Exact` is a
// group, implements the weight traits behind the `differential` feature, and opens a collection.
// This case also proves that the harness builds with that feature on, so the Binned case fails for
// the reason it names and not for a missing feature.
use differential_dataflow::input::Input;
use float_abel::Exact;

fn main() {
    timely::execute_directly(|worker| {
        worker.dataflow::<u32, _, _>(|scope| {
            let (_input, _rows) = scope.new_collection::<u32, Exact>();
        });
    });
}
