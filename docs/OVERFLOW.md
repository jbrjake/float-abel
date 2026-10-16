# Overflow inside differential-dataflow

The compact `Exact` panics inside a dataflow on ordinary numbers, and no check on a single value
can rule the panic out. This memo states what fits, why the panic lands where it does, and the
ways out, each with what it costs and the evidence. The owner ruled for option 5, the heap arm,
with option 1, the banded idiom, kept as its allocation-free fast path (fl-t154dsp), and § Cost
measures the ruled 32-byte layout against the alternatives. With the `alloc` feature the heap arm
stands: every history and the seeded multiset below complete through the plain idiom, and
products fold inside both engines (tests/differential_products.rs, tests/dbsp_products.rs); the
compact form alone, which is `Exact` without `alloc`, is what the measurements describe.

Every result below was run on a scratch crate outside the tree (its sources are attached to
fl-zxpq1wz and fl-h41w0tr) against float-abel at 57e1354, differential-dataflow 0.25.1 and timely
0.31.0, on an aarch64 Mac, with `explode` + `count_total` on 1 and 3 workers, except § Cost, whose
probe ran against 02adc7c on the same engine crates. A figure marked *derived* was worked out,
not run.

## What fits

| type | size | what goes in | how to check one value | once in |
|---|---|---|---|---|
| `Wide` | 296 B | every double, NaN and ±∞ included | nothing to check | never panics short of 2⁶³ updates the size of `f64::MAX` |
| `Fixed256` | 32 B | finite doubles smaller than 3.96·10²⁸, on its grid: every double from 5.7·10⁻¹⁴ up is on it, a smaller one only if it is a multiple of 2⁻⁹⁶ | `Fixed256::try_from(x)` is `Ok`; or `Fixed256::quantize(x)` rounds it onto the grid first, by choice | never panics short of 2⁶⁴ updates at the ceiling |
| `Exact`, compact form alone (no `alloc`) | 32 B | every double on its own | **no per-value check exists**: whether a value fits depends on the other values in the same total | panics when a sum it forms needs more than 127 bits |
| `Exact`, banded (§ The banded idiom) | 32 B | every double, NaN and ±∞ included | nothing to check | never panics short of 2⁴³ live rows in one band of one key, or 2³¹ NaN or ∞ rows in one key |
| `Exact`, heap arm (`alloc`) | 32 B, plus one allocation past a 127-bit span | every double, NaN and ±∞ included, and every product | nothing to check | panics at the limb cap, sixteen products' worth of bits (2³⁴³⁶⁸ in span), reachable through nested products alone, or at 2³⁰ in the exponent |

**The rule for the compact `Exact`.** Write a sum out in binary. From its first 1 to its last 1 it
may use at most 127 bits. A typical decimal such as 0.1, 3.14 or 1e-12 already uses 53 of them, so two
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
| `2^127 + (−1)` | `2^127` | fits: 127 one-bits, though 2¹²⁷ alone needs 128 at the unit, so the add takes it in two halves |

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
fl-h41w0tr). On the heap arm every cell of the first two columns is exact, and on 1e300 and 1e-300
too: history B holds the promoted `big + small` across an epoch as `count_total`'s data, C forms it
inside `consolidate_updates` on the way to `small`, and both orders of the `repro` case
consolidate to `[small]` in the compact form
(tests/differential_products.rs, `the_five_histories` and `consolidate_updates`). `Fixed256` refuses 2¹⁰⁰⁰ at its door (`AboveCeiling`) before a dataflow starts, which
is the per-value check doing its job. Every run that completed read the same double on both
worker counts, so the order-independence of a result holds; what depends on order is whether
a plain `Exact` gets to a result at all.

## Why one worker's panic hangs the rest

On 3 workers, one worker panicked. The main thread joins the workers in order (`WorkerGuards::drop`
in timely_communication 0.31.0), so it blocks on worker 0, which keeps stepping while it waits for
progress the dead worker will never send. The process spun at about 200% CPU until killed. Built
with `panic = "abort"`, the same run exits with status 134 on 1 and 3 workers. A consumer who keeps
any type that can still panic inside a dataflow needs that profile setting, or a panic becomes a
hang. On the heap arm the panic left inside a dataflow is the limb cap and the exponent bound: a
value of sixteen products' worth of bits, reachable through nested products alone, which the
2^±300 bands of the tests sit thousands of bits below; so the setting still applies to a consumer
who can reach it, and the hang is the same.

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

Put the double's band in the key: `Exact::band(x) = ⌊(exponent of the leading bit + 1074) / 32⌋`,
0..=65 for finite values, 255 for NaN and ±∞. `explode` into `Exact` and `count_total` per
`(key, band)`, then `reduce` each key's band totals, at most 66 and typically one to three, into
one `Exact` and emit it with multiplicity one, which is the plain idiom's output. Shipped as
`BandedSum::banded_sum` on a collection of `(key, amount as bits)` rows behind `differential`
(src/exact/differential.rs):

```text
rows.explode(|(key, bits)| { let x = f64::from_bits(bits); Some(((key, Exact::band(x)), Exact::from(x))) })
    .count_total()                                        // ((key, band), band total) with isize diffs
    .map(|((key, band), total)| (key, (band, total)))
    .reduce(|_key, bands, output| {                       // a key's few band totals, not its rows
        let mut total = Exact::ZERO;
        for ((_band, band_total), mult) in bands { total += band_total * (*mult as i64); }
        if !total.is_zero() { output.push((total, 1)); }  // the zero has no row, as under count_total
    })
```

Every `Exact` diff differential then forms is a sum of values inside one band. A value in band b
has its leading bit in a 32-binade window and its lowest bit at most 52 below that, so any partial
sum of n of them, in any order, with retractions and multiplicities, spans at most 84 + log₂ n bits:
inside the 127-bit width for n < 2⁴³ live rows per (key, band). Each band is a `Fixed` window chosen
by the value itself, which is what removes the per-value check: every double fits its band, and
nothing downstream depends on what else is in the key. Specials are counted in band 255 and
projected at read, as in any `Exact`. The reduce re-folds a key's bands, not its rows, so the work
per change stays O(bands), and its cross-band total is the one value that can promote: it is
emitted as data, never consolidated. A cancellation across two bands, `2^14` against two `2^13`s,
sums to the zero, and the reduce emits nothing, as `count_total` emits nothing for a key `is_zero`
deleted.

Measured (`band`, attached to fl-h41w0tr): the ten histories above complete on 1 and 3 workers
reading the small value exactly; a seeded multiset of 7,207 updates over 10 epochs on 5 keys,
magnitudes uniform over 2^±300 with full mantissas and either sign, a third of the live rows
retracted each epoch, NaN and ±∞ on one key, 20 bands occupied, lands all 50 (key, epoch) reads on
a `Wide` oracle bit for bit on 1 and 3 workers, while the same multiset through the plain idiom
panics on 1 worker and hangs on 3 on the compact form. On the heap arm the same shape of multiset
through the plain idiom lands all 50 (key, epoch) totals on the `Wide` oracle on 1 and 3 workers,
45 or more of them heap values (tests/differential_products.rs, `the_seeded_multiset`), and
through `banded_sum` lands the plain idiom's updates exactly on both worker counts with every band
total `count_total` forms compact across the twenty bands occupied, where more than half the key
totals the reduce emits are heap values (tests/differential_banded.rs).

What it does not do: products. A row's product has a 106-bit mantissa, so a band for products
would have to be 127 − 106 − log₂ n bits wide. A fold of products inside a dataflow on the compact
form is `Wide`'s (BENCH.md § Where the width caps: 42% of same-scale products refuse in an `Exact`
fold), with `deposit_product` refused at the door past 2¹⁰³⁷. The heap arm holds that fold: a
`float × float` join folded by `count_total`, a depth-3 join chain and three rounds of
`Exact × Exact` through `iterate` land a `num-bigint` oracle in tests/differential_products.rs,
and `aggregate_linear` over `Exact::product` of two columns lands it in tests/dbsp_products.rs.

## Cost

Rough, not a BENCH.md row (`cost`, attached to fl-nhcdk92 with its sitting logs): release, one
worker, 10⁶ updates over 16 keys in one epoch, ns an update, best of ten runs on an Apple M5 Max
with the one-minute load under 3 at the start. Typed exchange is `execute_directly`, timely's
Thread allocator; serialized is `ProcessBinary(1)`, every exchanged diff crossing as its serde
bytes. Two shapes of the heap arm run beside the Copy word: the 40-byte enum, `Exact` inline
and a `Box<Wide>` past the width, and the 32-byte niche form, the compact word unchanged and a
heap value tagged by an even nonzero `lo` (a canonical compact mantissa is odd or zero) with
`hi` the pointer to one allocation of normalised limbs, in two block shapes: the sign, exponent
and counts in the word (A) or in a header at the front of the block (B). Both demote after every
heap op, so the canonical form is the smallest form that fits, and the niche form's compact
paths take every cheap path the real arm will (a write over a compact self without drop glue, a
`clone_from` that copies, a bitwise `is_zero`). Two diagnostics bracket the niche form on the band
that never promotes: the Copy word with the tag test alone, and the niche word with an empty
`Drop`.

| diff | same-scale typed | same-scale serialized | spread 2^±60 typed | spread 2^±60 serialized | mixed 2^±300 typed | mixed 2^±300 serialized |
|---|--:|--:|--:|--:|--:|--:|
| `isize` | 11.0 | 19.2 | | | | |
| `Exact`, plain (the Copy word) | 22.9 | 37.9 | panics | panics | panics | panics |
| `Exact`, banded, with the reduce | 23.4 | 41.5 | 48.9 | 67.2 | 42.0 | 59.7 |
| heap arm, 40-byte enum | 30.3 | 51.5 | 99.5 | 120.4 | 117.9 | 138.8 |
| heap arm, 32-byte niche, A | 25.6 | 44.4 | 72.8 | 93.4 | 82.6 | 104.6 |
| heap arm, 32-byte niche, B | 25.5 | 45.4 | 69.8 | 91.6 | 85.7 | 106.9 |
| niche with an empty `Drop` (diagnostic) | 25.2 | 45.1 | | | | |
| the Copy word plus the tag test (diagnostic) | 23.3 | 40.2 | | | | |
| `Wide` as the diff | 171.8 | 236.2 | 178.9 | 236.9 | 179.4 | 236.9 |

On same-scale data no heap arm promotes, so the column is the shape's own cost. The enum costs
32% over the Copy word typed and 36% serialized: a tag, a 56-byte update tuple against 48, and a
non-`Copy` clone through consolidation's sort. The niche form costs 11% typed and 17–20%
serialized (the Copy word's serialized minimum sits 2 ms under its median; on medians the ratio
is 1.17×), and the diagnostics split that: the tag test is 2%, the drop glue 1–2%, and the rest
is being a non-`Copy` type at all, a `Clone` that branches and the moves through the sort and the
merges on a type with drop glue, which no layout removes. The two block shapes are 0.1 ns apart
on that column and split the promoting bands by under 5% each way; B is the one to build, because
the heap form's dyadic amounts are variable-length limb runs that fit no word, so one block with
one header holds the whole value. On the promoting bands the niche form runs 0.70–0.76× the enum
(a shorter limb add, a smaller block) and 1.4–2.0× the banded idiom, which never promotes; the
probe allocates a fresh block for every heap result, so that is an upper bound on an arm that
adds in place. The banded idiom costs 2% over the Copy word typed and 9% serialized. An update
tuple with a `u64` key is 48 bytes with the Copy word and with either niche shape, 56 with the
enum, 312 with `Wide`. BENCH.md § The end-to-end SUM has the criterion numbers for the Copy
word's row (2.01× `isize` serialized, 2.14× typed, at one worker on this machine); the heap arm's
budget is that row within 1.12× typed and 1.20× serialized. The same section has the shipped
idiom on the real heap arm beside the plain one, criterion over whole runs of 10⁷ updates: on
same-scale data banded costs 6–14% over plain, and on the mixed 2^±300 fixture it runs in
0.54–0.64× the plain idiom's time, 1.86× faster at one worker typed, inside the 1.5–2.0× this
table predicted.

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
   fuzz target; and the 32-byte niche form measures 11% over plain `Exact` typed with no promotion
   at all and 1.4–2.0× the banded idiom where promotions happen (§ Cost). Reopens the heap-tier
   work the redesign absorbed (fl-atkbp6q's original scope), the `alloc` row and limb cap in the
   portfolio plan
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
