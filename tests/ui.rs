//! The compile-fail cases, in one trybuild harness for the crate: each `tests/ui/*.rs` states the
//! `const` assertion it trips, and its `.stderr` is rustc's wording for it. The assertions are
//! post-monomorphisation, so the harness needs a `pass` case beside the `compile_fail` ones: with
//! only compile-fail cases trybuild runs `cargo check`, which never instantiates an impl and so
//! never evaluates an assertion. `TRYBUILD=overwrite cargo test ui_` rewrites the `.stderr` files
//! when stable's text moves.

/// `Wide: From<Fixed<LSB, LIMBS>>` is pinned to the pairs it is exact for, and
/// `Fixed::to_bytes` to the one array length its encoding has.
#[test]
fn ui_const_assertions_refuse_the_pairs_and_lengths_they_name() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/fixed256_into_wide.rs");
    t.compile_fail("tests/ui/fixed_below_wides_unit.rs");
    t.compile_fail("tests/ui/fixed_wider_than_wide.rs");
    t.compile_fail("tests/ui/fixed_to_bytes_wrong_length.rs");
}
