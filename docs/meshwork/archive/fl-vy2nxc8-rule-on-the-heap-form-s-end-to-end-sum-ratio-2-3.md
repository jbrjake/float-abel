---
id: fl-vy2nxc8
title: "Rule on the heap form's end-to-end SUM ratio — 2.38× isize typed and 2.20× serialized at one worker on NEON at 9714758, against the Copy word's 2.14× and 2.01× and the accepted 1.8–2.2×: accept the moved row, point same-scale consumers at a Copy diff, or revisit the layout"
status: done
category: rulings
labels: [reproducible-floats]
discovered-from: fl-xf8hv25
relates: [fl-xf8hv25, fl-p4dc5w5, fl-pwq0h8n]
docs:
  - BENCH.md#the-end-to-end-sum
  - docs/DESIGN.md#targets-and-kill-conditions-sp-kills
seq: 150
created: 2026-10-16T14:41Z
verify: contains docs/meshwork/fl-vy2nxc8-rule-on-the-heap-form-s-end-to-end-sum-ratio-2-3.md /2026-[0-9][0-9]-[0-9][0-9] owner ruled/
---

Owner's call, raised by the bench half of fl-xf8hv25. The baseline first, because the question is
a ratio over it: `isize` is differential-dataflow's native diff, a machine integer. The `isize`
run is the same SUM over the same 10⁷ rows (sixteen accounts, each amount a signed count of units
of 2⁻²⁰) with every amount pre-scaled to that count, so the engine adds plain integers: the
cheapest diff there is, and the floor an `f64` diff would cost if `f64` were a group. It cannot
carry real floats (a scale chosen in advance, nothing below it, overflow above it, no NaN or
infinities); `Exact` can, and the ratio is what that costs per row.

The owner accepted the Copy word's row on 2026-10-14 ("yeah i am good with the isize one then.
we can close it."): 2.01–2.20× at one worker, 1.77–2.07× at eight. The heap form, ruled on
2026-10-15 ("yes let's do heap and allow product fols inside a dataflow"), is the same word with a
non-`Copy` clone and drop glue; on same-scale data it never allocates, and docs/OVERFLOW.md § Cost
measured that at 1.11× the Copy word typed before the ruling. Re-taken at 9714758 in one quiet
sitting on NEON, the `isize` canaries within 0.3–5% of the earlier rows (BENCH.md § The
end-to-end SUM):

| 10⁷ updates, time per run | heap `Exact` | `isize` | ratio | Copy word, 2a82388 | its ratio |
|---|--:|--:|--:|--:|--:|
| serialized, 1 worker | 455.9 ms | 207.6 ms | 2.20× | 432.5 ms | 2.01× |
| serialized, 8 workers | 78.5 ms | 40.7 ms | 1.93× | 79.7 ms | 1.85× |
| typed, 1 worker | 296.7 ms | 124.6 ms | 2.38× | 265.1 ms | 2.14× |
| typed, 8 workers | 55.5 ms | 28.3 ms | 1.96× | 50.0 ms | 1.77× |

What it means for ordinary input: a SUM over ten million rows at one worker takes 297 ms with the
heap `Exact`, 265 ms with the Copy word and 125 ms with `isize` on this machine, 30 ns a row
against 26.5 and 12.5; at eight workers 5.6 ns a row against 5.0 and 2.8. The 3 ns a row the heap
form adds is what the ruling priced (1.12× measured here against the 1.11× predicted), and it
lands the one-worker typed row at 2.38×, past the 2.2× the owner accepted; the eight-worker rows
stay inside the band. Serialized exchange, the probe's shape, moves 5% at one worker and not at
all at eight. The one-sitting A/B against the saved Copy-word binary within the 1.12×/1.20×
budget is fl-p4dc5w5's and is not this question.

Options, none chosen:
(a) Accept: the diff row reads 1.9–2.4× on the heap form, the ruling for the heap arm having
    priced it, and DESIGN.md § Targets and kill conditions restates the band.
(b) Keep 2.2× for a same-scale consumer by pointing them at a `Copy` diff, `Fixed256` (a declared
    window, 32 bytes, no clone cost) once fl-pwq0h8n gives it the weight traits, with `Exact` the
    diff for data that can promote or needs products.
(c) Revisit the layout, as the kill row says: a `Copy` compact word for the diff with the heap
    form behind a separate type, which reopens the ruled 32-byte niche form.
The owner writes the line this task's verify looks for, dated.

## log
- 2026-10-16T14:41Z created
- 2026-10-16T15:04Z open→done — verify exit 0 @ aecb0fd+1

## comments
- 2026-10-16T15:04Z [claude (028f9a17-8bd2-4590-bb38-7e9820e83a28)] 2026-10-16 owner ruled, in session, on the table and the per-row cost above: "yes i am good with the moved row". Option (a): the diff row reads 1.9–2.4× isize on the heap form at one and eight workers on NEON (BENCH.md § The end-to-end SUM, 91abe38), the layout stands, and DESIGN.md § Targets and kill conditions states the heap form's band as accepted beside the Copy word's. The AVX2 heap rows are not re-taken; the Copy word's 1.78–2.20× stand there until a sitting on jons-amd-mini. The reading from the owner's line to option (a) is the session's, read back in the closing report; a correction reopens this.
