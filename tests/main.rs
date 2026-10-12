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

mod convert;
mod exact_cap;
mod exact_ring;
mod fixed;
mod laws;
mod layout;
mod posture;
mod rounding;
mod support;
mod ui;
mod wide;
mod wide_group;
mod wire;
