---
id: fl-k7gec8v
title: "Amend PLAN-reproducible-floats.md for the heap Exact — §3.2's heap arm returns as the 32-byte niche form, not a Box arm; §3.5's alloc row returns; §11 call 7's limb cap returns as EXACT_MAX_LIMBS with the checked API; Exact is Clone, not Copy, with dyadic special amounts"
status: open
category: rulings
labels: [reproducible-floats]
discovered-from: fl-th7g69s
to: portfolio
verify: contains docs/DESIGN.md /EXACT_MAX_LIMBS/
docs:
  - portfolio#PLAN-reproducible-floats.md#32-representations
  - docs/DESIGN.md#exact-sp-exact
created: 2026-10-15T17:40Z
---

Owner, 2026-10-15, in float-abel's session: "yes let's do heap and allow product fols inside a
dataflow"; on the layout, "yes i'm okay with one more unsafe to save that many bits on every
value". The heap arm is built (float-abel 09c92b5, task fl-th7g69s), and
float-abel/docs/DESIGN.md § Exact is the source for what the plan should say.

The settled design, for the plan's §3.2, §3.5 and §11 call 7: `Exact` is a 32-byte word with two
forms, not an enum with a `Box` arm. The compact form is the 127-bit dyadic with `i32` special
counts inline; the heap form is the same word tagged by an even nonzero low limb, `lo` the
block's capacity doubled and `hi` the pointer to one allocation holding a four-entry header and
the four normalised limb runs (the finite part and the three special amounts as dyadics), reached
by promotion only and left by demotion after every operation whose result fits again, so one
value has one form. `Exact` is `Clone`, not `Copy`; `Wide`, `Fixed` and `Binned` stay `Copy`
words. The `alloc` feature returns and `std` implies it; without it `Exact` is the compact form
with the width as its cap. The caps with `alloc` are the limb cap, `EXACT_MAX_LIMBS` at 16 × 2148
bits (call 7 returns: a named panic, with the checked API returning `PastCap::Limbs`), and the
exponent bound. `Mul<Exact>` carries dyadic special amounts, so `2.5 · NaN` is a value and a
`float × float` join chain holds. Measured cost of the form against the `Copy` word on same-scale
data, one worker: the budget is 1.12× typed and 1.20× serialized (docs/OVERFLOW.md § Cost), and
the sitting's rows are in BENCH.md § The end-to-end SUM.

The wire form of the heap value is the open item (float-abel task "Encode the heap form on the
wire"): `to_bytes` is the compact form's, serde carries a heap value under a provisional tag,
and rkyv's archive refuses it by name. The plan's rows that cite the 32-byte `Copy` value and
the width, exponent and count caps describe the dropped shape.

## log
- 2026-10-15T17:40Z created
