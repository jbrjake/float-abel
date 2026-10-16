---
id: fl-5h12vf3
title: "Rule on docs/OVERFLOW.md's fate — shorten it to the rule, the histories, the hang, the idiom and the measurements with § Options replaced by what was decided, keeping every section the tree cites, or fold it into DESIGN.md and delete it, moving twenty citations"
status: done
category: rulings
labels: [reproducible-floats, docs]
discovered-from: fl-v2msbby
relates: [fl-v2msbby]
docs:
  - docs/OVERFLOW.md#options
  - docs/DESIGN.md#exact-sp-exact
seq: 55
created: 2026-10-16T16:49Z
verify: contains docs/meshwork/fl-5h12vf3-rule-on-docs-overflow-md-s-fate-shorten-it-to-th.md /2026-[0-9][0-9]-[0-9][0-9] owner ruled/
---

Owner's call, surfaced from "State the heap Exact's contract in the outward docs" (fl-v2msbby),
whose body asks the session to decide between shortening docs/OVERFLOW.md and folding it into
DESIGN.md, and whose handoff says the owner reads the choice back before it lands, because the
design cites the doc.

What the doc holds today, 269 lines: an opening that says the owner ruled for the heap arm with
the banded idiom as its fast path and that the measurements describe the compact form; § What
fits (the rule for the compact form and the pairs table, `1e11 + 1e-11` at 126 bits against
`1e12 + 1e-12` at 132); § Why `Exact` panics in a dataflow (the two failure classes and the five
histories); § Why one worker's panic hangs the rest; § Why no 32-byte word takes every double;
§ The banded idiom; § Cost (the probe's rough table that DESIGN.md § Exact and BENCH.md § The
end-to-end SUM read against); and § Options, five options, three rejected shapes, and a
recommendation for option 4 that the ruling for option 5 overtook.

Who cites it, by section, twenty places in the tree: DESIGN.md § Exact (§ Cost), § Exactness
bounds (§ What fits) and § differential-dataflow (the five histories, § Why one worker's panic
hangs the rest, § The banded idiom); BENCH.md § Where the width caps and § The end-to-end SUM
(§ The banded idiom twice, § Cost twice); the rustdoc of src/exact/mod.rs and
src/exact/differential.rs (§ The banded idiom); tests/exact_heap.rs,
tests/differential_products.rs (the five histories, § Why `Exact` panics, § The banded idiom),
tests/differential_banded.rs, tests/dbsp_products.rs and benches/dd_sum.rs (§ The banded idiom);
and in the store fl-pwq0h8n's `docs:` line (docs/OVERFLOW.md#options) with ten task bodies that
name the doc. Every tree citation points at a section that holds the record; only the store's
one `docs:` line points at § Options.

Options:
(a) Shorten. Keep the opening and the six cited sections as they stand. Replace § Options and
    its recommendation with a short closing section, "What was decided": the ruling (the heap
    arm, with the banded idiom as its allocation-free fast path), what it cost (`Exact` is
    `Clone` and not `Copy`, the `alloc` feature, a variable-length wire form) and where the
    shipped record lives (DESIGN.md § Exact and § differential-dataflow, BENCH.md § The
    end-to-end SUM). Re-point fl-pwq0h8n's `docs:` line at § What fits. No citation in the tree
    moves; the doc stays a memo on a resolved question, about 220 lines.
(b) Fold and delete. Move § What fits's rule and table into DESIGN.md § Exactness bounds, the
    five histories and the hang into § differential-dataflow, the banded idiom's prose beside
    the paragraph § differential-dataflow already has, and § Cost's probe table into BENCH.md
    § The end-to-end SUM; delete the file; re-point twenty citations across two docs, two
    rustdoc modules, four test files, a bench and the store. DESIGN.md is 741 lines today, the
    longest doc in the tree, and about 150 more make it a 900-line design doc whose § Exactness
    bounds and § differential-dataflow carry a memo's narrative; the probe's rough table would
    sit in BENCH.md among criterion rows it is not.

Recommendation: (a), because every citation points at a section that stays, the memo's
measurements are a probe's and belong beside their method rather than among BENCH.md's criterion
rows, and DESIGN.md states the design where the memo records how the question was settled. (b)
is the choice if the owner wants one design doc and no memo; then the move is a task of its own
with the twenty citations as its checklist.

Once ruled, this task's comment carries "<date> owner ruled: <the words>", and the chosen
rewrite is filed as its own task with `lacks docs/OVERFLOW.md "## Options"` (a) or
`absent docs/OVERFLOW.md` (b) as the verify.

## log
- 2026-10-16T16:49Z created
- 2026-10-16T17:22Z open→done — verify exit 0 @ 96f8cc7+1

## comments
- 2026-10-16T17:22Z [claude (84727215-1ab8-4972-b9d9-cd4dd0eb435e)] 2026-10-16 owner ruled, in session: "yes follow your recs on overflow. ignore the push to main for now." So (a): shorten docs/OVERFLOW.md, keeping the opening and the six cited sections and replacing § Options and its recommendation with a closing section on what was decided; no citation in the tree moves. The rewrite is its own task, filed from this one.
