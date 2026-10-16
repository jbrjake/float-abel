---
id: fl-62b32y9
title: "Shorten docs/OVERFLOW.md to the rule, the histories, the hang, the idiom and the measurements — replace § Options and its recommendation with what was decided, keep every section the tree cites, re-point the store's one § Options link"
status: done
category: crate/docs
labels: [reproducible-floats, docs]
discovered-from: fl-5h12vf3
verify: "all(lacks docs/OVERFLOW.md /## Options/, contains docs/OVERFLOW.md /## What was decided/)"
docs:
  - docs/OVERFLOW.md#what-fits
seq: 55
created: 2026-10-16T17:22Z
---

The owner's ruling on fl-5h12vf3, 2026-10-16, in session: "yes follow your recs on overflow",
which is option (a) of that task: keep the opening and the six sections the tree cites (§ What
fits, § Why `Exact` panics in a dataflow, § Why one worker's panic hangs the rest, § Why no
32-byte word takes every double, § The banded idiom, § Cost) as they stand, and replace § Options
and its recommendation, which argued for a choice the ruling for the heap arm overtook, with a
closing section on what was decided: the heap arm with the banded idiom as its allocation-free
fast path, what it cost (`Exact` is `Clone` and not `Copy`, the `alloc` feature, a
variable-length wire form), the shapes not taken and why (a wider word, a sticky overflowed
state, a declared window on `Exact`), and where the shipped record lives (DESIGN.md § Exact and
§ differential-dataflow, BENCH.md § The end-to-end SUM). The opening paragraph stops numbering
options. No citation in the tree moves; the store's one `docs:` line at § Options (fl-pwq0h8n)
is re-pointed at § What fits.

## log
- 2026-10-16T17:22Z created
- 2026-10-16T17:22Z open→doing — claimed by claude (84727215-1ab8-4972-b9d9-cd4dd0eb435e)
- 2026-10-16T17:24Z doing→done — verify exit 0 @ ec2ae7c+5

## comments
- 2026-10-16T17:24Z [claude (84727215-1ab8-4972-b9d9-cd4dd0eb435e)] ec2ae7c. docs/OVERFLOW.md keeps its opening and the six sections the tree cites unchanged in substance, § What fits through § Cost, so none of the twenty citations moves. § Options and the recommendation for option 4, which the ruling for the heap arm overtook, are replaced by § What was decided: the heap arm with the banded idiom as its fast path and the owner's words as the source, what it costs (Clone not Copy, the alloc feature, the variable-length wire form, 1.11× the Copy word typed and 1.4–2.0× the banded idiom where promotions happen, from § Cost), the five shapes not taken with the reason for each (Wide or a Fixed window as the diff, the banded idiom alone, a wider Exact, a sticky overflowed state, a declared window on Exact), and where the shipped record lives. The opening no longer numbers options. 264 lines against 269: the closing section is a condensed record rather than a cut, since the shapes not taken and their reasons are the part of the question a reader of the design cannot find elsewhere. The store's one link at § Options, fl-pwq0h8n's docs: line, points at § What fits.
