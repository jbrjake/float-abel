//! Exact abelian-group sums of `f64`.
//!
//! An incremental engine keeps a running answer and applies changes to it: an insert adds a weight,
//! a delete adds its negation, and a background merge may add any subset in any order. That only
//! works if the weight type is an exact abelian group, and `f64` addition is not one. This crate
//! holds sums (and products) of IEEE doubles exactly, as a canonical abelian group with a ring on
//! top, so the engine lands the same bits under any order, worker count, merge tree, consolidation
//! timing or architecture.
//!
//! No floating-point instruction executes in any group operation or read: only `to_bits`, shifts
//! and adds with carry. That is what makes the bits architecture-independent.
//!
//! The crate is `no_std`. The `alloc` feature enables the heap tier, and `std` (on by default)
//! enables `std::error::Error` impls and the bench harness.
#![no_std]
#![deny(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]
// Limb arithmetic moves bits between u64, i64, u128, i32 and usize on purpose; each such cast is
// the point of the line it is on, so these three lints would only add noise here.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]

// The heap tier of `Exact` is the first user of `alloc`; nothing in the crate allocates yet.
#[cfg(feature = "alloc")]
#[allow(unused_extern_crates)]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod fixed;
mod ieee;
mod limbs;
mod round;

pub use fixed::{Fixed, Fixed256, OutOfWindow};
