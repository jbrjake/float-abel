//! The panic without a dataflow: differential's own public `consolidate_updates`, the function
//! at the top of the backtrace, over the same three updates in two orders. Then the boundary:
//! a sum of two doubles whose exact result spans 127 bits that `checked_add` refuses while
//! `Exact::try_from(&Wide)` accepts the same value.

use std::panic::{catch_unwind, AssertUnwindSafe};

use differential_dataflow::consolidation::consolidate_updates;
use float_abel::{Exact, PastCap, Wide};

fn e(x: f64) -> Exact {
    Exact::from(x)
}

fn consolidate(label: &str, updates: Vec<(u32, u64, Exact)>) {
    let shown: Vec<String> = updates.iter().map(|(_, _, d)| format!("{:e}", d.to_f64())).collect();
    let mut v = updates;
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        consolidate_updates(&mut v);
        v.iter().map(|(_, _, d)| format!("{:e}", d.to_f64())).collect::<Vec<_>>()
    }));
    match outcome {
        Ok(left) => println!("{label:<12} {shown:?} -> consolidates to {left:?}"),
        Err(_) => println!("{label:<12} {shown:?} -> PANIC in consolidate_updates"),
    }
}

fn main() {
    std::panic::set_hook(Box::new(|info| {
        let msg = info
            .payload()
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| info.payload().downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default();
        eprintln!("    panic: {msg}");
    }));

    println!("-- differential's consolidate_updates, one key, one time, three diffs --");
    for (big, small) in [(1e12, 1e-12), (2f64.powi(1000), 2f64.powi(-1000))] {
        consolidate("as C", vec![(1, 0, e(big)), (1, 0, e(small)), (1, 0, e(-big))]);
        consolidate("as E", vec![(1, 0, e(big)), (1, 0, e(-big)), (1, 0, e(small))]);
    }
    println!("(sort_unstable_by on equal (data, time) keeps no promised order; the slice order is what the fold walks)");

    println!("\n-- the boundary: a result that spans exactly 127 bits --");
    let two127 = 2f64.powi(127);
    let via_add = e(two127).checked_add(&e(-1.0));
    let via_wide = Exact::try_from(&(Wide::from(two127) + Wide::from(-1.0)));
    println!("2^127 + (-1) via checked_add:            {via_add:?}");
    println!("2^127 + (-1) via Wide then try_from:     {:?}", via_wide.map(|x| x.to_f64()));
    let (neg, mant, exp) = {
        let w = Wide::from(two127) + Wide::from(-1.0);
        let x = Exact::try_from(&w).expect("fits");
        (x < Exact::ZERO, x, 0)
    };
    let _ = (neg, mant, exp);
    assert_eq!(via_add, Err(PastCap::Width));
    println!("2^127 - 1 is 127 one-bits: inside the width, refused by the add alone.");
    // A sum whose magnitude genuinely reaches 2^127 is refused by both routes.
    let plus = e(two127).checked_add(&e(1.0));
    let plus_wide = Exact::try_from(&(Wide::from(two127) + Wide::from(1.0)));
    println!("2^127 + 1 via checked_add: {plus:?}; via Wide: {:?}", plus_wide.map(|x| x.to_f64()));
}
