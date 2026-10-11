---
id: fl-nna1djz
title: "Certify Wide and Fixed as plain words — repr(C), the layout pinned by const assertions and documented as the wire bytes, so an engine's unsafe cast over a mapping is sound"
category: crate/types
labels: [reproducible-floats, rf-critical-path]
docs:
  - docs/DESIGN.md#wide-sp-wide
  - docs/DESIGN.md#fixed-sp-fixed
  - docs/DESIGN.md#wire-format-sp-wire
verify: "all(contains src/wide.rs /#\\[repr\\(C\\)\\]/, contains src/fixed.rs /#\\[repr\\(C\\)\\]/, run cargo test package=float-abel layout_)"
seq: 46
status: open
created: 2026-10-11T18:00Z
covers:
  - ref: docs/DESIGN.md#sp-wide
    sha: f9cd0e6f7109063e08dd1b3fcadc03a4951424efd48c72a6412ff3e1cfdebdb8
  - ref: docs/DESIGN.md#sp-fixed
    sha: 8fa2eadaab9aab115d1312b30e795f024b3b0fcdf571a7817f12502a4690e1ae
  - ref: docs/DESIGN.md#sp-wire
    sha: ed67fb6c74a2f5d25395ee4cc391ead8d530b72592a9d5ea223a9ce4123ad2f3
---
Owner's ruling, 2026-10-11: "i cannot stress enough the paramount importance of zero-copy here";
"copy/alloc has to be limited to only the bare necessities. the normal path needs to be zero copy
when possible, zero copy after one conversion when not"; external stores (mmap, Arrow, shared
memory) "definitely, but wherever else we can too".

leras and sazed use no zero-copy crate. Each hand-rolls `unsafe trait PlainWord: Copy` and casts a
`memmap2` mapping with `align_to::<T>()`; the contract is no padding, no invalid bit pattern, and
alignment at most a page, with `repr(C)` or `repr(transparent)` on anything in an extent. What they
need from this crate is `Copy`, a `repr(C)` layout that equals the little-endian wire bytes, and a
`const` assertion pinning size and alignment, so the cast impl on their side is sound by
inspection. `Wide` and `Fixed` already have no padding and every bit pattern of theirs is a
canonical two's-complement value, so a blind cast is correct. `Exact` is `repr(C)` and `Copy` too
(the redesign task), but a byte pattern with an even mantissa is not canonical, so bytes from
outside it still go through the validating decoder.

Tests `layout_*` in `tests/layout.rs`: size and alignment of each type by `const` assertion; the
wire bytes of a value are its in-memory bytes on little-endian (read through `to_le_bytes` of each
limb, never through `unsafe`); and a `Wide` or `Fixed256` rebuilt from the bytes of another equals
it structurally.

## log
- 2026-10-11T18:00Z created
- 2026-10-11T18:12Z cover docs/DESIGN.md#sp-wide @f9cd0e6f7109
- 2026-10-11T18:12Z cover docs/DESIGN.md#sp-fixed @8fa2eadaab9a
- 2026-10-11T18:12Z cover docs/DESIGN.md#sp-wire @ed67fb6c74a2
