// Binned<K> is not a group, so it implements none of differential-dataflow's weight traits, and a
// collection cannot take it as a diff: `new_collection` bounds the diff on `Semigroup`, which is
// the first thing an engine asks of a weight and the first thing Binned lacks. The same fragment
// with `Exact` is the pass case beside this one.
use differential_dataflow::input::Input;
use float_abel::Binned;

fn main() {
    timely::execute_directly(|worker| {
        worker.dataflow::<u32, _, _>(|scope| {
            let (_input, _rows) = scope.new_collection::<u32, Binned<3>>();
        });
    });
}
