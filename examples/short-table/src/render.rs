//! The markdown the program prints: the table, then a section of evidence for every row.

use std::fmt::Write as _;

use crate::column::{Outcome, Shortfall};
use crate::columns::NAMES;

/// The column every line of prose wraps at.
pub(crate) const WIDTH: usize = 100;

/// One measured cell: whether the column passed the row, and the evidence that says why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Cell {
    /// Every probe of the row passed.
    pub(crate) pass: bool,
    /// For a pass, what was read; for a failure, every shortfall, each as its kind.
    pub(crate) evidence: String,
}

impl Cell {
    /// The cell a row's probes add up to. It passes only if every probe did, and then `passed`
    /// turns the probes' readings into one line; otherwise the evidence is every shortfall, the
    /// same one printed once.
    pub(crate) fn from_probes(
        probes: &[Outcome<String>],
        passed: impl FnOnce(&[&str]) -> String,
    ) -> Cell {
        let mut shortfalls: Vec<String> = Vec::new();
        for s in probes.iter().filter_map(|p| p.as_ref().err()) {
            let line = s.to_string();
            if !shortfalls.contains(&line) {
                shortfalls.push(line);
            }
        }
        if shortfalls.is_empty() {
            let readings: Vec<&str> = probes.iter().filter_map(|p| p.as_deref().ok()).collect();
            Cell {
                pass: true,
                evidence: format!("ran: {}", passed(&readings)),
            }
        } else {
            Cell {
                pass: false,
                evidence: shortfalls.join("; "),
            }
        }
    }
}

/// One row of the table: its name, the paragraph that says what was asked, and a cell per column
/// in table order.
#[derive(Debug, Clone)]
pub(crate) struct RowReport {
    /// The property, as the table names it.
    pub(crate) name: &'static str,
    /// What the probes ask, for the evidence section.
    pub(crate) intro: String,
    /// `(column name, cell)` in table order.
    pub(crate) cells: Vec<(&'static str, Cell)>,
}

/// `yes` or `no`.
pub(crate) fn yes_no(pass: bool) -> &'static str {
    if pass {
        "yes"
    } else {
        "no"
    }
}

/// The table: the header from the column names, then one line per row.
pub(crate) fn table(rows: &[RowReport]) -> String {
    let mut out = String::from("| |");
    for n in NAMES {
        let _ = write!(out, " {n} |");
    }
    out.push_str("\n|---|");
    for _ in NAMES {
        out.push_str("---|");
    }
    out.push('\n');
    for r in rows {
        let _ = write!(out, "| {} |", r.name);
        for (_, c) in &r.cells {
            let _ = write!(out, " {} |", yes_no(c.pass));
        }
        out.push('\n');
    }
    out
}

/// The evidence: a section per row, its intro, then a table of every column's cell and why.
pub(crate) fn evidence(rows: &[RowReport]) -> String {
    if rows.is_empty() {
        return String::new();
    }
    let mut out = String::from("\n## Evidence\n");
    for r in rows {
        let _ = write!(out, "\n### {}\n\n", r.name);
        out.push_str(&paragraph(&r.intro));
        out.push_str("\n| column | cell | evidence |\n|---|---|---|\n");
        for (name, c) in &r.cells {
            let _ = writeln!(out, "| {name} | {} | {} |", yes_no(c.pass), c.evidence);
        }
    }
    out
}

/// `text` re-flowed on spaces into lines of at most [`WIDTH`] characters, ending in a newline.
pub(crate) fn paragraph(text: &str) -> String {
    flow(text.split_whitespace())
}

/// `words` laid out in lines of at most [`WIDTH`] characters, a space between neighbours and a
/// newline at the end. A word is never split, so a caller that must keep a phrase on one line
/// passes it as one word.
pub(crate) fn flow<'a>(words: impl IntoIterator<Item = &'a str>) -> String {
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

/// A probe's `Err` for a wrong reading, so rows write `wrong(format!(…))`.
pub(crate) fn wrong<T>(text: String) -> Outcome<T> {
    Err(Shortfall::Wrong(text))
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn a_cell_passes_only_when_every_probe_did_and_prints_each_shortfall_once() {
        let all_ok: Vec<Outcome<String>> = vec![Ok("a".into()), Ok("b".into())];
        let c = Cell::from_probes(&all_ok, |r| r.join(", "));
        assert_eq!(
            c,
            Cell {
                pass: true,
                evidence: "ran: a, b".into()
            }
        );
        let mixed: Vec<Outcome<String>> = vec![
            Ok("a".into()),
            Err(Shortfall::Refused("x".into())),
            Err(Shortfall::Refused("x".into())),
            Err(Shortfall::Absent("y".into())),
        ];
        let c = Cell::from_probes(&mixed, |_| unreachable!());
        assert_eq!(
            c,
            Cell {
                pass: false,
                evidence: "refused: x; absent: y".into()
            }
        );
    }

    #[test]
    fn the_table_header_names_the_eight_columns_in_order() {
        let t = table(&[]);
        assert_eq!(
            t,
            "| | float-abel | bitrep | xsum | fsum | accurate | num-rational | bigdecimal | fixed |\n\
             |---|---|---|---|---|---|---|---|---|\n"
        );
    }

    #[test]
    fn a_row_prints_its_cells_and_its_evidence_section() {
        let row = RowReport {
            name: "a property",
            intro: "What was asked.".into(),
            cells: NAMES
                .iter()
                .enumerate()
                .map(|(i, n)| {
                    (
                        *n,
                        Cell {
                            pass: i == 0,
                            evidence: format!("e{i}"),
                        },
                    )
                })
                .collect(),
        };
        let t = table(std::slice::from_ref(&row));
        assert!(
            t.ends_with("| a property | yes | no | no | no | no | no | no | no |\n"),
            "{t}"
        );
        let e = evidence(std::slice::from_ref(&row));
        assert!(e.starts_with("\n## Evidence\n\n### a property\n\nWhat was asked.\n\n| column | cell | evidence |\n|---|---|---|\n| float-abel | yes | e0 |\n"), "{e}");
        assert!(e.ends_with("| fixed | no | e7 |\n"), "{e}");
        assert_eq!(evidence(&[]), "");
    }
}
