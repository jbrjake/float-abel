//! The short table, measured: `Exact` and seven neighbouring crates run through the eight
//! properties an incremental engine asks of a weight, and the comparison table that falls out.
//!
//! ```bash
//! cargo run --manifest-path examples/short-table/Cargo.toml > examples/short-table/OUTPUT.md
//! ```
//!
//! An incremental engine keeps a running `SUM` and applies changes to it: an insert adds a
//! weight, a delete adds its negation, and a background merge adds any subset in any order. The
//! type that carries the weight is asked for eight things, and each is a row here. It must hold
//! every `f64` exactly, because the weight *is* the column's value. One number must be one bit
//! pattern however it was reached, because consolidation matches diffs structurally. Partial sums
//! from different workers must merge. A state must have a negative that exists without its
//! inputs, because `negate` runs inside `iterate`, `antijoin` and `reduce_abelian`, and a −x can
//! arrive before its +x. `==`, `Hash` and `is_zero` must mean the number, because builders dedup
//! by `==`, `is_zero` deletes an update, and records route to workers by `Hash`. Products must be
//! exact, because `explode` multiplies a value by its diff and `join` distributes a product over a
//! sum. NaN and ±∞ must be retractable, because `f64` includes them and the `SUM` has to recover
//! when the NaN row is deleted. And the arithmetic underneath must be right, which every other
//! row assumes. A column is usable as the weight only if every cell in it is yes; the bold last
//! row is that conjunction, computed, never written.
//!
//! Every cell is measured. A probe builds a state from a recipe such as `+0.5 +0.5` (insert 0.5
//! twice) or `+0.1 −0.1` (insert 0.1, then retract it through the crate's own inverse), reads it
//! back, and compares. Every input's exact answer is itself a double, so there is no oracle and
//! no rounding argument: `1e100 + 1 − 1e100` is 1, ten `0.1`s less one is `2⁻⁵⁴`, and the merge
//! probe totals `15/8`. A cell that is not yes says why, as one of four kinds: `ran:` the probe
//! ran and read what follows; `refused:` the crate's API declined, an `Option::None` or a
//! `false`; `panicked:` the crate panicked, with its message and where; `absent:` the API does
//! not exist, which the compiler asserts with `impls!`, so a release that adds the trait fails
//! this build instead of leaving a stale no. Each column uses its crate's strongest API, so a no
//! is the crate's limit and never the adapter's choice. Below the evidence, plain `f64` runs the
//! same probes as a control, and in the tests `Exact` runs them with one property broken per row,
//! so every probe is shown to fail on something.
//!
//! The output is byte-identical in debug and release and across architectures: no timings, hash
//! values, paths or versions beyond the neighbours' pins, which the header reads from this
//! package's lockfile. `OUTPUT.md` beside this package is that output, committed; both gates
//! re-run the program and fail when the two differ (`scripts/check-short-table.sh`), CI's gate
//! job on `x86_64` Linux against the copy blessed on `aarch64` macOS.

mod column;
mod columns;
mod guard;
mod render;
mod rows;

use render::flow;

/// This package's lockfile, so the header names the exact neighbour versions the run measured
/// without a second copy of them in the source.
const LOCK: &str = include_str!("../Cargo.lock");

/// The neighbours in the table's column order.
const NEIGHBOURS: [&str; 7] = [
    "bitrep",
    "xsum",
    "fsum",
    "accurate",
    "num-rational",
    "bigdecimal",
    "fixed",
];

/// What the table is, between it and the evidence.
const RULE: &str = "Each row is a property an incremental engine needs of its weight, \
                    DESIGN.md § What an incremental engine asks of a weight; a column is usable \
                    only if every cell in it is yes, and the bold row is that conjunction, \
                    computed. Every cell was measured by running it, and the evidence for each \
                    follows: `ran` says what was read, `refused` that the crate's API declined, \
                    `panicked` where the crate panicked, and `absent` that the API does not \
                    exist, which the compiler asserts.";

/// The version `Cargo.lock` records for `name`: the `version = "…"` line that follows its
/// `name = "…"` line.
fn locked_version(name: &str) -> &'static str {
    let mut lines = LOCK.lines();
    while let Some(line) = lines.next() {
        if line
            .strip_prefix("name = \"")
            .and_then(|l| l.strip_suffix('"'))
            == Some(name)
        {
            if let Some(v) = lines
                .next()
                .and_then(|l| l.strip_prefix("version = \""))
                .and_then(|l| l.strip_suffix('"'))
            {
                return v;
            }
        }
    }
    panic!("{name} is not in examples/short-table/Cargo.lock")
}

/// The file's title and provenance: what wrote it, and which versions it measured. float-abel is
/// named as this checkout, never by version, because every release moves the version and would
/// stale the committed copy for nothing.
fn header() -> String {
    let versions: Vec<String> = NEIGHBOURS
        .iter()
        .map(|n| format!("{n} {}", locked_version(n)))
        .collect();
    let (last, rest) = versions.split_last().expect("seven neighbours");
    let (penultimate, commaed) = rest.split_last().expect("seven neighbours");
    // Each "name version" pair is one word to the wrapper, so a line break never parts them.
    let prose =
        "Written by `cargo run --manifest-path examples/short-table/Cargo.toml`, from this \
                 checkout of float-abel. The neighbours are the versions this package's \
                 Cargo.lock pins:";
    let listed: Vec<String> = commaed
        .iter()
        .map(|v| format!("{v},"))
        .chain([penultimate.clone(), "and".to_owned(), format!("{last}.")])
        .collect();
    let provenance = flow(
        prose
            .split_whitespace()
            .chain(listed.iter().map(String::as_str)),
    );
    format!("# The short table, measured\n\n{provenance}")
}

/// The whole file: the header, the table with its verdict, what the table is, the evidence for
/// every row, and the control.
fn report() -> String {
    let rows = rows::all();
    let mut out = header();
    out.push('\n');
    out.push_str(&render::table(&rows));
    out.push('\n');
    out.push_str(&render::paragraph(RULE));
    out.push_str(&render::evidence(&rows));
    out.push_str(&render::control(&rows::control()));
    out
}

fn main() {
    guard::install();
    print!("{}", report());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_neighbour_is_locked_at_its_pin() {
        // The pins in Cargo.toml, read back from the lockfile the build used.
        assert_eq!(locked_version("bitrep"), "0.5.1");
        assert_eq!(locked_version("xsum"), "0.1.6");
        assert_eq!(locked_version("fsum"), "0.1.4");
        assert_eq!(locked_version("accurate"), "0.4.1");
        assert_eq!(locked_version("num-rational"), "0.4.2");
        assert_eq!(locked_version("bigdecimal"), "0.4.11");
        assert_eq!(locked_version("fixed"), "1.31.0");
    }

    #[test]
    fn header_keeps_each_neighbour_and_its_version_on_one_line() {
        let h = header();
        for n in NEIGHBOURS {
            let pair = format!("{n} {}", locked_version(n));
            assert!(
                h.lines().any(|l| l.contains(&pair)),
                "{pair} is split in\n{h}"
            );
        }
        assert!(h.lines().all(|l| l.chars().count() <= render::WIDTH), "{h}");
    }

    #[test]
    fn header_never_names_float_abel_version() {
        let h = header();
        assert!(!h.contains(env!("CARGO_PKG_VERSION")), "{h}");
        assert!(!h.contains("float-abel 0."), "{h}");
    }

    #[test]
    fn the_report_carries_no_path_from_this_machine_and_wraps_its_prose() {
        let r = report();
        for leak in ["/Users/", "/home/", "/root/", ".cargo/registry", "C:\\"] {
            assert!(!r.contains(leak), "{leak} in\n{r}");
        }
        assert!(
            r.lines()
                .all(|l| l.chars().count() <= render::WIDTH || l.starts_with('|')),
            "{r}"
        );
    }

    #[test]
    fn the_report_has_the_table_its_verdict_the_evidence_and_the_control_in_order() {
        let r = report();
        let at = |needle: &str| r.find(needle).unwrap_or_else(|| panic!("{needle} missing"));
        let table = at("| | float-abel | bitrep |");
        let verdict = at("| **usable as the weight** | **yes** |");
        let evidence = at("\n## Evidence\n");
        let control = at("\n## Control: plain f64\n");
        assert!(table < verdict && verdict < evidence && evidence < control);
        assert_eq!(r.matches("\n### ").count(), 8);
    }
}
