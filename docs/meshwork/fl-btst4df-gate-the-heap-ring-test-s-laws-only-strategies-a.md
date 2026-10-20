---
id: fl-btst4df
title: Gate the heap ring test's laws-only strategies and imports off under miri
status: open
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
