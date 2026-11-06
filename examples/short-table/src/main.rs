//! The short table, measured.
//!
//! ```bash
//! cargo run --manifest-path examples/short-table/Cargo.toml
//! ```
//!
//! Prints `OUTPUT.md`: the comparison table of `Exact` and seven neighbouring crates over the
//! eight properties an incremental engine asks of a weight, every cell measured by running it.

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

/// The column every line of prose wraps at.
const WIDTH: usize = 100;

/// `words` laid out in lines of at most [`WIDTH`] characters, a space between neighbours and a
/// newline at the end. A word is never split, so a caller that must keep a phrase on one line
/// passes it as one word.
fn flow<'a>(words: impl IntoIterator<Item = &'a str>) -> String {
    let mut out = String::new();
    let mut line = String::new();
    for word in words {
        if !line.is_empty() && line.chars().count() + 1 + word.chars().count() > WIDTH {
            out.push_str(&line);
            out.push('\n');
            line.clear();
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    out.push_str(&line);
    out.push('\n');
    out
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

fn main() {
    print!("{}", header());
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
    fn flow_wraps_at_the_width_and_keeps_every_word() {
        let words: Vec<String> = (0..60).map(|i| format!("word{i}")).collect();
        let wrapped = flow(words.iter().map(String::as_str));
        assert!(wrapped.lines().count() > 1);
        assert!(
            wrapped.lines().all(|l| l.chars().count() <= WIDTH),
            "{wrapped}"
        );
        assert_eq!(wrapped.split_whitespace().collect::<Vec<_>>(), words);
        assert!(wrapped.ends_with('\n'));
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
        assert!(h.lines().all(|l| l.chars().count() <= WIDTH), "{h}");
    }

    #[test]
    fn header_never_names_float_abel_version() {
        let h = header();
        assert!(!h.contains(env!("CARGO_PKG_VERSION")), "{h}");
        assert!(!h.contains("float-abel 0."), "{h}");
    }
}
