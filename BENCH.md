# Measured

The numbers DESIGN.md § Targets and kill conditions reads from. Advisory, never a gate: a missed
target is a design question, not a broken build. Every number names the machine, the toolchain and
the commit it was taken on, and the two architectures are reported side by side, never blended.

- NEON: Apple M5 Max, macOS 26.5, rustc 1.99.0. Two `u64` lanes a step.
- AVX2: AMD Ryzen 7 5700U, Ubuntu 24.04, rustc 1.96.1. Four `u64` lanes a step.

Four harnesses under criterion 0.8.2, each section naming its commit. A figure is criterion's
point estimate of the time per run divided by the run's elements, after a 3 s warm-up and 5 s of
samples (100 of them); `benches/dd_sum.rs` takes ten flat samples of whole runs instead. `cargo
bench` re-takes every table. Timed rows are the crate as a consumer builds it, without LTO:
`Exact`'s operators, its checked methods and the compact tier beneath them are `#[inline]`, so a
consumer's `+=` on the word compiles to the integer ops in place, while `to_f64` and `Wide`'s add
cross the crate boundary as calls, and the bench binary's symbol table shows those two as
out-of-line functions and `checked_add` and `from_f64` as none. A sitting is read only when the
rows no change touched, the `f64` floor, `Wide`'s add and the `isize` dataflow, match an idle
host's within a few percent; both machines are shared, and a sitting that failed that check was
re-taken.

`bench-baseline.json` is the last full sitting on each machine in a form a script can diff: every
criterion id's median per run with its element count, and the peak resident set of the bench
process on the five end-to-end rows below, keyed by the CPU's brand string and naming the commit.
`scripts/bench-baseline.sh write` records a sitting from criterion's estimates and measures the
resident sets, and `compare` prints every row whose median per element is past 1.5× of the
recorded one (`--rss` re-measures the resident sets too, `--all` prints every row). It is
advisory and runs from no hook: a row past the line is a design question for the owner. The
tables in this file stay the record of what each number meant; the file is the record of where
the last sitting stood. The entries at 85b93c2 were taken with the canaries clean on both
machines: on NEON the `f64` floor at 1.00× of the sitting before it, `Wide`'s add at 1.01× and the
four same-scale `isize` rows at 1.01–1.03×; on AVX2 the floor at 0.75 ns, `Wide`'s add at 29.4 ns
and the `isize` rows within 0–6% of the rows in § The end-to-end SUM. Their resident sets are
the harness's as it stands, three 160 MB fixtures and a check phase that runs every
configuration once before any row is timed: 11.4–12.4 GB on every row on macOS and 7.08 GB on
every row on Linux, against the 1.3–5 GB the eight-row harness read in that section, so a
comparison reads them against the entry and never against that table.

## Terms

The crate replaces an `f64` running total with an integer word that holds the sum exactly, so the
result no longer depends on the order the inputs arrived in. The tables name four such words, the
tiers, and a few shapes.

- `Exact`: the engine weight, a 32-byte word: a 127-bit integer mantissa, an exponent, and three
  counts of NaN, +∞ and −∞ inputs. It holds any sum whose largest and smallest set bits lie
  within 127 binades of each other in that word, and past that promotes to one allocation, the
  heap form, which it leaves as soon as a result fits the word again: on same-scale data it never
  promotes. It is the type an engine such as differential-dataflow carries as a diff, so it can
  add, retract and merge.
- `Wide`: a 296-byte word, 34 limbs of 64 bits, a fixed-point accumulator whose lowest bit is the
  smallest subnormal's and whose top is past the largest double: it holds every sum of doubles
  exactly and never refuses. Its cost is bytes, so it suits a per-key aggregate slot rather than a
  per-update diff.
- `Fixed256`: a 32-byte fixed-point accumulator with its lowest bit at 2⁻⁹⁶ and its ceiling at
  2⁹⁵ (256 bits less a sign bit and 64 bits of headroom), for data whose scale the caller
  declares; it refuses a value off that grid or past that ceiling.
- `Binned<3>`: the batch tier, keeping the top three limbs of the running sum at a position that
  follows the largest input seen, the Demmel–Nguyen scheme on integer limbs. It is
  order-independent and exact to within `n · 2⁻¹²⁸ · max|x|`, but it cannot subtract, so it is not
  a group and cannot be an engine's diff; it is for summing a column once.
- *one value at a time*: a running total updated per input, the shape of an aggregate fed one row
  at a time. *bulk*: a whole slice summed by the SIMD kernel, the shape of a column engine. The
  SIMD kernels are the `simd` feature's NEON and AVX2 code; NEON is ARM's 128-bit vector unit
  (the Apple M5 Max here) and AVX2 is x86's 256-bit one (the AMD Ryzen 7 5700U here).
- *same-scale*: inputs whose magnitudes lie within 2^±20 of each other, the band the design's
  targets are stated on; the other bands are named where they appear.

## What reproducibility costs

Every table below carries a plain-float baseline, and this is the one reading that matters: how
much slower the exact tiers are than the `f64` arithmetic they replace, on the same-scale band, on
each machine. The baselines are a scalar `f64 +=` into one slot (a dependent chain of hardware
adds, the shape of an aggregate updated one row at a time), the vectorised naive `f64` sum (eight
independent accumulators, the shape a column engine sums a block in), and, end to end, the engine's
own `isize` diff, which costs what an `f64` diff would: one word, one hardware add. Every figure
is at commit f24b051: `benches/ops.rs` for the one-slot rows, `benches/simd.rs` for the bulk rows
and `benches/dd_sum.rs` for the dataflow, all three in one sitting per machine.

| shape | `f64` | `Exact` | `Wide` | `Fixed256` | `Binned<3>` |
|---|--:|--:|--:|--:|--:|
| one value at a time into one slot, NEON | 0.57 ns | 8.57 ns, 15× | 18.75 ns, 33× | 8.04 ns, 14× | 10.09 ns, 18× |
| one value at a time into one slot, AVX2 | 0.74 ns | 21.27 ns, 29× | 37.11 ns, 50× | 16.36 ns, 22× | 18.52 ns, 25× |
| a column summed in bulk, NEON | 0.080 ns | | 1.83 ns, 23× | 1.61 ns, 20× | 2.00 ns, 25× |
| a column summed in bulk, AVX2 | 0.369 ns | | 2.56 ns, 6.9× | 2.00 ns, 5.4× | 1.68 ns, 4.6× |
| a `SUM` in differential-dataflow, 10⁷ updates, 8 workers, NEON | 41.4 ms (`isize`) | 96.5 ms, 2.33× | | | |
| a `SUM` in differential-dataflow, 10⁷ updates, 8 workers, AVX2 | 175.5 ms (`isize`) | 358.3 ms, 2.04× | | | |

Read it in three parts. Updating an aggregate one row at a time, the engine's shape, costs 15× the
hardware add on NEON and 29× on AVX2 for `Exact`, the engine weight: eight and a half nanoseconds a
row on NEON where a float costs half of one. Summing a whole column, the analytics shape, the SIMD
kernels bring every tier to about two nanoseconds a value, 5–7× the vectorised float sum on AVX2 and
20–25× on NEON, whose float floor is 4.6× lower than the AVX2 machine's; 390 to 620 million doubles
a second a core, exactly, against 2.7 billion (AVX2) to twelve billion (NEON) wrongly. Inside
differential-dataflow, where the diff's cost is diluted by everything else the engine does per
update, `Exact` is 1.8–2.3× the native integer diff at eight workers: 2.33× serialized and 2.23×
typed on NEON, 2.04× and 1.83× on AVX2; the next sections split that between the word and its wire
form. Reading a total out (`to_f64`, 10.1 ns and 19.3 ns) happens once per aggregate, not once per
row, and is not in the table. What the tiers buy for that: the
same bits under any order, worker count or architecture, and exact retraction, where the `f64`
column in tests/differential.rs leaves residue after inserting and deleting the same rows. The
other reading that matters, the group size past which a change through the engine beats
re-summing the group's floats, is § The crossover.

## The prototype table, re-taken

DESIGN.md § Measured, on a prototype: the same operations on the crate, on the prototype's two
magnitude bands and on an adversarial one, 2²⁰ doubles each. Same-scale is top bits uniform in
[2⁻²⁰, 2²⁰) with full random fractions and random signs; 2^±300 is the same over [2⁻³⁰⁰, 2³⁰⁰);
adversarial is uniformly random finite bit patterns, every exponent equally likely, subnormals and
the largest magnitudes included. `benches/ops.rs`, at commit 2a82388.

Every add is a running slot, each operation depending on the last, the shape of an engine's
`plus_equals` into one aggregate. On a band where the 127-bit word caps, the fold restarts at the
refused value, so the slot carries a realistically wide operand everywhere and a refused deposit is
counted (next section) and costs what the refusal costs: the early bound check, not the add. The
read is independent per word, the shape of a column read out, over the words the fold passed
through.

| operation, ns per element | band | NEON | AVX2 |
|---|---|--:|--:|
| `f64 +=`, the floor | same-scale | 0.58 | 0.73 |
| `Exact::deposit` (`from_f64` + add) | same-scale | 4.94 | 15.89 |
| `Exact += &Exact`, the engine's call, operands built | same-scale | 3.81 | 10.89 |
| `Exact::checked_add`, operands built | same-scale | 3.74 | 10.85 |
| `Exact::to_f64`, correctly rounded | same-scale | 9.78 | 17.78 |
| `Exact::deposit_product`, the dot-product shape (41.8% refused) | same-scale | 11.45 | 24.78 |
| `Wide + Wide`, operands built, in L2 | same-scale | 15.60 | 29.48 |
| `f64 +=` | 2^±300 | 0.58 | 0.73 |
| `Exact::deposit` (77.7% refused) | 2^±300 | 6.19 | 15.47 |
| `Exact::checked_add` (77.7% refused) | 2^±300 | 2.87 | 8.16 |
| `Exact::to_f64` | 2^±300 | 6.51 | 14.39 |
| `Wide + Wide` | 2^±300 | 16.16 | 29.53 |
| `f64 +=` | adversarial | 0.58 | 0.73 |
| `Exact::deposit` (92.9% refused) | adversarial | 4.77 | 13.66 |
| `Exact::checked_add` (92.9% refused) | adversarial | 1.62 | 6.18 |
| `Exact::to_f64` | adversarial | 4.69 | 12.36 |
| `Wide + Wide` | adversarial | 16.78 | 29.48 |

Against the targets, which are stated on the same-scale band: compact add ≤ 5 ns is met on NEON,
at 4.9 ns for the deposit and 3.7–3.8 ns for the engine's `+=` and `checked_add` with the operands
built, and missed on AVX2, at 15.9 ns for the deposit and 10.9 ns for the add (3.2× and 2.2×
over); `to_f64` ≤ 5 ns lands at 9.8 ns and 17.8 ns (2.0× and 3.6× over); `Wide` add ≤ 3 ns lands
at 15.6 ns and 29.5 ns (5.2× and 9.8× over). Against the prototype on the same NEON machine
(rustc 1.97, `-C target-cpu=native`, one thread): the compact add with operands built runs at
0.9–1.0× the prototype's 3.7–4.0 ns add and the deposit at 1.1× its 4.3–4.5 ns, the read at
2.2–2.4× its 4.0–4.4 ns, and `Wide + Wide` at 7× its 2.1–2.3 ns.

Three things the rows show about where the time goes. The engine's `+=` on two built words is the
compact tier's integer arithmetic in place, every operator and helper down to it being
`#[inline]`, and the deposit's extra nanosecond on NEON is the `From<f64>` decomposition that
builds one operand; on AVX2 the same add is 10.9 ns and the deposit 15.9, a lead and not a
finding. `Wide`'s add is `limbs::add_assign`, a 34-limb carry chain that threads a flag-derived
carry through every limb, and crosses the crate boundary as a call. The read is the limb-slice
rounding routine `Wide` shares, over a two-limb mantissa, also a call; on the adversarial band,
where most words are a fresh 53-bit double, it is twice as fast as on the same-scale band, where
the running sums are near the width.

## Where the width caps

The kill condition reads the same-scale fold: more than 5% of deposits leaving the compact form
means widening the word before the adapters build on the type. Counted, not timed, over the three
bands above and the same-scale dot-product shape (2¹⁹ pairs of full-mantissa factors), bit for
bit the same on both architectures, at commit 2a82388, when the width was a refusal; the rule is
the same one the heap arm promotes on (a span past 127 bits), so each count below is the
promotion count, and `benches/ops.rs` now prints the rows under that name. A pairwise add is
`from_f64(a) + from_f64(b)` over consecutive pairs; a running fold is `deposit` into one word.

| band | pairwise adds promoted | running fold: deposits in the heap form | first promotion |
|---|--:|--:|--:|
| same-scale | 0 of 524,288 (0.00%) | 0 of 1,048,576 (0.00%) | never |
| 2^±300 | 400,874 of 524,288 (76.46%) | 815,086 of 1,048,576 (77.73%) | deposit 2 |
| adversarial | 486,359 of 524,288 (92.77%) | 974,081 of 1,048,576 (92.90%) | deposit 2 |
| same-scale products | | 219,027 of 524,288 (41.78%) | deposit 4 |

The compact form carries same-scale data inside the width with no promotion at all, which clears
the kill condition; the prototype's 0 of 10⁶ holds on the crate. The 2^±300 band reproduces the
prototype's 77% of pairwise adds needing more than the width, and the adversarial band, where two
values 128 binades apart meet at once, promotes nine deposits in ten: the width is a same-scale
property, as the design says, and a fold over mixed magnitudes runs in the heap form, or stays
compact under the banded idiom (docs/OVERFLOW.md § The banded idiom). The product row is the new
reading. A same-scale sum spans 53 bits of mantissa plus 40 of magnitude, inside 127; a same-scale
product spans 106 bits of mantissa with its lowest bit near 2⁻¹⁴⁴, and a fold of them reaches 2⁴⁰
above, so two products in five promote the running sum, which the heap form holds exactly.

## SIMD earns its feature

Each tier's bulk entry against the one-at-a-time deposits that define it, on the same-scale band.
The kill condition is a kernel under 3× its scalar reference; the design's absolute targets are
1.0 ns (`Fixed`, NEON) and 0.6 ns (AVX2), 0.5 ns (`Binned`) and 1.5 ns (`Wide`) per value.
`benches/simd.rs`, at commit 2a82388.

| tier, same-scale | NEON: one at a time | NEON: bulk | NEON gain | AVX2: one at a time | AVX2: bulk | AVX2 gain |
|---|--:|--:|--:|--:|--:|--:|
| `Wide` | 18.70 ns | 1.82 ns | 10.3× | 36.36 ns | 2.52 ns | 14.4× |
| `Fixed256` | 7.89 ns | 1.60 ns | 4.9× | 16.18 ns | 2.02 ns | 8.0× |
| `Binned<3>` | 10.22 ns | 2.00 ns | 5.1× | 18.51 ns | 1.68 ns | 11.0× |
| vectorised naive `f64` sum, the floor | | 0.080 ns | | | 0.368 ns | |

Every kernel clears the kill condition on both architectures. No kernel reaches its absolute
target: `Fixed` lands at 1.6× its NEON target and 3.4× its AVX2 one, `Wide` at 1.2× and 1.7×, and
`Binned` at 4.0× and 3.4× of a 0.5 ns target that assumed five vector ops per limb and value,
where the kernels spend about seven: a per-limb shift count that has to stay inside a signed byte
on NEON, a left and a right shift on AVX2, and the carry-free digit accumulation.

`Wide`'s NEON kernel owes 14% to the `#[inline]` on `ieee::decompose`, a function the kernel
reaches only through `Wide::deposit` in its refusal closure: with the decomposition inline, the
compiler inlines that closure, and with it `fold_tile` itself, into `wide_extend`; built without
the mark, `wide_extend` calls `fold_tile` out of line with the closure behind an indirect call and
the partial sums spilled around it. The two builds run A/B/A in one sitting at 2.11, 1.82 and
2.11 ns with the one-at-a-time loop and the `f64` floor flat; the AVX2 build inlines the fold
either way and is unchanged to 0.1%.

`Binned<3>` bulk against `Wide` bulk is the batch tier's own kill condition, under 2× faster
meaning `Binned` stays a convenience rather than a recommended default: 0.9× on NEON, where
`Wide`'s kernel is the faster of the two, and 1.5× on AVX2. It stays a convenience.

## Where the window falls back

`Wide` on 2²⁰ doubles with top bits uniform in [2⁻³⁰⁰, 2³⁰⁰): a tile spans far more than the
four-limb window, so almost every vector carries a lane outside it and goes to the scalar deposit.
`benches/simd.rs`, at commit 2a82388.

| `Wide`, 2^±300 | one at a time | bulk |
|---|--:|--:|
| NEON | 18.29 ns | 18.00 ns |
| AVX2 | 35.65 ns | 37.69 ns |

The bulk entry is the scalar fold plus the span pass here; AVX2 pays 6% for it, and NEON nothing,
the deposit inlined into the fold's refusal path running 2% under the bench's own deposit loop.
That fixture is not a column, and the design's `Wide` target is stated on real columns, but it
bounds the cost of the fallback.

## The product-kernel row

The ruling DESIGN.md § Speed waits on: whether `deposit_products` gets a SIMD kernel. Each tier's
bulk `deposit_products` as it stands, the scalar check-then-deposit that checks every pair against
the window and then deposits one pair at a time, beside the one-at-a-time `deposit_product` loop
(the check pass's share is the difference) and the same tier's bulk `deposit` kernel on the same
corpus. The corpus is 2²⁰ same-scale factor pairs with 26-bit mantissas, so every product sits on
`Fixed256`'s grid and under every ceiling; the kernel's cost does not depend on how many mantissa
bits are set. `benches/simd.rs`, at commit 2a82388.

| tier | NEON: `deposit_products` | NEON: loop | NEON: bulk `deposit` | ratio | AVX2: `deposit_products` | AVX2: loop | AVX2: bulk `deposit` | ratio |
|---|--:|--:|--:|--:|--:|--:|--:|--:|
| `Wide` | 25.84 ns | 24.46 ns | 1.80 ns | 14.4× | 56.08 ns | 50.13 ns | 2.49 ns | 22.5× |
| `Fixed256` | 10.70 ns | 9.22 ns | 1.60 ns | 6.7× | 26.08 ns | 20.37 ns | 2.01 ns | 13.0× |
| `Binned<3>` | 15.13 ns | 13.21 ns | 2.00 ns | 7.6× | 31.69 ns | 24.93 ns | 1.69 ns | 18.8× |

The scalar product path costs 7–23× the bulk deposit kernel per pair, which is the band the sums'
one-at-a-time loops sit in above their kernels (5–14× above). A product lane would do what a sum
lane does, the shift and the placement, plus the 106-bit product as four 32×32 multiplies and a
third limb placed, so the gain available is in the same band less the multiply. The check pass is
1.4–1.9 ns a pair on NEON and 6–7 ns on AVX2: a second decomposition of both doubles and a second
product, which a kernel would fold into its lane. The decision is the owner's; this row is its
input.

## The end-to-end SUM

The kill condition on `Exact` as a diff: a differential-dataflow `SUM` over 10⁷ updates with
`Exact` as the diff against the same dataflow with `isize`, at one worker and at eight. Over 2×
means revisiting the layout; 1.3× or under is done. `benches/dd_sum.rs`, at commit 2a82388: the
integration harness's same-scale fixture (sixteen accounts, each amount a signed count of units of
2⁻²⁰ below 2²⁰, carried by `Exact` as the double and by `isize` as the count, so both runs fold one
multiset and an integer oracle checks every `Exact` total bit for bit before timing), the design's
idiom (`explode` into the diff, then `count_total`), and two exchanges. Serialized is
`ProcessBinary`, the shape the probe ran on, where every exchanged diff crosses as its serde
bytes even on one worker, one 33-byte string behind bincode's 8-byte length; typed is `Process`,
the in-process deployment, where it crosses as the word. A sample is one whole run, workers
spawned to workers joined, ten flat samples of several runs each after a 1 s warm-up.

| 10⁷ updates, time per run | NEON: `Exact` | NEON: `isize` | ratio | AVX2: `Exact` | AVX2: `isize` | ratio |
|---|--:|--:|--:|--:|--:|--:|
| serialized, 1 worker | 432.5 ms | 214.9 ms | 2.01× | 1,684.3 ms | 766.3 ms | 2.20× |
| serialized, 8 workers | 79.7 ms | 43.0 ms | 1.85× | 376.3 ms | 181.4 ms | 2.07× |
| typed, 1 worker | 265.1 ms | 123.7 ms | 2.14× | 1,290.6 ms | 603.4 ms | 2.14× |
| typed, 8 workers | 50.0 ms | 28.2 ms | 1.77× | 301.5 ms | 169.7 ms | 1.78× |

On the probe's own shape, serialized exchange, `Exact` runs 1.85–2.20× `isize`: on the 2× line
or just past it at one worker (2.01× on NEON, 2.20× on AVX2) and at eight on AVX2 (2.07×), under
it at eight on NEON (1.85×). On typed exchange it runs 1.77–2.14×. No row reaches the 1.3× done
line. Per update on NEON at one worker, `isize` costs 12 ns typed and 21 ns serialized, `Exact`
27 ns typed and 43 ns serialized; on AVX2, 60 and 77 against 129 and 168. The split says where
the gap is. Typed exchange isolates the word's own cost: 14 ns an update over `isize` on NEON and
69 on AVX2, which is the compact add (3.7 ns and 10.9 ns in the rows above), the 32-byte diff
moving through consolidation's sort as a 48-byte update tuple against 24, and the `From<f64>`
that builds it. Serialized exchange adds another 8 ns on NEON and 23 on AVX2 net of `isize`'s own
serialization, which is the byte string sized, written and read back through bincode (§ The wire
form), validated on decode. The word is now the larger share of the serialized gap, about two
thirds on NEON and three quarters on AVX2.

Re-taken on the heap form, the 32-byte niche word that is `Clone` and not `Copy`, at commit
9714758 on NEON in one quiet sitting (the one-minute load 2.9 at the start, the four `isize`
rows within 0.3–5% of the rows above, so the sittings compare; the fixture is the same multiset).
On same-scale data the form never promotes, so the column is the word's own cost as a non-`Copy`
type: the tag test, the drop glue, and the clone through consolidation's sort, none of which
touches the arithmetic.

| 10⁷ updates, time per run | NEON: heap `Exact` | NEON: `isize` | ratio | the Copy word's row above | heap / Copy |
|---|--:|--:|--:|--:|--:|
| serialized, 1 worker | 455.9 ms | 207.6 ms | 2.20× | 432.5 ms | 1.05× |
| serialized, 8 workers | 78.5 ms | 40.7 ms | 1.93× | 79.7 ms | 0.99× |
| typed, 1 worker | 296.7 ms | 124.6 ms | 2.38× | 265.1 ms | 1.12× |
| typed, 8 workers | 55.5 ms | 28.3 ms | 1.96× | 50.0 ms | 1.11× |

Per update at one worker, `isize` costs 12.5 ns typed and 20.8 ns serialized, the heap `Exact`
29.7 and 45.6, where the Copy word cost 26.5 and 43.2: the form adds 3 ns an update on typed
exchange, which is the 1.11× docs/OVERFLOW.md § Cost predicted from the probe, and 2 ns on
serialized exchange at one worker and nothing measurable at eight, under the probe's 1.17–1.20×,
because the serialized rows carry the wire form's share, which the heap arm's compact bytes
leave unchanged. Read against the kill line, the eight-worker rows stay inside the 1.8–2.2× the
owner accepted for the Copy word and the one-worker typed row moves past it, 2.38×; the owner
accepted the moved row (fl-vy2nxc8), so the layout stands and DESIGN.md § Targets and kill
conditions carries 1.9–2.4× for the heap form. The heap-over-Copy column is a
cross-sitting ratio; the one-sitting A/B against the Copy word's saved control binary within the
1.12×/1.20× budget is fl-p4dc5w5's and is not read from this table. Peak RSS was not re-taken:
the fixture never promotes, so the heap form allocates nothing on these rows, and the macOS
high-water mark below is the allocator's as much as the dataflow's.

The banded idiom beside the plain one, at commit 16db257 on NEON in one quiet sitting (the
one-minute load 2.90 at the start with nothing else running on the host, the four `isize` rows
within 0.4–4.4% of the 9714758 rows above). Two fixtures: the same-scale one above, and a mixed
one whose amounts spread over 2^±300 with full mantissas and either sign, checked against a `Wide`
oracle per account, on which every account's total spans past the width. `exact` is the plain
idiom, `explode` into the diff then `count_total`; `banded` is `BandedSum::banded_sum`, `explode`
and `count_total` per (account, band) and a `reduce` of each account's band totals into one
`Exact` (docs/OVERFLOW.md § The banded idiom). `isize` has no mixed row: an integer diff has no
magnitudes to mix.

| 10⁷ updates, time per run | `exact` | `banded` | banded / exact | `isize` | exact / isize | banded / isize |
|---|--:|--:|--:|--:|--:|--:|
| same-scale, serialized, 1 worker | 478.4 ms | 517.6 ms | 1.08× | 198.4 ms | 2.41× | 2.61× |
| same-scale, serialized, 8 workers | 82.4 ms | 94.3 ms | 1.14× | 39.5 ms | 2.09× | 2.39× |
| same-scale, typed, 1 worker | 323.2 ms | 341.0 ms | 1.06× | 128.4 ms | 2.52× | 2.66× |
| same-scale, typed, 8 workers | 57.9 ms | 62.7 ms | 1.08× | 28.2 ms | 2.05× | 2.22× |
| mixed 2^±300, serialized, 1 worker | 1,082.0 ms | 693.2 ms | 0.64× | | | |
| mixed 2^±300, serialized, 8 workers | 189.9 ms | 116.9 ms | 0.62× | | | |
| mixed 2^±300, typed, 1 worker | 930.5 ms | 501.5 ms | 0.54× | | | |
| mixed 2^±300, typed, 8 workers | 156.4 ms | 87.9 ms | 0.56× | | | |

Three readings. On same-scale data neither idiom promotes, and the banded one costs 6–14% over
the plain one: 1.8 ns an update typed and 3.9 serialized at one worker, which is the wider
(account, band) key through `count_total`'s arrangement and the second operator, a `reduce`
over each account's band totals with an arrangement and an exchange of its own, the exchange
being why the gap is widest on serialized exchange at eight workers. On mixed data the plain
idiom's running diffs promote, and its row costs 2.3–2.9× its same-scale row, 60 ns an update
more at one worker on either exchange; the typed and serialized gaps are the same 60 ns, so the
cost is the heap form's promotion and limb arithmetic at each consolidation step, not the wire,
whose compact bytes the heap form leaves alone. The banded idiom's diffs stay compact
(tests/differential_banded.rs asserts every band total compact on its multiset), and its mixed
row costs 1.24–1.47× its same-scale row, 16–18 ns an update at one worker: the fixture occupies
nineteen finite bands per account, so `count_total` arranges nineteen times the groups and the
reduce re-folds up to nineteen band totals per change. Net, the banded idiom runs the mixed
rows in 0.54–0.64× the plain idiom's time, 1.56–1.86× faster, and allocates nothing where the
plain one promotes. Read against the probe's rough table in docs/OVERFLOW.md § Cost, which had
the niche form at 70–86 ns typed on the spread and mixed bands against banded 42–49, 1.5–2.0×,
the real arm reads 93 against 50 on the mixed fixture at one worker typed, 1.86×, inside that
range in a slower sitting shape (criterion's whole runs against the probe's one timed epoch).

A cross-sitting note. The plain idiom's same-scale rows here read 4–9% above the 9714758 rows
above while the `isize` rows read within 4% of theirs, so the exact / isize column sits at
2.05–2.52× against that sitting's 1.93–2.38×. The plain path's code is unchanged between the two
commits; the bench binary now carries the banded and mixed rows and holds three 160 MB fixtures
in place of one. The two binaries then ran the one-worker typed rows back to back in one quiet
window, criterion data apart: 9714758's read 283.7 ms `exact` and 121.6 ms `isize`, 2.33×, and
this one 318.8 and 128.5, 2.48×. The `isize` dataflow and its fixture are byte for byte the same
in both, so the shift is the bench binary's, 6% on `isize` and 12% on `exact`, not the lib's
plain path and not the day. A ratio in this table reads against the rows above only through
such an A/B, and the heap arm's budget against the Copy word (fl-p4dc5w5) compares binaries of
one harness shape.

Peak resident set of the bench process over one row's iterations, `/usr/bin/time -l` on macOS and
`/usr/bin/time -v` on Linux, on the serialized rows: NEON 3.43 GB (`Exact`, 1 worker), 5.04 GB
(`isize`, 1), 3.09 GB (`Exact`, 8), 2.51 GB (`isize`, 8); AVX2 1.26 GB, 1.25 GB, 1.31 GB and
1.26 GB in the same order. The process holds the 160 MB fixture and each run's input batches, and
the figure is a high-water mark across every iteration, so it carries the allocator's reuse
policy as much as the dataflow: macOS's puts `isize` above `Exact` at one worker, and the Linux
figures are the ones to read, where the diff's width costs 4 MB at one worker and 55 MB at eight.

`Wide` as the diff, the `Copy` word (DESIGN.md § Wide), at commit 440ac8c on NEON in one sitting
(the one-minute load 2.75 at the start with nothing else running on the host, the four `isize`
rows within 1–5% of the 16db257 rows above): the same-scale fixture through the plain idiom with
`Wide` in the diff, every exchanged update carrying a 296-byte weight in a 312-byte tuple against
`Exact`'s 32 and 48. The plain `Exact` and `isize` rows were re-taken in the same sitting, so the
ratios are in-sitting. `Wide` has no mixed row, because its cost does not depend on the
magnitudes.

| 10⁷ updates, time per run | `wide` | `exact` | `isize` | wide / exact | wide / isize |
|---|--:|--:|--:|--:|--:|
| serialized, 1 worker | 2,443.6 ms | 513.9 ms | 209.3 ms | 4.75× | 11.7× |
| serialized, 8 workers | 377.3 ms | 86.3 ms | 39.7 ms | 4.37× | 9.5× |
| typed, 1 worker | 1,790.9 ms | 348.6 ms | 130.5 ms | 5.14× | 13.7× |
| typed, 8 workers | 264.6 ms | 60.1 ms | 27.8 ms | 4.41× | 9.5× |

Per update at one worker, `Wide` costs 179 ns typed and 244 serialized, where `Exact` costs 35 and
51 and `isize` 13 and 21: the rough figure docs/OVERFLOW.md § Cost took from the probe, 172 and
236, within 4%. The gap is bytes, not arithmetic: `Wide`'s add is the fastest of any tier
(DESIGN.md § Wide), and the 144 ns an update over `Exact` on typed exchange is the 312-byte tuple,
6.5× the bytes, moving through consolidation's sort, the batcher's merges and the arrangement;
serialized exchange adds 65 ns over typed where it adds 17 to `Exact`, which is the 297-byte
string written and read back through bincode. At eight workers the ratio closes to 4.4×. Peak
resident set of the bench process over the `wide/1w/serialized` row alone, `/usr/bin/time -l`,
13.64 GB against the 3.43 GB the `Exact` row read on this machine above: the explode's output
alone is 3.1 GB of tuples at 10⁷ updates, and the figure carries the check phase, which runs
every configuration once, and macOS's high-water mark. The row is what the `Copy` diff costs
where an engine's weight slot must be `Copy` or casts over a mapping; where it need not, `Exact`
is 4.4–5.1× cheaper on the same multiset. A cross-sitting note: the plain `Exact` rows here read
4–8% above the 16db257 rows while the `isize` rows read within 1–5%, with the lib's plain path
unchanged between the two commits; the bench binary carries one more dataflow, and the note above
on harness shape applies.

### The heap arm's budget

The one-sitting A/B against the `Copy` word: its `dd_sum` binary at 03d96e9, the eight-row
harness on the same-scale fixture, saved before the heap arm landed, against the heap form's lib
at 85b93c2 built into that same harness (a worktree at 85b93c2 with 03d96e9's benches/dd_sum.rs),
so the two binaries differ in the lib alone. Both ran their one-worker rows back to back on NEON,
criterion data apart, with nothing else of ours on the host. The budget is the probe's
(docs/OVERFLOW.md § Cost): within 1.12× the control typed and 1.20× serialized.

| 10⁷ updates, one worker, time per run | `Copy` word (control) | heap `Exact` (variant) | variant / control | budget |
|---|--:|--:|--:|--:|
| `exact`, typed | 271.1 ms | 294.0 ms | 1.08× | 1.12× |
| `exact`, serialized | 427.2 ms | 463.8 ms | 1.09× | 1.20× |
| `isize`, typed | 123.7 ms | 123.0 ms | 0.99× | canary |
| `isize`, serialized | 207.4 ms | 207.7 ms | 1.00× | canary |

The `isize` rows, whose dataflow and fixture are byte for byte the same in both binaries, agree
to 0.6%, so the sitting is clean and the `exact` gap is the lib's. The heap form costs 2.3 ns an
update typed (27.1 against 29.4) and 3.7 serialized (42.7 against 46.4) at one worker: inside the
budget on both exchanges, and under the 1.11× typed and 1.17–1.20× serialized the probe
predicted. The cross-sitting heap-over-Copy column in the heap-form table above, read against
another day's control, put the typed row at 1.12× and the serialized one at 1.05×; this row is the
one the budget is read from. The control rows match the 2a82388 rows at the top of this section
within 1–3% (432.5, 265.1, 214.9 and 123.7 ms): the same binary on the same machine, days apart.

## The wire form

Where the serialized gap is. timely's `ProcessBinary` exchange encodes every batch of updates
through bincode 1 three times over: `length_in_bytes` runs the serializer once to count,
`into_bytes` runs it again into the reserved slab, and the receiver's `from_bytes` decodes it, so
a diff's `Serialize` runs twice and its `Deserialize` once per crossing. `benches/wire.rs`, at
commit 2a82388, runs each pass alone on one batch of `((u32, ()), u64, R)` updates, the tuple
`count_total`'s `arrange_by_self` exchanges, at timely's buffer capacity for the tuple (170 updates
with a 32-byte diff, 341 with `isize`), on the SUM's rows, with every form round-tripped against
its input before timing. Four diffs: `isize`; `Exact` as the crate serialises it, one byte string
of its 33 wire bytes, which bincode 1 frames with an 8-byte length (DESIGN.md § Wire format); and
two other forms built in the bench as newtypes over `to_bytes`. `tuple` is the 33-element tuple of
`u8`, as a `[u8; 33]` serialises, 33 bytes with no length. `fields` is a 7-element tuple
`(u8, u64, u64, i32, i32, i32, i32)`, which bincode's fixed-int encoding frames as the tuple's 33
bytes exactly, checked in the bench, and a varint format such as postcard does not.

| ns per update | NEON: size | write | read | all three | AVX2: size | write | read | all three |
|---|--:|--:|--:|--:|--:|--:|--:|--:|
| `isize`, 20 bytes an update | 0.00 | 5.67 | 2.41 | 8.1 | 0.01 | 7.76 | 8.52 | 16.3 |
| `Exact`, one byte string, 53 bytes | 0.52 | 8.24 | 5.98 | 14.7 | 1.22 | 14.07 | 17.99 | 33.3 |
| `tuple`, the 33-element tuple, 45 bytes | 1.13 | 77.59 | 6.47 | 85.2 | 3.46 | 127.82 | 26.15 | 157.4 |
| `fields`, the 7-element tuple, 45 bytes | 0.54 | 21.15 | 10.50 | 32.2 | 1.22 | 29.55 | 25.04 | 55.8 |

The write is the cost, and it is per serializer call, not per byte. bincode 1 writes each element
with one `write_all` on the slab writer, a 1-byte copy for a `u8` and an 8-byte one for a `u64` at
about the same price, so a form's write pass costs its call count: `isize` makes three calls an
update (the key, the epoch, the diff), the byte string four (the key, the epoch, the length, then
the 33 bytes in one copy), the field tuple nine and the 33-element tuple 35, and the pass costs
5.7, 8.2, 21.1 and 77.6 ns on NEON and 7.8, 14.1, 29.6 and 127.8 on AVX2. The sizing pass is near
free, bincode's size counter being a sum the compiler folds, and the read is cheap on bincode's
slice reader even one byte at a time, 6.5 ns for the tuple on NEON and 26 on AVX2; the byte string
reads through `from_bytes` in one borrowed slice. Net of `isize`, the crate's form costs 6.7 ns an
update on NEON and 17 on AVX2 for its three passes, which is the 8 and the 23 ns that § The
end-to-end SUM attributes to the wire on serialized exchange at one worker (the serialized gap
less the typed one), for 8 bytes of length an update more on the wire than the tuple forms
carry. The field tuple's row is the bench newtype's, which gathers the seven elements back into
33 bytes for `from_bytes`; a crate impl would build the word from the fields directly, and would
still make nine calls.

## The crossover

The question the crate's usefulness turns on. An engine cannot hold a float `SUM` incrementally
(DESIGN.md § The problem), so the alternative to the crate is the recompute: keep the group's
floats and re-sum them whenever one changes, O(group) per change and no engine around it. The
crossover is the group size past which one change through differential-dataflow with `Exact` as
the diff, with everything the engine does per change, costs less than that re-sum.
`benches/crossover.rs`, at commit f24b051. The incremental side is one worker, one group
preloaded at N rows under one key and stepped to the probe untimed, then one change an epoch,
each epoch stepped to the probe before the next; a change is an insert and, the epoch after, its
retraction, so the group holds at N through the sample. The dataflow is `explode` into the diff
then `count_total`, with `Exact` and, beside it as the floor, the amount scaled by 2²⁰ into an
`isize` (the fixed-point workaround), on serialized and typed exchange as in § The end-to-end
SUM, and both are oracle-checked before timing. The recompute side is a `Vec<f64>` of the group
re-summed per change: vectorised, eight accumulators, the shape a column engine re-sums a block
in; and as the dependent chain, a running total's shape. A reading is the time of one change;
the engine's rows are criterion's `iter_custom` over whole epochs, the re-sum's its plain loop.
The docs/CROSSOVER.md memo reads this section for the owner's decision.

| one change, group of N | 100 | 1,000 | 10,000 | 100,000 | 1,000,000 |
|---|--:|--:|--:|--:|--:|
| NEON: re-sum, vectorised | 5.3 ns | 61.2 ns | 739 ns | 7.51 µs | 77.2 µs |
| NEON: re-sum, chain | 14.4 ns | 390 ns | 4.78 µs | 46.8 µs | 489 µs |
| NEON: `Exact` in differential, typed | 3.89 µs | 4.01 µs | 4.00 µs | 4.05 µs | 4.08 µs |
| NEON: `Exact` in differential, serialized | 4.43 µs | 4.47 µs | 4.58 µs | 4.54 µs | 4.61 µs |
| NEON: `isize` in differential, typed | 3.83 µs | 3.89 µs | 3.93 µs | 3.97 µs | 4.01 µs |
| NEON: `isize` in differential, serialized | 4.37 µs | 4.47 µs | 4.41 µs | 4.45 µs | 4.49 µs |
| AVX2: re-sum, vectorised | 9.8 ns | 86.8 ns | 910 ns | 9.50 µs | 365 µs |
| AVX2: re-sum, chain | 47.8 ns | 672 ns | 6.92 µs | 69.4 µs | 733 µs |
| AVX2: `Exact` in differential, typed | 15.88 µs | 15.84 µs | 16.52 µs | 16.21 µs | 13.22 µs |
| AVX2: `Exact` in differential, serialized | 16.30 µs | 16.23 µs | 16.32 µs | 16.15 µs | 11.70 µs |
| AVX2: `isize` in differential, typed | 15.67 µs | 15.84 µs | 15.02 µs | 15.71 µs | 12.86 µs |
| AVX2: `isize` in differential, serialized | 15.88 µs | 15.80 µs | 16.52 µs | 16.49 µs | 12.96 µs |

The same dataflows at a thousand changes an epoch, where the engine amortises one epoch over the
batch, per change; the re-sum rows do not move, because a re-sum is per change whatever the
batch. The `Exact` deposit into a running slot with no engine, the incremental side's own floor,
is 5.52 ns a change on NEON and 15.12 ns on AVX2 in the same sittings.

| a thousand changes an epoch, group of N | 10,000 | 100,000 | 1,000,000 |
|---|--:|--:|--:|
| NEON: `Exact`, typed | 24.0 ns | 24.0 ns | 24.0 ns |
| NEON: `Exact`, serialized | 41.1 ns | 40.7 ns | 41.9 ns |
| NEON: `isize`, typed | 10.1 ns | 10.0 ns | 10.0 ns |
| NEON: `isize`, serialized | 19.1 ns | 19.0 ns | 19.4 ns |
| AVX2: `Exact`, typed | 61.6 ns | 61.1 ns | 61.0 ns |
| AVX2: `Exact`, serialized | 97.5 ns | 97.3 ns | 95.6 ns |
| AVX2: `isize`, typed | 30.9 ns | 30.9 ns | 26.8 ns |
| AVX2: `isize`, serialized | 48.3 ns | 47.4 ns | 43.8 ns |

The crossover, the group size at which the re-sum costs what the engine's change does, by linear
interpolation between the measured sizes. The re-sum is linear in N from 10⁴ up on NEON, 0.075 ns
a value vectorised and 0.48 ns as the chain, and on AVX2 to 10⁵, 0.09 and 0.69 ns; below 10⁴
criterion's independent iterations overlap on the core and the chain reads 0.14–0.39 ns a value
on NEON and 0.48–0.67 on AVX2, which flatters the re-sum and is the conservative side. On AVX2
the vectorised series bends at 10⁶, where 8 MB of doubles leaves the cache (0.36 ns a value, the
bulk floor of § SIMD earns its feature), so the one-change crossover against it lies between the
interpolated figure in the table and the 160,000 the in-cache rate gives.

| crossover, group size | NEON: against the chain | against the vectorised re-sum | AVX2: against the chain | against the vectorised re-sum |
|---|--:|--:|--:|--:|
| `Exact`, one change an epoch, typed | 8,410 | 53,400 | 22,400 | 115,000 |
| `Exact`, one change an epoch, serialized | 9,470 | 60,300 | 22,100 | 115,000 |
| `isize`, one change an epoch, typed | 8,250 | 52,400 | 21,700 | 114,000 |
| `isize`, one change an epoch, serialized | 9,300 | 59,200 | 22,400 | 115,000 |
| `Exact`, a thousand changes an epoch, typed | 123 | 401 | 119 | 701 |
| `Exact`, a thousand changes an epoch, serialized | 164 | 678 | 171 | 1,110 |
| `isize`, a thousand changes an epoch, typed | 70 | 176 | 62 | 331 |
| `isize`, a thousand changes an epoch, serialized | 111 | 324 | 97 | 529 |

Four readings. The engine's change is flat in N: 3.89–4.08 µs typed and 4.43–4.61 µs serialized
on NEON across four decades of group size, within 5%, and 15.8–16.5 and 16.1–16.3 µs on AVX2 from
10² to 10⁵, within 5% again, because `count_total` arranges `(account, ())` with the total as its
diff and the rows are never stored, so the state is per key, not per row, while the re-sum holds
every row and grows with them. (AVX2's 10⁶ rows ran at 11.7–13.2 µs, 17–28% under the rest, on
both diffs and both exchanges alike; the only difference in what runs is the longer untimed
preload before each sample, so that reads as the host's clock, not the group, and is unverified.)
At one change an epoch the epoch is the cost: `Exact` and `isize` sit within 4% of each other on
both machines, but for AVX2's typed pair at 10⁴, 10% apart where the `isize` row reads low against
its neighbours (and AVX2's 10⁶ rows aside), and the word's own deposit, 5.5 ns on NEON and 15 ns
on AVX2, is a seven-hundredth to a thousandth of the epoch, which is timely's step and progress
machinery for one epoch on one worker. The
crossover therefore moves with the epoch granularity and not with the word: a thousand changes an
epoch takes the engine's share per change from 4 µs to 10–19 ns on `isize` and 24–42 ns on
`Exact` (NEON), and from 16 µs to 27–48 and 61–98 ns (AVX2), and the crossover from about 10⁴
rows (NEON) and 2×10⁴ (AVX2) against the chain and 5×10⁴ and 10⁵ against the vectorised sum
down to about 120–170 and 400–1,100. The word's whole cost over an integer diff a change, 14 ns
typed and 22 ns serialized on NEON and 32 and 50 ns on AVX2 (the 2.1–2.4× of the batched rows,
near the band § The end-to-end SUM measures at scale), moves the batched crossover by 1.5–2.3×
and the one-change crossover by 3.4% or less, in either direction. Below the crossover the re-sum
wins because a few nanoseconds of adds beat an engine epoch; above it the gap grows linearly in
N, and at 10⁶ rows one change costs 19× (vectorised) and 120× (chain) the engine's on NEON, 28×
and 55× on AVX2.

## Binned kernel (ii)

DESIGN.md § Speed names two candidate kernels for `Binned` and keeps the winner. Kernel (i) is the
integer fold at a per-tile index, the one `extend_from_slice` runs. Kernel (ii) is ReproBLAS's
pinned-exponent floating-point fold, `q = (M + x) − M` per bin, in the one shape that lands the
one-at-a-time fold's bits: `2K` half-bins of 32 bits on `Wide`'s half-limb grid (the fold's bins
are independent, so a bin must align with a limb and a piece must fit the accumulator), each
level's nearest rounding corrected to truncation toward zero by one compare and one masked
subtract, pieces signed by an xor, and every half-bin read back as an integer through its bits.
That is eight vector ops a level on NEON (`fadd`, `fsub`, `fcmgt`, `fsub`, `bit`, `eor`, `fadd`,
`fsub`) and seven on AVX2 over four lanes, for `2K` levels, against (i)'s about seven per limb for
`K` limbs. Beside it ran the same peel without the fix-up, ReproBLAS's own kernel, which cannot land
the bits and only bounds what the shape would cost if truncation were free. Both live at commit
4b533a9: `cargo bench --bench simd -- binned3`, with the loop and (i) re-taken in the same sitting.

| `Binned<3>`, same-scale | NEON | AVX2 |
|---|--:|--:|
| one at a time | 10.89 ns | 18.04 ns |
| (i) the integer fold, `extend_from_slice` | 2.00 ns | 1.67 ns |
| (ii) the floating-point peel, truncating | 3.17 ns | 3.01 ns |
| (ii) without the fix-up, nearest: not the fold's bits | 2.29 ns | 3.55 ns |

(i) is the kernel, 1.6× faster than (ii) on NEON and 1.8× on AVX2. The floating-point shape loses
on its own, not only on the truncation: with the fix-up gone it is still 1.14× behind (i) on NEON.
The AVX2 figure for that variant is a codegen artefact and not the bound to read: the compiler
shuffled its lanes through 128-bit halves (`vunpcklpd`, `vshufpd`, `vextractf128`) where the
truncating peel got straight 256-bit code. The cost is structural, `2K` levels of seven or eight
ops against `K` limbs of about seven, and no fixture in the design's set changes that ratio.
