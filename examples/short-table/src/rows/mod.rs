//! One file per row: the eight properties, each a set of probes run over every column.

pub(crate) mod bits;
pub(crate) mod equality;
pub(crate) mod holds;
pub(crate) mod merge;
pub(crate) mod negatives;

use crate::column::Column;
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

/// Every row, in table order. A row joins this list when its file lands.
pub(crate) fn all() -> Vec<RowReport> {
    vec![
        measure::<holds::Holds>(),
        measure::<bits::Bits>(),
        measure::<merge::Merge>(),
        measure::<negatives::Negatives>(),
        measure::<equality::Equality>(),
    ]
}
