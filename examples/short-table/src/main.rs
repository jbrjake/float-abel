//! The short table, measured.
//!
//! ```bash
//! cargo run --manifest-path examples/short-table/Cargo.toml
//! ```
//!
//! Prints `OUTPUT.md`: the comparison table of `Exact` and seven neighbouring crates over the
//! eight properties an incremental engine asks of a weight, every cell measured by running it.
// Until the rows and the control section land, parts of the trait, the columns and the renderer
// have no caller; the verdict row's commit removes this line.
#![allow(dead_code)]

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

/// The whole file: the header, the table, and the evidence for every row.
fn report() -> String {
    let rows = rows::all();
    let mut out = header();
    out.push('\n');
    out.push_str(&render::table(&rows));
    out.push_str(&render::evidence(&rows));
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
    fn the_report_carries_no_path_from_this_machine() {
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
}
