# Overflow inside differential-dataflow

`Exact` panics inside a dataflow on ordinary numbers, and no check on a single value can rule the
panic out. This memo states what fits, why the panic lands where it does, and the ways out, each
with what it costs and the evidence. Which one differential carries is the owner's call
(fl-t154dsp).

Every result below was run on a scratch crate outside the tree (its sources are attached to
fl-zxpq1wz and fl-h41w0tr) against float-abel at 57e1354, differential-dataflow 0.25.1 and timely
0.31.0, on an aarch64 Mac, with `explode` + `count_total` on 1 and 3 workers. A figure marked
*derived* was worked out, not run.

## What fits

| type | size | what goes in | how to check one value | once in |
|---|---|---|---|---|
| `Wide` | 296 B | every double, NaN and ±∞ included | nothing to check | never panics short of 2⁶³ updates the size of `f64::MAX` |
| `Fixed256` | 32 B | finite doubles smaller than 3.96·10²⁸, on its grid: every double from 5.7·10⁻¹⁴ up is on it, a smaller one only if it is a multiple of 2⁻⁹⁶ | `Fixed256::try_from(x)` is `Ok`; or `Fixed256::quantize(x)` rounds it onto the grid first, by choice | never panics short of 2⁶⁴ updates at the ceiling |
| `Exact` | 32 B | every double on its own | **no per-value check exists**: whether a value fits depends on the other values in the same total | panics when a sum it forms needs more than 127 bits |
| `Exact`, banded (§ The banded idiom) | 32 B | every double, NaN and ±∞ included | nothing to check | never panics short of 2⁴³ live rows in one band of one key, or 2³¹ NaN or ∞ rows in one key |

**The rule for a plain `Exact`.** Write a sum out in binary. From its first 1 to its last 1 it may
use at most 127 bits. A typical decimal such as 0.1, 3.14 or 1e-12 already uses 53 of them, so two
such values in one total must be within about 10²² of each other in size:

| sum | `f64` | `Exact` |
|---|---|---|
| `1e11 + 1e-11` | `1e11`, the small one lost | fits (126 bits) |
| `1e12 + 1e-12` | `1e12`, the small one lost | **past the cap** (132 bits) |
| `1 + 1e-22` | `1` | fits (126 bits) |
| `1 + 1e-23` | `1` | **past the cap** (130 bits) |
| `1 + 2^-126` | `1` | fits: a power of two uses 1 bit, not 53 |
| `1 + 5e-324` | `1` | **past the cap** |
| `f64::MAX + f64::MAX`, then `− f64::MAX` | `inf`, stuck | fits: reads `inf`, then exactly `f64::MAX` again |
| `2^127 + (−1)` | `2^127` | **refused** by the add, though 2¹²⁷−1 is 127 bits and `Exact::try_from(&Wide)` accepts the same value: the boundary bug, fl-h41w0tr's findings |

The bit counts were derived from the doubles' bits in integer arithmetic and match `checked_add`.
Inside a dataflow the same sum is a panic, because differential adds with `+=`.

Each added term can push the total's first 1 higher, so a total of *n* values narrows the
allowance by about log₂ *n* bits: with 10⁶ values in one key it is nearer 10¹⁶ than 10²² (derived).
Whole numbers and short binary fractions (0.5, 0.25) use few bits and can spread much further.

The 127 bits apply to every sum differential forms along the way, not only to the answer, which
is why one value cannot be checked alone.

## Why `Exact` panics in a dataflow

Differential adds diffs with `Semigroup::plus_equals`, which `Exact` maps to `+=`, and `+=` panics
past the cap because `plus_equals` cannot return an error. The set of `Exact` values is not closed
under addition, and closure is the first group axiom; the laws kit does not see this because its
`Exact` strategy draws doubles within 56 binades of each other, inside the width by construction.
Two kinds of sum get formed, and they fail differently:

1. **A key's running total at some epoch.** `count.rs:108,120` folds a key's diffs in time order,
   so 2¹⁰⁰⁰ held while 2⁻¹⁰⁰⁰ arrives is a total that is formed whatever the arrival order, and
   does not fit even if 2¹⁰⁰⁰ leaves in the next epoch. Only a wider carrier holds it.
2. **A partial sum on the way to a total that fits.** Retracting 2¹⁰⁰⁰ and inserting 2⁻¹⁰⁰⁰ in
   the same epoch makes the change `−2¹⁰⁰⁰ + 2⁻¹⁰⁰⁰`, which does not fit even though both totals
   do. Partial sums are formed in `consolidate_updates` (consolidation.rs:127: `sort_unstable_by`
   on `(data, time)`, then a fold of each run in the slice's order), the batcher's chain merge
   (merge_batcher.rs:315), the spine's chunk merges (trace/chunk/vec.rs:264 and :363, the second
   collapsing times after compaction), and the running totals in count.rs and threshold.rs. The
   same updates panic in one arrival order and complete in another.

Five histories, every one ending at the same multiset {small}, with `|` between epochs:

| history | `Exact`, 1e12 and 1e-12 | `Exact`, 2¹⁰⁰⁰ and 2⁻¹⁰⁰⁰ | `Exact` banded, both pairs | `Wide`, both pairs | `Fixed256`, 1e12 and 1e-12 |
|---|---|---|---|---|---|
| A: `+big` \| `−big +small` | panics | panics | exact | exact | exact |
| B: `+big` \| `+small` \| `−big` | panics | panics | exact | exact | exact |
| C: `+big +small −big` | panics | panics | exact | exact | exact |
| D: `+big` \| `−big` \| `+small` | exact | exact | exact | exact | exact |
| E: `+big −big +small` | exact | exact | exact | exact | exact |

Each cell held on 1 and 3 workers alike. C and E are the same three updates in one epoch in two
arrival orders. The panic needs no dataflow: differential's public `consolidate_updates` over the
three updates of C panics, and over E's order consolidates to `[small]` (`repro`, attached to
fl-h41w0tr). `Fixed256` refuses 2¹⁰⁰⁰ at its door (`AboveCeiling`) before a dataflow starts, which
is the per-value check doing its job. Every run that completed read the same double on both
worker counts, so the order-independence of a result holds; what depends on order is whether
a plain `Exact` gets to a result at all.

## Why one worker's panic hangs the rest

On 3 workers, one worker panicked. The main thread joins the workers in order (`WorkerGuards::drop`
in timely_communication 0.31.0), so it blocks on worker 0, which keeps stepping while it waits for
progress the dead worker will never send. The process spun at about 200% CPU until killed. Built
with `panic = "abort"`, the same run exits with status 134 on 1 and 3 workers. A consumer who keeps
any type that can still panic inside a dataflow needs that profile setting, or a panic becomes a
hang.

## Why no 32-byte word takes every double

A 32-byte word has 2²⁵⁶ possible values. Exact sums of doubles reach from 2⁻¹⁰⁷⁴ to past 2¹⁰²⁴,
which takes at least 2,098 bits to tell apart. So a 32-byte type must refuse something; the only
choice is where. A refusal at the door is checkable per value and cannot depend on order, because
every later sum stays inside a bound the door fixed. A refusal mid-dataflow depends on whatever
else is in the total, and so on history and arrival order. `Fixed256` refuses at the door. A plain
`Exact` places its 127 bits wherever the values are, which is what lets it take any double, and is
also why its refusal lands mid-dataflow. The banded idiom below keeps the 32-byte diff and moves
the refusal out of the dataflow by choosing the window per value.

## The banded idiom

Put the double's band in the key: `band(x) = ⌊(exponent of the leading bit + 1074) / 32⌋`, 0..=65
for finite values, 255 for NaN and ±∞. `explode` into `Exact` and `count_total` per `(key, band)`,
then `reduce` each key's band totals, at most 66 and typically one to three, into a `Wide` and
emit the double:

```text
rows.explode(|(key, bits)| { let x = f64::from_bits(bits); Some((((key, band(x)), ()), Exact::from(x))) })
    .count_total()                                        // ((key, band), band total) with isize diffs
    .map(|(((key, band), ()), total)| (key, (band, total)))
    .reduce(|_key, bands, output| {                       // a key's few band totals, not its rows
        let mut sum = Wide::default();
        for ((_band, total), mult) in bands { sum += Wide::try_from(total).unwrap() * (*mult as i64); }
        output.push((sum.to_f64().to_bits(), 1));
    })
```

Every `Exact` diff differential then forms is a sum of values inside one band. A value in band b
has its leading bit in a 32-binade window and its lowest bit at most 52 below that, so any partial
sum of n of them, in any order, with retractions and multiplicities, spans at most 84 + log₂ n bits:
inside the 127-bit width for n < 2⁴³ live rows per (key, band). Each band is a `Fixed` window chosen
by the value itself, which is what removes the per-value check: every double fits its band, and
nothing downstream depends on what else is in the key. Specials are counted in band 255 and
projected by the `Wide` fold. The reduce re-folds a key's bands, not its rows, so the work per
change stays O(bands).

Measured (`band`, attached to fl-h41w0tr): the ten histories above complete on 1 and 3 workers
reading the small value exactly; a seeded multiset of 7,207 updates over 10 epochs on 5 keys,
magnitudes uniform over 2^±300 with full mantissas and either sign, a third of the live rows
retracted each epoch, NaN and ±∞ on one key, 20 bands occupied, lands all 50 (key, epoch) reads on
a `Wide` oracle bit for bit on 1 and 3 workers, while the same multiset through the plain idiom
panics on 1 worker and hangs on 3.

What it does not do: products. A row's product has a 106-bit mantissa, so a band for products
would have to be 127 − 106 − log₂ n bits wide. A fold of products inside a dataflow is `Wide`'s
(BENCH.md § Where the width caps: 42% of same-scale products refuse in an `Exact` fold), with
`deposit_product` refused at the door past 2¹⁰³⁷.

## Cost

Rough, not a BENCH.md row (`cost`, attached to fl-t154dsp): release, one worker, timely's Thread
allocator, 10⁶ updates over 16 keys in one epoch, best of three runs, ns an update, on a host at
load 4, so the ratios are what to read. The heap-arm proxy is the design's dropped arm as a
newtype in the scratch crate: `Exact` inline, promoted to a `Box<Wide>` when the compact add
refuses, demoted after every heap op so the canonical form is the smallest tier that fits; the
real arm held a normalised `Vec<u64>` (two allocations a promotion, a shorter limb add), so the
proxy is the cost shape, not the implementation.

| diff | same-scale (units of 2⁻²⁰ below 2²⁰) | spread 2^±60 | mixed 2^±300 |
|---|--:|--:|--:|
| `isize` | 12.8 | | |
| `Exact`, plain | 24.5 | panics | panics |
| `Exact`, banded, with the reduce | 26.5 | 50.1 | 43.0 |
| heap-arm proxy | 33.5 | 101.1 | 119.4 |
| `Wide` as the diff | 198.3 | 179.9 | 180.2 |

On same-scale data the heap-arm proxy never promotes, so its 37% over plain `Exact` is the enum
tag, the 40-byte update against 32, and a non-`Copy` clone through the sort; on the spread and
mixed bands the promotions add the allocations and the wide adds, and it runs 2.0–2.8× the banded
idiom. The banded idiom costs 8% over plain `Exact` on same-scale data. BENCH.md § The end-to-end
SUM has the criterion numbers for the plain `Exact` row (2.01× `isize` serialized, 2.14× typed, at
one worker on this machine).

## Options

1. **The banded idiom, shipped.** `Exact` stays the 32-byte diff; the crate ships the band
   function and a differential operator for the idiom, with the adversarial test above in the
   suite. Costs 2% over the plain idiom on same-scale data, a second small arrangement for the
   reduce, and a documented idiom in place of a bare `count_total`. Takes every double; nothing to
   check; sums only.
2. **`Wide` as differential's diff.** Takes every double and counts NaN and ±∞; nothing to check;
   products through `deposit_product`, refused at the door past 2¹⁰³⁷. Costs 296 bytes per update,
   8× the plain `Exact` row above and 16× `isize`. Needs differential's weight traits on `Wide`;
   the scratch crate's newtype needed five (`IsZero`, `Semigroup`, `Monoid`, `Abelian`,
   `Multiply<isize>`).
3. **`Fixed256`, or another `Fixed` window, as differential's diff.** 32 bytes, `Copy`, a
   per-value check at the door, no panic after it. Costs a declared window: a value outside is
   refused, or rounded onto the grid by `quantize` if the caller chooses; NaN and ±∞ are refused,
   not counted; no `Fixed × Fixed` into `Fixed`. Needs the same weight traits on `Fixed`.
4. **Options 1 and 2 together.** The banded idiom for sums of any doubles at 32 bytes a diff,
   `Wide` for product folds and for a caller who wants no idiom; `Exact`'s checked API for folds
   inside one thread.
5. **The heap arm, restored.** The original design's `Exact`: the compact word inline, a boxed
   wide tier past the width, canonical form the smallest tier that fits, the one panic a limb cap
   reachable only through nested products. Takes every double and every product with nothing to
   check and no idiom; the only option that holds a fold of products or a `float × float` join
   chain. Costs: `Exact` stops being `Copy` and a plain word, so it leaves the engines' `Copy`
   slots and a mapping cast, which is why `Wide` and `Fixed` exist for those; an `alloc`
   feature returns and the no-allocator build loses the arm; the wire form becomes variable
   length, and with it the serde and rkyv shapes, the golden hash's `Exact` leg and the decoder
   fuzz target; and the proxy measures 37% over plain `Exact` with no promotion at all and 2–2.8×
   the banded idiom where promotions happen (§ Cost). Reopens the heap-tier work the redesign
   absorbed (fl-atkbp6q's original scope), the `alloc` row and limb cap in the portfolio plan
   that fl-cfx001b asks to remove, and the standing decision "every type is a `Copy` plain word
   and nothing allocates".

Not recommended, each for a stated reason:

- **A wider `Exact`** (say 64 bytes and 255 bits) moves the line without removing it: still no
  per-value check, and still order-dependent at the edge.
- **A sticky "overflowed" state in place of the panic** is not a group, for the reason a sticky
  NaN is not (DESIGN.md § NaN and ±∞): nothing cancels it, so a retraction cannot restore the sum
  and `is_zero` cannot delete the key.
- **A declared window on `Exact` itself** holds decimals across a factor of only about 2¹¹ in
  size at 127 − 64 bits of headroom (derived); the banded idiom is that window chosen per value.

**Recommendation: option 4**, with the banded idiom as the documented SUM idiom. It is the fastest
panic-free option measured on every band, keeps every standing decision (32-byte `Copy` word,
nothing allocates, specials counted, `Mul<Exact>` for a row's product), and is the only 32-byte
option in which a caller has nothing to check. Option 5 is the one to take if a fold of products
or a `float × float` join chain inside a dataflow is a requirement, because nothing else holds
one. Under any option, the README tells multi-worker consumers to set `panic = "abort"`.
