//! The compile-fail cases, in one trybuild harness for the crate: each `tests/ui/*.rs` states the
//! `const` assertion it trips, and its `.stderr` is rustc's wording for it. The assertions are
//! post-monomorphisation, so the harness needs a `pass` case beside the `compile_fail` ones: with
//! only compile-fail cases trybuild runs `cargo check`, which never instantiates an impl and so
//! never evaluates an assertion.
//!
//! The `.stderr` files match one compiler: stable [`BLESSED_RUSTC`] with `rust-src` installed,
//! because rustc rewords a diagnostic between releases and renders core's source inside one only
//! when the component is there (a bare `= note:` line otherwise). On any other compiler each
//! harness runs its `pass` cases alone and skips the compile-fail ones, saying so on stderr
//! (`--nocapture` shows it); scripts/regression.sh re-runs the topic with that output kept and
//! echoes the line, and ci.yml asserts the wording on the blessed compiler. `TRYBUILD=overwrite
//! cargo +1.99.0 test --test main ui_` re-blesses the files there; a new release is a bump of the
//! constant, ci.yml and CLAUDE.md § Commands together.

use std::path::Path;
use std::process::Command;

/// The stable release whose wording the `.stderr` files hold, as `major.minor`.
const BLESSED_RUSTC: &str = "1.99";

/// `rustc <args>`'s stdout, through the `rustc` cargo and trybuild resolve: `RUSTC` when cargo
/// sets it, else the one on `PATH`, which rustup routes by the same `RUSTUP_TOOLCHAIN` as the
/// `cargo` trybuild spawns. So `cargo +1.96.1 test` sees 1.96.1 here.
fn rustc(args: &[&str]) -> String {
    let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
    let out = Command::new(rustc).args(args).output().expect("rustc runs");
    assert!(
        out.status.success(),
        "rustc {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("rustc prints UTF-8")
}

/// Whether the compiler running this harness is the one the `.stderr` files were blessed on:
/// stable [`BLESSED_RUSTC`], any patch, with `rust-src`. Says why not on stderr otherwise.
fn wording_is_blessed() -> bool {
    let version = rustc(&["-vV"]);
    let release = version
        .lines()
        .find_map(|line| line.strip_prefix("release: "))
        .expect("rustc -vV prints a release line");
    // `1.99.0` is blessed; `1.99.0-nightly`, `1.99.0-beta.3` and `1.100.0` are not.
    let blessed_release = release
        .strip_prefix(BLESSED_RUSTC)
        .and_then(|rest| rest.strip_prefix('.'))
        .is_some_and(|patch| !patch.is_empty() && patch.bytes().all(|b| b.is_ascii_digit()));
    let sysroot = rustc(&["--print", "sysroot"]);
    let has_src = Path::new(sysroot.trim())
        .join("lib/rustlib/src/rust/library/core/src/panic.rs")
        .is_file();
    if blessed_release && has_src {
        return true;
    }
    let src = if has_src { "with" } else { "without" };
    eprintln!(
        "ui: skipping the compile-fail cases: rustc {release} {src} rust-src is not the compiler \
         tests/ui's .stderr files are blessed on, stable {BLESSED_RUSTC} with rust-src"
    );
    false
}

/// `Wide: From<Fixed<LSB, LIMBS>>` is pinned to the pairs it is exact for, `Fixed::to_bytes` to
/// the one array length its encoding has, and `Fixed<LSB, LIMBS>: From<i8>` to the windows that
/// hold every `i8`.
#[test]
#[cfg_attr(miri, ignore = "trybuild spawns cargo, which miri cannot")]
fn ui_const_assertions_refuse_the_pairs_and_lengths_they_name() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/fixed256_into_wide.rs");
    if !wording_is_blessed() {
        return;
    }
    t.compile_fail("tests/ui/fixed_below_wides_unit.rs");
    t.compile_fail("tests/ui/fixed_wider_than_wide.rs");
    t.compile_fail("tests/ui/fixed_to_bytes_wrong_length.rs");
    t.compile_fail("tests/ui/fixed_from_i8_above_one.rs");
}

/// `Binned<K>` implements none of differential-dataflow's weight traits, so a collection refuses
/// it as a diff where it takes `Exact`, and `K` is pinned to `1..=33`. The `binned_` filter of the
/// task verify reaches this through its name.
#[test]
#[cfg_attr(miri, ignore = "trybuild spawns cargo, which miri cannot")]
fn ui_binned_is_no_engine_weight_and_k_is_bounded() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/exact_as_differential_diff.rs");
    if !wording_is_blessed() {
        return;
    }
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
    if !wording_is_blessed() {
        return;
    }
    t.compile_fail("tests/ui/binned_as_dbsp_weight.rs");
}
