//! The compile-fail cases, in one trybuild harness for the crate: each `tests/ui/*.rs` states the
//! `const` assertion it trips, and its `.stderr` is rustc's wording for it. The assertions are
//! post-monomorphisation, so the harness needs a `pass` case beside the `compile_fail` ones: with
//! only compile-fail cases trybuild runs `cargo check`, which never instantiates an impl and so
//! never evaluates an assertion. `TRYBUILD=overwrite cargo test ui_` rewrites the `.stderr` files
//! when stable's text moves.

/// `Wide: From<Fixed<LSB, LIMBS>>` is pinned to the pairs it is exact for, and
/// `Fixed::to_bytes` to the one array length its encoding has.
#[test]
#[cfg_attr(miri, ignore = "trybuild spawns cargo, which miri cannot")]
fn ui_const_assertions_refuse_the_pairs_and_lengths_they_name() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/fixed256_into_wide.rs");
    t.compile_fail("tests/ui/fixed_below_wides_unit.rs");
    t.compile_fail("tests/ui/fixed_wider_than_wide.rs");
    t.compile_fail("tests/ui/fixed_to_bytes_wrong_length.rs");
}

/// `Binned<K>` implements none of differential-dataflow's weight traits, so a collection refuses
/// it as a diff where it takes `Exact`, and `K` is pinned to `1..=33`. The `binned_` filter of the
/// task verify reaches this through its name.
#[test]
#[cfg_attr(miri, ignore = "trybuild spawns cargo, which miri cannot")]
fn ui_binned_is_no_engine_weight_and_k_is_bounded() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/exact_as_differential_diff.rs");
    t.compile_fail("tests/ui/binned_as_differential_diff.rs");
    t.compile_fail("tests/ui/binned_zero_bins.rs");
}

/// `Binned<K>` is no DBSP weight either: `OrdIndexedWSet` bounds its weight on `DBWeight`, which
/// `Exact` meets behind the `dbsp` feature and `Binned` never will. Under the feature only, so
/// the harness can name `dbsp`; the `dbsp_` filter of the task verify reaches it through its name.
#[cfg(feature = "dbsp")]
#[test]
#[cfg_attr(miri, ignore = "trybuild spawns cargo, which miri cannot")]
fn ui_dbsp_binned_is_no_dbsp_weight() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/exact_as_dbsp_weight.rs");
    t.compile_fail("tests/ui/binned_as_dbsp_weight.rs");
}
