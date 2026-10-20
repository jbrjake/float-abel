---
id: fl-jjy5xt9
title: "Rule on the heap word's integer-to-pointer cast under miri: a real pointer in the niche layout, or ptr::with_exposed_provenance once the MSRV allows"
status: open
category: crate/hardening
relates: [fl-sq2sv0n]
verify: lacks src/exact/heap.rs /as usize as \*mut u64/
docs:
  - docs/DESIGN.md#hardening-sp-hardening
seq: 80
created: 2026-10-20T19:18Z
---

`cargo miri test` prints "warning: integer-to-pointer cast" at src/exact/heap.rs's `block_ptr` (`self.hi as usize as *mut u64`: the heap word keeps its block's address in the `hi` limb of the 32-byte niche layout and rebuilds the pointer from the integer on every access), three times a suite run (seen in CI's miri job 118797851505 and here on 2026-10-20). Under miri's default permissive provenance the cast is accepted, but miri says it "might miss pointer bugs" in that code: every access to a heap block goes through a pointer with exposed provenance, so the one place the crate has `unsafe` outside src/simd/ is the one place the interpreter checks least. The strict fix, `ptr::with_exposed_provenance` / `expose_provenance`, is stable from rustc 1.84 and the MSRV is 1.74 on the default features, so it is out until the MSRV moves; the other shape keeps a real `*mut u64` in the word (a union over `hi`, or an enum the niche no longer fits), which touches the layout the owner ruled on ("the niche layout that keeps the word at 32 bytes rather than a 40-byte enum"). Either is the owner's call: ask before building, with the cost of each spelled out (a union keeps the 32 bytes and the `Copy` bound but needs its own `// SAFETY:` reads; the MSRV route is a one-line change behind a version). Meanwhile the miri job's log carries the warning, and a run with `MIRIFLAGS=-Zmiri-strict-provenance` would fail on it.

## log
- 2026-10-20T19:18Z created
