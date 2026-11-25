---
id: fl-btst4df
title: Gate the heap ring test's laws-only strategies and imports off under miri
status: done
category: crate/hardening
relates: [fl-jphvxrs]
verify: contains tests/exact_heap_ring.rs /(?s)#\[cfg\(not\(miri\)\)\]\s*(use float_abel::laws|fn double|mod )/
docs:
  - docs/DESIGN.md#hardening-sp-hardening
seq: 70
created: 2026-10-20T17:08Z
---

tests/exact_heap_ring.rs imports `laws::{self, Weight}` and defines the four proptest strategies `double`, `special`, `mixed` and `products` for the laws test alone, which is `#[cfg(not(miri))]`, so `cargo +nightly miri test --test main` prints six warnings (unused import `self`, unused import `Weight`, and the four functions never used; seen on 2026-10-20 at e813edf). No gate runs clippy under `cfg(miri)`, so nothing fails, but the miri log carries noise that would hide a real warning. Put the import and the four strategies under the same `#[cfg(not(miri))]` as the test they feed, one attribute each or one gated module, and confirm `cargo +nightly miri test --test main exact_heap_` builds with no warning.

## log
- 2026-10-20T17:08Z created
- 2026-11-24T23:51Z open→doing — claimed by claude (4aebe618-581f-48ef-9ee8-943721730ef9)
- 2026-11-24T23:54Z doing→done — verify exit 0 @ 0efc0e4+2

## comments
- 2026-11-24T23:54Z [claude (4aebe618-581f-48ef-9ee8-943721730ef9)] 2026-11-24 at 0efc0e4: the laws test and what it alone used (the proptest prelude, laws::{self, Weight}, and the strategies double, special, mixed and products) sit in one `#[cfg(not(miri))] mod kit`, the shape tests/main.rs gates the laws topics with; the test's path is now exact_heap_ring::kit::…, which no file names. Observed on nightly 11177f223 (2026-08-02), aarch64 macOS: `cargo +nightly miri test --test main no_such_test_filter_xyz` printed the six warnings before and none after, 187 tests filtered out both times; natively the file's five tests pass. `cargo miri test --no-run` prints no warning either way: cargo-miri stores the test target's rustc invocation and the interpreter compiles it when the binary runs, so a filter that matches no test is the three-second check.
