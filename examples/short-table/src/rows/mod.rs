//! One file per row: the eight properties, each a set of probes run over every column.

pub(crate) mod bits;
pub(crate) mod correct;
pub(crate) mod equality;
pub(crate) mod holds;
pub(crate) mod merge;
pub(crate) mod negatives;
pub(crate) mod products;
pub(crate) mod specials;

use crate::column::Column;
use crate::columns::control::Control;
use crate::columns::{Accurate, BigDec, Bitrep, Fixed, FloatAbel, Fsum, NumRational, Xsum};
use crate::render::{Cell, RowReport};

/// One property an engine asks of a weight: what it asks, and how a column is measured on it.
pub(crate) trait Row {
    /// The property, as the table names it.
    const NAME: &'static str;
    /// What the probes ask and why the engine needs it, for the evidence section.
    fn intro() -> String;
    /// The column's cell: every probe run, the shortfalls collected.
    fn cell<C: Column>() -> Cell;
}

/// The row measured over the eight columns, in table order.
pub(crate) fn measure<R: Row>() -> RowReport {
    RowReport {
        name: R::NAME,
        intro: R::intro(),
        cells: vec![
            (FloatAbel::NAME, R::cell::<FloatAbel>()),
            (Bitrep::NAME, R::cell::<Bitrep>()),
            (Xsum::NAME, R::cell::<Xsum>()),
            (Fsum::NAME, R::cell::<Fsum>()),
            (Accurate::NAME, R::cell::<Accurate>()),
            (NumRational::NAME, R::cell::<NumRational>()),
            (BigDec::NAME, R::cell::<BigDec>()),
            (Fixed::NAME, R::cell::<Fixed>()),
        ],
    }
}

/// Every row, in table order.
pub(crate) fn all() -> Vec<RowReport> {
    vec![
        measure::<holds::Holds>(),
        measure::<bits::Bits>(),
        measure::<merge::Merge>(),
        measure::<negatives::Negatives>(),
        measure::<equality::Equality>(),
        measure::<products::Products>(),
        measure::<specials::Specials>(),
        measure::<correct::Correct>(),
    ]
}

/// One row's name and its cell on one column.
fn on<R: Row, C: Column>() -> (&'static str, Cell) {
    (R::NAME, R::cell::<C>())
}

/// Every row on one column, in table order.
pub(crate) fn column<C: Column>() -> Vec<(&'static str, Cell)> {
    vec![
        on::<holds::Holds, C>(),
        on::<bits::Bits, C>(),
        on::<merge::Merge, C>(),
        on::<negatives::Negatives, C>(),
        on::<equality::Equality, C>(),
        on::<products::Products, C>(),
        on::<specials::Specials, C>(),
        on::<correct::Correct, C>(),
    ]
}

/// Every row on plain `f64`, the control.
pub(crate) fn control() -> Vec<(&'static str, Cell)> {
    column::<Control>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::columns::mutant::{Honest, Mutant};
    use crate::render::verdict;

    #[test]
    fn float_abel_passes_every_row_and_so_does_its_honest_wrapper() {
        let rows = all();
        assert_eq!(rows.len(), 8);
        for r in &rows {
            assert_eq!(r.cells[0].0, FloatAbel::NAME);
            assert!(r.cells[0].1.pass, "{}: {}", r.name, r.cells[0].1.evidence);
        }
        assert!(verdict(&rows)[0]);
        for (name, c) in column::<Mutant<Honest>>() {
            assert!(c.pass, "{name}: {}", c.evidence);
        }
    }

    #[test]
    fn the_control_passes_the_first_row_alone() {
        let cells = control();
        assert_eq!(cells.len(), 8);
        let passes: Vec<bool> = cells.iter().map(|(_, c)| c.pass).collect();
        assert_eq!(
            passes,
            [true, false, false, false, false, false, false, false],
            "{cells:?}"
        );
    }
}
