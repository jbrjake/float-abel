---
id: fl-cfx001b
title: "Amend PLAN-reproducible-floats.md for the POD Exact — §3.2's heap arm and the Box, §3.5's alloc row, and §11 call 7 (the limb cap) are gone; Exact is a 32-byte Copy value with i32 special counts and the width, exponent and count caps"
category: rulings
labels: [reproducible-floats]
to: portfolio
docs:
  - portfolio#PLAN-reproducible-floats.md#32-representations
verify: contains docs/DESIGN.md /32 bytes/
seq: 47
status: dropped
created: 2026-10-11T18:00Z
---
Owner's rulings, 2026-10-11, in float-abel's session: zero-copy is paramount; "in general i approve
dropping the heap arm"; "we absolutely need exact multiplication". The design in
float-abel/docs/DESIGN.md § Exact is the source; the plan's §3.2 ("the `Wide` arm needs a `Box`"),
§3.5 (the `alloc` row) and §11 call 7 (the `Exact` limb cap) describe the shape that was dropped.
The portfolio's open ruling task names call 7; it dissolves with the limb cap.

## log
- 2026-10-11T18:00Z created
- 2026-10-15T15:33Z open→dropped — Owner, 2026-10-15: 'yes let's do heap and allow product fols inside a dataflow'. The plan's §3.2 heap arm, §3.5 alloc row and §11 call 7 return; a new ask follows the heap task fl-th7g69s with the settled design.
