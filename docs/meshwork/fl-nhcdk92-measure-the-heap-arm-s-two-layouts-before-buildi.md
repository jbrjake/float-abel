---
id: fl-nhcdk92
title: "Measure the ruled 32-byte niche form of the heap arm before building it — against the Copy word, the banded idiom and the 40-byte enum on the cost probe's three bands and under serialized exchange, settling the block shape and the budget the heap task must meet"
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
  next session", and on
  the layout: "yes i'm okay with one more unsafe to save that many bits on
  every value".
  
  Read fl-t154dsp's closing comment first: the ruling and five derived
  points read back; point 4
  (the niche layout and its `unsafe` module) is ratified by the second
  quote above; points 1, 2, 3
  and 5 are derived and still await the owner's confirmation in session
  before the heap task
  builds on them, in particular point 3 (dyadic special amounts in the
  heap form, so
  `PastCap::Count` goes away).
  
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
  
  This task: build the niche form as a newtype in the scratch crate with
  the two block shapes, add
  a ProcessBinary column, run on a quiet host, and close with "Layout
  findings:" carrying the
  budget. Then the heap task fl-th7g69s (needs this one), the wire task
  fl-mrdqpay, the engine
  proof fl-xf8hv25, in that order; the banded fast path fl-gpjzcqc, the
  closure law fl-x8fwfe3 and
  the docs fl-v2msbby follow them; the `raise` fix fl-7r8qney is
  independent and can go first if
  the host is busy.
---

Owner, 2026-10-15, in session: "yes let's do heap and allow product fols inside a dataflow. but i
like your option 4 so if there's a way to get the best of both worlds, i'm all for it." And on
the layout, same session: "yes i'm okay with one more unsafe to save that many bits on every
value". So the form is ruled, the 32-byte niche form; what this task settles is the budget and
the block.

The heap-arm proxy in docs/OVERFLOW.md § Cost ran 37% over the plain 32-byte `Exact` on the
same-scale band, where it never promotes, so that cost is the shape alone: an enum tag, a 40-byte
update through differential's sort against 32, and a non-`Copy` clone. Build the niche form as a
newtype in the scratch crate (`cost`, attached to fl-t154dsp; copy it into a scratchpad crate
with float-abel as a path dependency and `CARGO_TARGET_DIR` outside ~/Documents; raw pointers
are fine in a probe): today's word for the compact form, an even nonzero `lo` carrying the limb
count and `hi` the pointer for the heap form, `Clone` a copy on the compact path, `Drop` freeing
the block, `plus_equals` through `checked_add` first with promotion to a `Wide`-shaped block on
refusal and demotion after. Try two block shapes, a bare `Box<[u64]>` of limbs with `exp` and
the amounts inline, and one block holding a header and the limbs, and keep the faster.

Run it against plain `Exact`, the banded idiom and the 40-byte enum proxy on the three bands,
one worker, with a serialized-exchange column (`CommunicationConfig::ProcessBinary`, the probe's
shape), since a heap form serializes as its limbs. Ten runs each, the fastest kept, a quiet host
(load under 3, `uptime` and `ps aux | grep cargo` first). The same-scale column against plain
`Exact` is the budget fl-th7g69s must meet; note what the form does to `size_of` of a
differential update tuple (unchanged, 48 bytes with a `u64` key) and to the wire form.

Close with a comment opening "Layout findings:" carrying the table, the block shape chosen and
the budget. The heap task needs this one.

## log
- 2026-10-15T15:33Z created
- 2026-10-15T15:33Z handoff by claude (6cbb3399-1134-4aff-94e4-f8c1137cf60f)
- 2026-10-15T15:36Z handoff by claude (6cbb3399-1134-4aff-94e4-f8c1137cf60f)

## comments
- 2026-10-15T15:38Z [claude (6cbb3399-1134-4aff-94e4-f8c1137cf60f)] Owner, 2026-10-15, in session: the whole readback on fl-t154dsp is ratified ("yes confirmed"), so next session starts building here with no ruling outstanding.
