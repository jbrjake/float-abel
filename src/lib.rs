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
//! The crate is `no_std`. `Wide`, `Fixed` and `Binned` are `Copy` plain words that never
//! allocate; `Exact` is a 32-byte word that, behind the `alloc` feature, promotes a result past its
//! 127-bit width to one allocation and demotes it after every operation that fits again. Without
//! `alloc` it is the compact form alone, with the width as a named cap. `std` (on by default)
//! implies `alloc` and adds `std::error::Error` impls and the bench harness.
//!
//! [`Exact`], [`Wide`] and [`Fixed`] are the groups. [`Binned`] is the batch-only tier beside
//! them: constant size and the full range, order-independent, and not a group, so it folds a tile
//! and converts to a [`Wide`] rather than serving as a weight.
//!
//! Each group type has one canonical encoding, `FORMAT.md`: `to_bytes` and `from_bytes` on every
//! feature set, `Exact::encode` for either of its forms behind `alloc`, and
//! `Serialize`/`Deserialize` carrying the same bytes behind the `serde` feature. A decoder refuses
//! anything that is not a canonical value, as a [`WireError`], never a panic.
//! `Binned` has none, and crosses a boundary as the `Wide` it converts to. Behind the `arbitrary`
//! feature each group type implements `Arbitrary`, so a fuzz target draws canonical values
//! directly; the crate's own targets live under `fuzz/`.
//!
//! Behind the `differential` feature, [`Exact`] implements differential-dataflow's weight traits
//! (`Semigroup`, `Monoid`, `Abelian`, and `Multiply` by `Exact`, `i64` or `isize` on either side),
//! so a collection takes it as a diff the way it takes `isize`: `explode` a float column into the
//! diff and `count_total` reads each key's exact total back out, with a retraction arriving as
//! `-1 × Exact`. `examples/dd_sum.rs` is that idiom on three workers. `BandedSum` is the same
//! `SUM` with the amount's magnitude band beside the key, so that on amounts of mixed magnitudes
//! no diff the engine consolidates leaves the compact form. [`Wide`] and [`Fixed`] carry the same
//! traits less `Multiply` by themselves: `Wide` is the diff for a weight slot that must be `Copy`
//! or a cast over a mapping, and `Fixed` the declared window, its `try_from` inside the `explode`
//! the per-value door check.
//!
//! Behind the `dbsp` feature, [`Exact`] is a DBSP `DBWeight`: a weighted batch takes it, and
//! `aggregate_linear` takes it as the accumulator, which is the shape of Feldera's
//! `CREATE LINEAR AGGREGATE`, the sanctioned route to an exact `SUM` over doubles.
//! `examples/dbsp_uda/` holds that aggregate's three Rust items beside its SQL. The feature builds
//! on the toolchain its CI job pins, because rustc 1.97 and 1.98 ICE inside dbsp. [`Wide`] and
//! [`Fixed`] are `DBWeight`s too, their archived form the word itself, for the weighted batch and
//! the accumulator; neither is a ring, so a stream's `neg`, `minus` and `differentiate` are
//! `Exact`'s alone.
#![no_std]
#![deny(unsafe_code)]
// Every public item carries rustdoc. A warning here, not a deny: the gate's `-D warnings` is what
// fails on a missing one, because warnings fail the gate only, never the build.
#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]
// Limb arithmetic moves bits between u64, i64, u128, i32 and usize on purpose; each such cast is
// the point of the line it is on, so these three lints would only add noise here.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "arbitrary")]
mod arbitrary;
mod binned;
mod exact;
mod fixed;
mod ieee;
#[cfg(feature = "laws")]
#[cfg_attr(docsrs, doc(cfg(feature = "laws")))]
pub mod laws;
mod limbs;
mod round;
mod simd;
mod wide;
mod wire;

pub use binned::Binned;
#[cfg(feature = "differential")]
#[cfg_attr(docsrs, doc(cfg(feature = "differential")))]
pub use exact::BandedSum;
pub use exact::{Exact, PastCap, EXACT_MAX_EXP, EXACT_MAX_LIMBS};
pub use fixed::{Fixed, Fixed256, OutOfWindow};
pub use wide::Wide;
pub use wire::WireError;

/// The README's Rust blocks, run as doctests so its claims are checked rather than asserted. Behind
/// `differential` because one block drives a dataflow; the self dev-dependency turns that on for a
/// bare `cargo test`.
#[cfg(all(doctest, feature = "differential"))]
#[doc = include_str!("../README.md")]
mod readme {}
