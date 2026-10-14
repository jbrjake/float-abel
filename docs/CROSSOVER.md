# The crossover

The premise the crate's usefulness rests on, in the owner's words: it "only becomes broadly useful
if it makes sense to turn to it because you can't do floats incrementally in dd or dbsp. And it
only makes sense to turn to it for that if it's still faster than recompute of all the floats
despite all the overhead of using float-abel." This memo reads BENCH.md § The crossover against
that premise. The numbers live there, each with its machine, toolchain and commit; this memo
carries the group sizes the decision turns on and what they do and do not settle.

## The two sides

The recompute is the alternative to the crate: keep the group's floats in a `Vec<f64>` and re-sum
them whenever one changes, with no engine around it. It costs O(group) per change: 0.075 ns a
value on NEON and 0.09 on AVX2 when vectorised the way a column engine sums a block, 0.50 and
0.69 ns a value as the dependent chain a running total is. It is cheap for small groups and grows
without bound.

The incremental side is the crate the way differential-dataflow uses it: `explode` the amount into
an `Exact` diff, then `count_total`, on one worker, with one change an epoch stepped to the probe
before the next, which is the engine applying changes as they arrive, with everything the engine
does per change. Beside it, the same dataflow with an `isize` diff (the amount scaled to a count of
units, the fixed-point workaround): the floor an integer diff sets, and the measure of what the
word itself adds.

## The number

The group size past which one change through differential-dataflow with `Exact` costs less than
re-summing the group, commit fa2e7b5, NEON (Apple M5 Max) and AVX2 (AMD Ryzen 7 5700U):

| the engine's granularity | NEON: vs the chain re-sum | vs the vectorised re-sum | AVX2: vs the chain | vs the vectorised |
|---|--:|--:|--:|--:|
| one change an epoch, typed exchange | 8,300 rows | 54,000 rows | 20,800 rows | 113,000 rows |
| one change an epoch, serialized exchange | 9,300 rows | 60,000 rows | 21,200 rows | 113,000 rows |
| a thousand changes an epoch, typed exchange | 116 rows | 362 rows | 132 rows | 800 rows |
| a thousand changes an epoch, serialized exchange | 151 rows | 604 rows | 180 rows | 1,180 rows |

Above the crossover the crate wins and the margin grows linearly with the group: at a million rows
one change is 4 µs through the engine against 77 µs (vectorised) or 499 µs (chain) of re-sum on
NEON, 19× and 122×, and 12.6 µs against 364 or 740 µs on AVX2, 29× and 59×. Below it the re-sum
wins, because a few nanoseconds of hardware adds beat an engine epoch, and the crate is bought for
reproducibility and exact retraction rather than for speed.

A `SUM` over a `GROUP BY` reads the table per group: the recompute re-sums the changed group, so
the row that applies is the group's size, not the table's.

## What sets it

**The engine's epoch, not the word.** One change an epoch costs 4 µs on NEON and 15 µs on AVX2
with `Exact`, and the same within 3% with `isize`, flat from a hundred rows to a hundred thousand
and beyond. That is timely's step and progress machinery for one epoch on one worker; the word's
own deposit is 4.7 ns on NEON and 13 ns on AVX2, a thousandth of it. Nothing done to `Exact` can
move the one-change crossover.

**Batching moves it, by the batch.** A thousand changes an epoch takes the engine's share per
change from 4 µs to 10–20 ns on `isize` and 21–37 ns on `Exact` (NEON), from 15 µs to 26–47 and
69–104 ns (AVX2), and the crossover from tens of thousands of rows to a few hundred. At that
granularity the word is visible: `Exact` is 1.9–2.7× `isize` a change, the band BENCH.md § The
end-to-end SUM measures at scale, and its 11–17 ns (NEON) and 40–57 ns (AVX2) over the integer
diff move the batched crossover by 1.4–2.5× (69 to 116 rows against the chain on NEON, 321 to
800 against the vectorised sum on AVX2). The crossover is set by how often the consumer closes an
epoch, which is the consumer's choice, and the word's cost is a second-order term in it.

**The engine's state is per key, the recompute's per row.** `count_total` arranges `(account, ())`
with the total as its diff; the rows are never stored, which is why the engine's change is flat in
N. The recompute must hold every row to re-sum it. Past the crossover the crate wins on memory as
well as time, by the same linear factor.

## What it settles

The premise holds. For any group past about 10⁴ rows on NEON and 2×10⁴ on AVX2 at one change an
epoch (5×10⁴ and 10⁵ against a vectorised re-sum), and past a few hundred to a thousand rows once
changes are batched by the thousand, `Exact` inside differential-dataflow is faster than
re-summing the floats, with the whole engine's overhead counted, and the margin grows with the
group. Below those sizes the re-sum is faster and the crate is a correctness tool, not a speed
tool.

Speeding the word up has no leverage on the question. The one-change crossover is the engine's,
and the batched crossover moves by 2.5× at most if `Exact` cost nothing more than an integer. The
measurement that would change the answer is the engine's epoch cost or the consumer's batch size,
neither of which the crate owns. The open ruling on the end-to-end ratio (`Exact` at 1.8–2.2×
`isize` at scale, fl-bbevtyv: accept, or revisit the layout) is a throughput question for large
batches and many workers, a different question from this one, and this memo does not decide it.

## What it leaves open

- DBSP is unmeasured. Its `aggregate_linear` has the same O(1)-per-change shape, with its own
  per-step cost in place of timely's epoch; its crossover is that cost over the same `f64` rates.
- One change an epoch on many workers is unmeasured; § The end-to-end SUM covers eight workers
  over 10⁷ updates in one epoch, the throughput shape.
- The consumer's epoch granularity decides which row applies, and only the consumer can measure
  that.
- AVX2's million-row epochs ran 15–25% under its smaller ones on both diffs and both exchanges,
  with the longer untimed preload the only difference, which reads as the host's clock and is
  unverified; the one-change crossovers above average it in, and leaving it out moves them by
  under 4%.
- The timing prices speed alone. The re-sum lands the same bits only while its rows stay in one
  order on one worker, and it cannot retract a row it no longer holds; those are the properties the
  crate exists for, and no row above charges the re-sum for lacking them.
