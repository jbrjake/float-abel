---
id: fl-nhcdk92
title: "Measure the heap arm's two layouts before building it — a safe 40-byte enum against a 32-byte form with the pointer in the mantissa's odd-only niche — on the cost probe's three bands and under serialized exchange, and pick by the same-scale number"
category: crate/types
labels: [reproducible-floats, rf-critical-path]
discovered-from: fl-t154dsp
seq: 10
docs:
  - docs/OVERFLOW.md#cost
  - docs/DESIGN.md#exact-sp-exact
status: open
created: 2026-10-15T15:33Z
verify: contains docs/meshwork/fl-nhcdk92-measure-the-heap-arm-s-two-layouts-before-buildi.md /^- 20[0-9][0-9]-[0-9][0-9]-[0-9][0-9]T[0-9:]+Z \[[^]]+\] Layout findings:/
handoff: |
  Next session starts here, on the owner's word (2026-10-15): "yes let's
  do heap and allow product
  fols inside a dataflow … if there's a way to get the best of both
  worlds, i'm all for it. don't
  commence work this session but make sure tasks are filed and ready to go
  next session."
  
  First, read fl-t154dsp's closing comment: it carries the ruling and five
  derived points read
  back for the owner's confirmation; confirm them in session before
  building, in particular
  point 4 (the niche layout would need `unsafe` in `src/exact/heap.rs`, a
  ruling).
  
  What is proven: the panic and its cause (fl-h41w0tr's "Spike findings:"
  comment,
  docs/OVERFLOW.md); the banded idiom's correctness on 1 and 3 workers;
  the rough costs on three
  bands (docs/OVERFLOW.md § Cost), where the heap-arm proxy is 37% over
  the Copy word with no
  promotion. The probes: `repro`, `band` on fl-h41w0tr, `cost` (with the
  proxy `H`) on fl-t154dsp,
  `dd`/`tiers`/`door` on fl-zxpq1wz; copy into a scratchpad crate with the
  attached Cargo.toml,
  `CARGO_TARGET_DIR` outside ~/Documents, `cargo build --release --bin
  cost`.
  
  This task: build the two shapes as newtypes in the scratch crate (the
  safe enum as `H` is, but
  leaner; the niche form with raw pointers, which is fine in a probe), add
  a ProcessBinary column,
  run on a quiet host, and close with "Layout findings:". Then the heap
  task (needs this one), the
  wire task, the adapters task, in that order; the banded fast path and
  the docs follow them; the
  `raise` fix and the closure law are independent and can go first if the
  host is busy.
---
Owner, 2026-10-15, in session: "yes let's do heap and allow product fols inside a dataflow. but i
like your option 4 so if there's a way to get the best of both worlds, i'm all for it."

The heap-arm proxy in docs/OVERFLOW.md § Cost ran 37% over the plain 32-byte `Exact` on the
same-scale band, where it never promotes, so that cost is the shape alone: an enum tag, a 40-byte
update through differential's sort against 32, and a non-`Copy` clone. Before the arm is built,
measure the two shapes it can take, in the scratch crate (`cost`, attached to fl-t154dsp; copy it
into a scratchpad crate with float-abel as a path dependency and `CARGO_TARGET_DIR` outside
~/Documents):

1. **Safe enum, 40 bytes.** `pub struct Exact(Repr)` over a private `enum Repr { Compact { lo,
   hi, exp, nan, pos_inf, neg_inf }, Heap(Box<HeapExact>) }`: no `unsafe`, the layout the proxy
   approximated. Try it leaner than the proxy: `#[inline]` `plus_equals` with the compact path
   first, `Clone` hand-written so the compact arm is a copy, and one allocation for the heap form
   (`Box<[u64]>` or a header-plus-limbs block), not the design's `Box` of a `Vec`.
2. **Niche form, 32 bytes.** Today's `repr(C)` word unchanged for the compact form, whose
   canonical mantissa is odd or the zero, so an even nonzero `lo` is not a value: the heap form
   uses that niche, `lo` even and nonzero (the limb count shifted up, say), `hi` the pointer,
   `exp` and the counts inline. Non-`Copy`, with `Drop` and `Clone` on the pointer. Needs raw
   pointers, so `unsafe` in one module (`src/exact/heap.rs`) with a `// SAFETY:` line per block,
   which lifts `#![deny(unsafe_code)]` outside `src/simd/` for the first time: **a ruling**, put
   to the owner with this task's numbers, not taken.

Run both against plain `Exact`, the banded idiom and `Wide` on the three bands, one worker, and
add a serialized-exchange column (`CommunicationConfig::ProcessBinary`, the probe's shape), since
a boxed arm serializes as its limbs. Ten runs each, the fastest kept, a quiet host (load under 3,
`uptime` and `ps aux | grep cargo` first). Read the same-scale column: the form within a few
percent of plain `Exact` is the one to build; if only the niche form gets there, the ruling
decides; if the enum does, no ruling is needed. Also note what each form does to `size_of` of a
differential update tuple and to the wire form.

Close with a comment opening "Layout findings:" that names the form chosen or the ruling asked,
with the table. The implementation task needs this one.

## log
- 2026-10-15T15:33Z created
- 2026-10-15T15:33Z handoff by claude (6cbb3399-1134-4aff-94e4-f8c1137cf60f)
