//! The crate's posture, as a bare `cargo test` observes it.

/// The crate lists itself as a dev-dependency with the test features on, which is the one Cargo
/// spelling that turns them on for a bare `cargo test` with no flag. `dbsp` stays out of that list
/// because it needs its own toolchain, so it is not asserted here. If the dev-dependency goes,
/// every feature-gated test silently stops compiling, and this is the test that notices.
#[test]
fn a_bare_cargo_test_turns_the_test_features_on() {
    let on = [
        ("std", cfg!(feature = "std")),
        ("serde", cfg!(feature = "serde")),
        ("differential", cfg!(feature = "differential")),
        ("laws", cfg!(feature = "laws")),
        ("simd", cfg!(feature = "simd")),
        ("arbitrary", cfg!(feature = "arbitrary")),
    ];
    let off: Vec<&str> = on.iter().filter(|(_, b)| !b).map(|(n, _)| *n).collect();
    assert!(
        off.is_empty(),
        "features off under a bare `cargo test`: {off:?}"
    );
}
