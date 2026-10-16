//! The one integration-test target. Each topic is a flat file under `tests/` and a `mod` here, so
//! the suite links once. `autotests = false` in the manifest keeps a topic file from becoming its
//! own binary, and scripts/smoke.sh checks that every topic file is named below.

// The tests compare doubles bit for bit on purpose and move bits between integer widths freely.
#![allow(
    clippy::float_cmp,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::unreadable_literal
)]

mod arbitrary;
mod binned;
mod binned_product;
mod convert;
#[cfg(feature = "dbsp")]
mod dbsp;
#[cfg(feature = "dbsp")]
mod dbsp_products;
#[cfg(feature = "dbsp")]
mod dbsp_words;
mod differential;
mod differential_banded;
mod differential_products;
mod differential_words;
mod exact_cap;
mod exact_heap;
mod exact_heap_ring;
mod exact_ring;
mod fixed;
// A proptest case of the kit runs minutes under the interpreter, and the operators it exercises
// are the tier topics' own.
#[cfg(not(miri))]
mod laws;
mod layout;
mod posture;
mod rounding;
mod simd;
mod support;
mod ui;
mod wide;
mod wide_group;
mod wire;
mod wire_heap;
