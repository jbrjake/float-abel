# Measured

The numbers DESIGN.md § Targets and kill conditions reads from. Advisory, never a gate: a missed
target is a design question, not a broken build. Every number names the machine, the toolchain and
the commit it was taken on, and the two architectures are reported side by side, never blended.

- NEON: Apple M5 Max, macOS 26.5, rustc 1.99.0. Two `u64` lanes a step.
- AVX2: AMD Ryzen 7 5700U, Ubuntu 24.04, rustc 1.96.1. Four `u64` lanes a step.

Three harnesses under criterion 0.8.2, all at commit 7792a3f unless a section says otherwise. A
figure is criterion's point estimate of the time per run divided by the run's elements, after a
3 s warm-up and 5 s of samples (100 of them); `benches/dd_sum.rs` takes ten flat samples of whole
runs instead. `cargo bench` re-takes every table. Timed rows are the crate as a consumer builds it,
without LTO: every operator crosses the crate boundary as a call, because none is marked
`#[inline]`, and the bench binary's symbol table shows `checked_add`, `to_f64` and `Wide`'s
`add_assign` as out-of-line functions.

## Terms

The crate replaces an `f64` running total with an integer word that holds the sum exactly, so the
result no longer depends on the order the inputs arrived in. The tables name four such words, the
tiers, and a few shapes.

- `Exact`: the engine weight, a 32-byte word: a 127-bit integer mantissa, an exponent, and three
  counts of NaN, +∞ and −∞ inputs. It holds any sum whose largest and smallest set bits lie
  within 127 binades of each other, and refuses one that would not (a *width cap*): on same-scale
  data it never refuses. It is the type an engine such as differential-dataflow carries as a diff,
  so it can add, retract and merge.
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
own `isize` diff, which costs what an `f64` diff would: one word, one hardware add.

| shape | `f64` | `Exact` | `Wide` | `Fixed256` | `Binned<3>` |
|---|--:|--:|--:|--:|--:|
| one value at a time into one slot, NEON | 0.54 ns | 5.98 ns, 11× | 18.36 ns, 34× | 7.83 ns, 14× | 9.91 ns, 18× |
| one value at a time into one slot, AVX2 | 0.74 ns | 16.10 ns, 22× | 36.37 ns, 49× | 15.99 ns, 22× | 18.50 ns, 25× |
| a column summed in bulk, NEON | 0.084 ns | | 2.09 ns, 25× | 1.59 ns, 19× | 1.99 ns, 24× |
| a column summed in bulk, AVX2 | 0.370 ns | | 2.53 ns, 6.8× | 2.01 ns, 5.4× | 1.68 ns, 4.5× |
| a `SUM` in differential-dataflow, 10⁷ updates, 8 workers, NEON | 42.8 ms (`isize`) | 150.3 ms, 3.5× | | | |
| a `SUM` in differential-dataflow, 10⁷ updates, 8 workers, AVX2 | 181.0 ms (`isize`) | 503.8 ms, 2.8× | | | |

Read it in three parts. Updating an aggregate one row at a time, the engine's shape, costs 11×
the hardware add on NEON and 22× on AVX2 for `Exact`, the engine weight: six nanoseconds a row
where a float costs half of one. Summing a whole column, the analytics shape, the SIMD kernels
bring every tier to about two nanoseconds a value, 5–7× the vectorised float sum on AVX2 and
19–25× on NEON, whose float floor is four times lower than the AVX2 machine's; five hundred
million doubles a second a core, exactly, against three billion (AVX2) to twelve billion (NEON)
wrongly. Inside differential-dataflow, where the diff's cost is diluted by everything else the
engine does per update, `Exact` is 2–4× the native integer diff; the next sections split that
between the word and its wire form. Reading a total out (`to_f64`, 9.5 ns and 17.6 ns) happens
once per aggregate, not once per row, and is not in the table. What the tiers buy for that: the
same bits under any order, worker count or architecture, and exact retraction, where the `f64`
column in tests/differential.rs leaves residue after inserting and deleting the same rows.

## The prototype table, re-taken

DESIGN.md § Measured, on a prototype: the same operations on the crate, on the prototype's two
magnitude bands and on an adversarial one, 2²⁰ doubles each. Same-scale is top bits uniform in
[2⁻²⁰, 2²⁰) with full random fractions and random signs; 2^±300 is the same over [2⁻³⁰⁰, 2³⁰⁰);
adversarial is uniformly random finite bit patterns, every exponent equally likely, subnormals and
the largest magnitudes included. `benches/ops.rs`.

Every add is a running slot, each operation depending on the last, the shape of an engine's
`plus_equals` into one aggregate. On a band where the 127-bit word caps, the fold restarts at the
refused value, so the slot carries a realistically wide operand everywhere and a refused deposit is
counted (next section) and costs what the refusal costs: the early bound check, not the add. The
read is independent per word, the shape of a column read out, over the words the fold passed
through.

| operation, ns per element | band | NEON | AVX2 |
|---|---|--:|--:|
| `f64 +=`, the floor | same-scale | 0.54 | 0.74 |
| `Exact::deposit` (`from_f64` + add) | same-scale | 5.98 | 16.10 |
| `Exact += &Exact`, the engine's call, operands built | same-scale | 6.80 | 17.68 |
| `Exact::checked_add`, operands built | same-scale | 8.12 | 19.52 |
| `Exact::to_f64`, correctly rounded | same-scale | 9.47 | 17.58 |
| `Exact::deposit_product`, the dot-product shape (41.8% refused) | same-scale | 11.43 | 25.03 |
| `Wide + Wide`, operands built, in L2 | same-scale | 15.70 | 29.48 |
| `f64 +=` | 2^±300 | 0.54 | 0.74 |
| `Exact::deposit` (77.7% refused) | 2^±300 | 6.19 | 15.52 |
| `Exact::checked_add` (77.7% refused) | 2^±300 | 3.82 | 11.12 |
| `Exact::to_f64` | 2^±300 | 6.41 | 14.23 |
| `Wide + Wide` | 2^±300 | 16.97 | 29.49 |
| `f64 +=` | adversarial | 0.54 | 0.74 |
| `Exact::deposit` (92.9% refused) | adversarial | 4.79 | 13.69 |
| `Exact::checked_add` (92.9% refused) | adversarial | 2.03 | 8.01 |
| `Exact::to_f64` | adversarial | 4.56 | 12.15 |
| `Wide + Wide` | adversarial | 16.50 | 29.41 |

Against the targets, which are stated on the same-scale band: compact add ≤ 5 ns lands at 6.0 ns
for the deposit and 6.8 ns for the engine's `+=` on NEON (1.2× and 1.4× over) and at 16–18 ns on
AVX2 (3.2–3.5× over); `to_f64` ≤ 5 ns lands at 9.5 ns and 17.6 ns (1.9× and 3.5× over); `Wide` add
≤ 3 ns lands at 15.7 ns and 29.5 ns (5.2× and 9.8× over). Every target is missed on both
architectures. Against the prototype on the same NEON machine (rustc 1.97, `-C target-cpu=native`,
one thread): the compact rows run 1.3–1.5× the prototype's 4.3–4.5 ns deposit, 3.7–4.0 ns add and
4.0–4.4 ns read, and `Wide + Wide` runs 7× its 2.1–2.3 ns.

Three things the rows show about where the time goes, each a lead and not a finding. The add is
slower with both operands built than inside the deposit that builds one of them, which is the
cost of the call boundary: `checked_add` returns a 40-byte `Result` through memory, and the
dependent chain waits on that store before the next add. `Wide`'s add is `limbs::add_assign`, a
34-limb carry chain that threads a flag-derived carry through every limb. The read is the
limb-slice rounding routine `Wide` shares, over a two-limb mantissa; on the adversarial band, where
most words are a fresh 53-bit double, it is twice as fast as on the same-scale band, where the
running sums are near the width.

## Where the width caps

The kill condition reads the same-scale fold: more than 5% caps means widening the word before the
adapters build on the type. Counted, not timed, over the three bands above and the same-scale
dot-product shape (2¹⁹ pairs of full-mantissa factors), bit for bit the same on both
architectures. A pairwise add is `from_f64(a).checked_add(&from_f64(b))` over consecutive pairs; a
running fold is `checked_deposit` into one word, restarted at each refusal.

| band | pairwise adds refused | running fold: deposits refused | first refusal |
|---|--:|--:|--:|
| same-scale | 0 of 524,288 (0.00%) | 0 of 1,048,576 (0.00%) | never |
| 2^±300 | 400,874 of 524,288 (76.46%) | 815,086 of 1,048,576 (77.73%) | deposit 2 |
| adversarial | 486,359 of 524,288 (92.77%) | 974,081 of 1,048,576 (92.90%) | deposit 2 |
| same-scale products | | 219,027 of 524,288 (41.78%) | deposit 4 |

The compact tier carries same-scale data inside the width with no cap at all, which clears the
kill condition; the prototype's 0 of 10⁶ holds on the crate. The 2^±300 band reproduces the
prototype's 77% of pairwise adds needing more than the width, and the adversarial band, where two
values 128 binades apart meet at once, caps nine deposits in ten: the width is a same-scale
property, as the design says, and a fold over mixed magnitudes belongs in `Wide`. The product row
is the new reading. A same-scale sum spans 53 bits of mantissa plus 40 of magnitude, inside 127; a
same-scale product spans 106 bits of mantissa with its lowest bit near 2⁻¹⁴⁴, and a fold of them
reaches 2⁴⁰ above, so two products in five cannot share the word with the running sum. `Exact`
holds any one product exactly, as DESIGN.md § Exact promises, and a fold of same-scale products is
a `Wide` job.

## SIMD earns its feature

Each tier's bulk entry against the one-at-a-time deposits that define it, on the same-scale band.
The kill condition is a kernel under 3× its scalar reference; the design's absolute targets are
1.0 ns (`Fixed`, NEON) and 0.6 ns (AVX2), 0.5 ns (`Binned`) and 1.5 ns (`Wide`) per value.
`benches/simd.rs`.

| tier, same-scale | NEON: one at a time | NEON: bulk | NEON gain | AVX2: one at a time | AVX2: bulk | AVX2 gain |
|---|--:|--:|--:|--:|--:|--:|
| `Wide` | 18.36 ns | 2.09 ns | 8.8× | 36.37 ns | 2.53 ns | 14.4× |
| `Fixed256` | 7.83 ns | 1.59 ns | 4.9× | 15.99 ns | 2.01 ns | 8.0× |
| `Binned<3>` | 9.91 ns | 1.99 ns | 5.0× | 18.50 ns | 1.68 ns | 11.0× |
| vectorised naive `f64` sum, the floor | | 0.084 ns | | | 0.370 ns | |

Every kernel clears the kill condition on both architectures. No kernel reaches its absolute
target: `Fixed` lands at 1.6× its NEON target and 3.3× its AVX2 one, `Wide` at 1.4× and 1.7×, and
`Binned` at 4.0× and 3.4× of a 0.5 ns target that assumed five vector ops per limb and value,
where the kernels spend about seven: a per-limb shift count that has to stay inside a signed byte
on NEON, a left and a right shift on AVX2, and the carry-free digit accumulation.

`Binned<3>` bulk against `Wide` bulk is the batch tier's own kill condition, under 2× faster
meaning `Binned` stays a convenience rather than a recommended default: 1.05× on NEON and 1.5× on
AVX2. It stays a convenience.

## Where the window falls back

`Wide` on 2²⁰ doubles with top bits uniform in [2⁻³⁰⁰, 2³⁰⁰): a tile spans far more than the
four-limb window, so almost every vector carries a lane outside it and goes to the scalar deposit.

| `Wide`, 2^±300 | one at a time | bulk |
|---|--:|--:|
| NEON | 18.15 ns | 18.23 ns |
| AVX2 | 35.66 ns | 37.71 ns |

The bulk entry is the scalar fold plus the span pass here; AVX2 pays 6% for it. That fixture is not
a column, and the design's `Wide` target is stated on real columns, but it bounds the cost of the
fallback.

## The product-kernel row

The ruling DESIGN.md § Speed waits on: whether `deposit_products` gets a SIMD kernel. Each tier's
bulk `deposit_products` as it stands, the scalar check-then-deposit that checks every pair against
the window and then deposits one pair at a time, beside the one-at-a-time `deposit_product` loop
(the check pass's share is the difference) and the same tier's bulk `deposit` kernel on the same
corpus. The corpus is 2²⁰ same-scale factor pairs with 26-bit mantissas, so every product sits on
`Fixed256`'s grid and under every ceiling; the kernel's cost does not depend on how many mantissa
bits are set.

| tier | NEON: `deposit_products` | NEON: loop | NEON: bulk `deposit` | ratio | AVX2: `deposit_products` | AVX2: loop | AVX2: bulk `deposit` | ratio |
|---|--:|--:|--:|--:|--:|--:|--:|--:|
| `Wide` | 25.99 ns | 24.22 ns | 2.09 ns | 12.4× | 56.08 ns | 50.17 ns | 2.44 ns | 23.0× |
| `Fixed256` | 10.70 ns | 9.24 ns | 1.60 ns | 6.7× | 26.01 ns | 20.27 ns | 2.00 ns | 13.0× |
| `Binned<3>` | 14.88 ns | 12.89 ns | 1.97 ns | 7.5× | 31.94 ns | 24.96 ns | 1.67 ns | 19.1× |

The scalar product path costs 7–23× the bulk deposit kernel per pair, which is the band the sums'
one-at-a-time loops sat in before their kernels (5–14× above). A product lane would do what a sum
lane does, the shift and the placement, plus the 106-bit product as four 32×32 multiplies and a
third limb placed, so the gain available is in the same band less the multiply. The check pass is
1.5–2 ns a pair on NEON and 6–7 ns on AVX2: a second decomposition of both doubles and a second
product, which a kernel would fold into its lane. The decision is the owner's; this row is its
input.

## The end-to-end SUM

The kill condition on `Exact` as a diff: a differential-dataflow `SUM` over 10⁷ updates with
`Exact` as the diff against the same dataflow with `isize`, at one worker and at eight. Over 2×
means revisiting the layout; 1.3× or under is done. `benches/dd_sum.rs`, at commit fdf0575: the
integration harness's same-scale fixture (sixteen accounts, each amount a signed count of units of
2⁻²⁰ below 2²⁰, carried by `Exact` as the double and by `isize` as the count, so both runs fold one
multiset and an integer oracle checks every `Exact` total bit for bit before timing), the design's
idiom (`explode` into the diff, then `count_total`), and two exchanges. Serialized is
`ProcessBinary`, the shape the probe ran on, where every exchanged diff crosses as its 33-byte
serde tuple even on one worker; typed is `Process`, the in-process deployment, where it crosses as
the word. A sample is one whole run, workers spawned to workers joined, ten flat samples of several
runs each after a 1 s warm-up.

| 10⁷ updates, time per run | NEON: `Exact` | NEON: `isize` | ratio | AVX2: `Exact` | AVX2: `isize` | ratio |
|---|--:|--:|--:|--:|--:|--:|
| serialized, 1 worker | 864.5 ms | 206.5 ms | 4.19× | 2,816.8 ms | 727.5 ms | 3.87× |
| serialized, 8 workers | 150.3 ms | 42.8 ms | 3.51× | 503.8 ms | 181.0 ms | 2.78× |
| typed, 1 worker | 328.1 ms | 118.0 ms | 2.78× | 1,585.4 ms | 607.4 ms | 2.61× |
| typed, 8 workers | 57.8 ms | 27.2 ms | 2.12× | 323.9 ms | 168.9 ms | 1.92× |

The claim is not met as measured. On the probe's own shape, serialized exchange, `Exact` runs
2.8–4.2× `isize`, past the 2× line on both architectures and at both worker counts; on typed
exchange it runs 1.9–2.8×, past the line on three rows of four and on the line on the fourth. Per
update on NEON at one worker, `isize` costs 12 ns typed and 21 ns serialized, `Exact` 33 ns typed
and 86 ns serialized. The split says where the gap is. Typed exchange isolates the word's own
cost: 21 ns an update over `isize` at one worker, which is the compact add (6–8 ns in the rows
above), the 32-byte diff moving through consolidation's sort as a 48-byte update tuple against
24, and the `From<f64>` that builds it. Serialized exchange adds another 45 ns an update net of
`isize`'s own serialization, which is the 33-element byte tuple written and read back one `u8`
element at a time through serde, validated on decode. Two thirds of the serialized gap is the
wire form, one third is the word.

Peak resident set of the bench process over its iterations, `/usr/bin/time -l` on macOS and
`/usr/bin/time -v` on Linux, on the serialized rows: NEON 3.82 GB (`Exact`, 1 worker), 3.31 GB
(`isize`, 1), 3.17 GB (`Exact`, 8), 2.73 GB (`isize`, 8); AVX2 1.17 GB, 1.16 GB, 1.22 GB and
1.16 GB in the same order. The process holds the 160 MB fixture and each run's input batches, and
the figure is a high-water mark across every iteration of the sitting, so it carries the
allocator's reuse policy as much as the dataflow: the Linux figures are the ones to read, and
there the diff's width costs 8 MB at one worker and 53 MB at eight.

## The wire form

Where the serialized gap is. timely's `ProcessBinary` exchange encodes every batch of updates
through bincode 1 three times over: `length_in_bytes` runs the serializer once to count,
`into_bytes` runs it again into the reserved slab, and the receiver's `from_bytes` decodes it, so
a diff's `Serialize` runs twice and its `Deserialize` once per crossing. `benches/wire.rs`, at
commit a22fae7, runs each pass alone on one batch of `((u32, ()), u64, R)` updates, the tuple
`count_total`'s `arrange_by_self` exchanges, at timely's buffer capacity for the tuple (170 updates
with a 32-byte diff, 341 with `isize`), on the SUM's rows, with every form round-tripped against
its input before timing. Four diffs: `isize`; `Exact` as the crate serialises it, the 33-element
tuple of `u8` that DESIGN.md § Wire format specifies; and two candidate forms built in the bench
as newtypes over `to_bytes`, neither in the crate. `bytes` is one `serialize_bytes` call, which
bincode 1 frames with an 8-byte length (41 bytes an `Exact`) and postcard with a 1-byte one (34).
`fields` is a 7-element tuple `(u8, u64, u64, i32, i32, i32, i32)`, which bincode's fixed-int
encoding frames as the crate's 33 bytes exactly, checked in the bench, and a varint format such as
postcard does not.

| ns per update | NEON: size | write | read | all three | AVX2: size | write | read | all three |
|---|--:|--:|--:|--:|--:|--:|--:|--:|
| `isize`, 20 bytes an update | 0.00 | 6.33 | 2.38 | 8.7 | 0.01 | 7.72 | 8.51 | 16.2 |
| `Exact`, the 33-element tuple, 45 bytes | 0.52 | 41.35 | 5.88 | 47.8 | 1.21 | 89.16 | 25.22 | 115.6 |
| `fields`, the 7-element tuple, 45 bytes | 0.53 | 17.90 | 8.34 | 26.8 | 1.21 | 29.59 | 25.03 | 55.8 |
| `bytes`, one `serialize_bytes` call, 53 bytes | 0.52 | 8.03 | 6.26 | 14.8 | 1.21 | 13.91 | 18.35 | 33.5 |

The write is the cost, and it is per call, not per byte. bincode 1 writes each element with one
`write_all` on the slab writer, a 1-byte copy for a `u8` and an 8-byte one for a `u64` at about
the same price, so the 33-element tuple pays for 33 calls where `isize` pays for one and the bytes
form for two, the length and then the 33 bytes in one copy: 41 ns against 6 on NEON, 89 against 8
on AVX2. The sizing pass is near free, bincode's size counter being a sum the compiler folds, and
the read is cheap on bincode's slice reader even one byte at a time, 6 ns for the 33-tuple on NEON
and 25 on AVX2. Net of `isize`, the tuple's three passes cost 39 ns an update on NEON and 99 on
AVX2, which is the 45 ns and the 111 ns that the end-to-end rows above attribute to the wire form
on serialized exchange at one worker (the serialized gap less the typed one); the bytes form would
leave 6 and 17, the fields form 18 and 40. The fields row's read is the bench newtype's, which
gathers the seven elements back into 33 bytes for `from_bytes`; a crate impl would build the word
from the fields directly. Neither candidate is the crate's. Both change DESIGN.md § Wire format's
clause that a format writing a tuple's elements back to back emits `to_bytes` byte for byte: the
bytes form by a length prefix in each format's own convention, the fields form by holding only on
fixed-width formats. The call is the owner's; this table is its input.

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

- NEON: Apple M5 Max, macOS, rustc 1.99.0, commit 4b533a9.
- AVX2: AMD Ryzen 7 5700U, Ubuntu 24.04, rustc 1.96.1, commit 4b533a9.
