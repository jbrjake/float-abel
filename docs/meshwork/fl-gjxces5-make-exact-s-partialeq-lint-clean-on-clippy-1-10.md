---
id: fl-gjxces5
title: "Make Exact's PartialEq lint clean on clippy 1.100 in the no-alloc build: stable moved to 1.100.0 on the runner and the nightly's gate job fails at regression.sh's first clippy leg on needless_bool"
status: doing
category: crate/hardening
verify: "all(lacks src/exact/form.rs /return true;/, run cargo test exact_ target=main)"
docs:
  - docs/DESIGN.md#hardening-sp-hardening
seq: 8
created: 2026-11-13T18:58Z
claimed-by: claude (703e78c8-930e-41d9-9c50-c37aaeb95109)
---

Found 2026-11-13: the nightly `ci` run 43414233153 on 08d92c1 is red in one job of seventeen,
`smoke + regression (stable)`, at the first clippy leg of scripts/regression.sh
(`cargo clippy --quiet --lib --no-default-features -- -D warnings`). The runner's `stable` moved
from rustc 1.99.1 to 1.100.0 (0ef1bc386 2026-11-09), and clippy 0.1.100 reports
`clippy::needless_bool` ("this `if` guard returns a bool literal and is followed by another") on
`impl PartialEq for Exact` in src/exact/form.rs. With `alloc` off the `#[cfg(feature = "alloc")]`
heap comparison is stripped, so `eq` reads `if <six fields equal> { return true; } false`. The
build with `alloc` keeps the heap check between the two and is clean.

Reproduced on this machine with a side-by-side 1.100.0 toolchain: that leg exits 101 on
`cargo +1.100.0` and 0 on `cargo +1.99.0`. Nights 8 to 12 November are green because the
`changes` job skipped the suite on an unchanged dev; 08d92c1's README commits were the first
thing to run it on the new compiler.

Wanted: `eq` in a form both builds lint clean on 1.100.0 and 1.99.0, with the comparison order
unchanged (the six fields first, the blocks only when the words differ and both are heap), and
no `#[allow]`. Clippy stops at the first failing leg, so run the whole of scripts/regression.sh
under `RUSTUP_TOOLCHAIN=1.100.0` and fix or file whatever sits behind this one. Done is a
dispatched `ci` run on dev with `conclusion=success`.

## log
- 2026-11-13T18:58Z created
- 2026-11-13T18:58Z open→doing — claimed by claude (703e78c8-930e-41d9-9c50-c37aaeb95109)
