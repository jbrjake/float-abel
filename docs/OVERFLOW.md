# Overflow inside differential-dataflow

`Exact` can panic inside a dataflow on ordinary numbers, and no check on a single value can rule
the panic out. `Wide` and `Fixed256` cannot panic once a value is past their door, in any order, on
any worker count. Which type differential carries as its diff is the owner's call; this memo lays
out what each one accepts, what it costs, and the evidence.

Every result below was measured on a scratch crate outside the tree (attached to task fl-zxpq1wz)
against float-abel at 75c25fe, differential-dataflow 0.25.1 and timely 0.31.0, on an aarch64 Mac,
with one key summed by `explode` + `count_total` on 1 and 3 workers. A figure marked *derived* was
worked out, not run.

## What fits

| type | size | what goes in | how to check one value | once in |
|---|---|---|---|---|
| `Wide` | 296 B | every double, NaN and ±∞ included | nothing to check | never panics short of 2⁶³ updates the size of `f64::MAX` |
| `Fixed256` | 32 B | finite doubles smaller than 3.96·10²⁸, on its grid: every double from 5.7·10⁻¹⁴ up is on it, a smaller one only if it is a multiple of 2⁻⁹⁶ | `Fixed256::try_from(x)` is `Ok`; or `Fixed256::quantize(x)` rounds it onto the grid first, by choice | never panics short of 2⁶⁴ updates at the ceiling |
| `Exact` | 32 B | every double on its own | **no per-value check exists**: whether a value fits depends on the other values in the same total | panics when a sum it forms needs more than 127 bits |

**The rule for `Exact`.** Write a sum out in binary. From its first 1 to its last 1 it may use
at most 127 bits. A typical decimal such as 0.1, 3.14 or 1e-12 already uses 53 of them, so two
such values in one total must be within about 10²² of each other in size:

| sum | `f64` | `Exact` |
|---|---|---|
| `1e11 + 1e-11` | `1e11`, the small one lost | fits |
| `1e12 + 1e-12` | `1e12`, the small one lost | **past the cap** |
| `1 + 1e-22` | `1` | fits |
| `1 + 1e-23` | `1` | **past the cap** |
| `1 + 2^-126` | `1` | fits: a power of two uses 1 bit, not 53 |
| `1 + 5e-324` | `1` | **past the cap** |
| `f64::MAX + f64::MAX`, then `− f64::MAX` | `inf`, stuck | fits: reads `inf`, then exactly `f64::MAX` again |

These were run through `checked_add`, which returns the cap as a value. Inside a dataflow the same
sum is a panic, because differential adds with `+=`; the 1e12 and 1e-12 pair panics there below.

Each added term can push the total's first 1 higher, so a total of *n* values narrows the
allowance by about log₂ *n* bits: with 10⁶ values in one key it is nearer 10¹⁶ than 10²² (derived).
Whole numbers and short binary fractions (0.5, 0.25) use few bits and can spread much further.

The 127 bits apply to every sum differential forms along the way, not only to the answer, which
is why one value cannot be checked alone (§ Why `Exact` panics).

## Why `Exact` panics in a dataflow

Differential adds diffs with `Semigroup::plus_equals`, which `Exact` maps to `+=`, and `+=` panics
past the cap because `plus_equals` cannot return an error. The backtrace runs `plus_equals` ←
`consolidate_updates_slice_slow` ← `ContainerChunker::finish` ← `arrange_core`: the arrangement
consolidating its input. Three kinds of sum get formed, and each must fit:

1. **A key's net change at one time.** Retracting 2¹⁰⁰⁰ and inserting 2⁻¹⁰⁰⁰ in the same epoch
   makes the change `−2¹⁰⁰⁰ + 2⁻¹⁰⁰⁰`, which does not fit even though both totals do.
2. **A key's running total at each time.** 2¹⁰⁰⁰ held while 2⁻¹⁰⁰⁰ arrives is a total that does
   not fit, even if 2¹⁰⁰⁰ leaves in the next epoch.
3. **The partial sums of a consolidation.** `consolidate_updates_slice_slow` sorts by key and
   time only (`sort_unstable_by`) and adds the diffs of a run in the order they lie, so the same
   updates panic in one arrival order and complete in another.

Five histories, every one ending at the same multiset {small}, with `|` between epochs:

| history | `Exact`, 1e12 and 1e-12 | `Exact`, 2¹⁰⁰⁰ and 2⁻¹⁰⁰⁰ | `Wide`, both pairs | `Fixed256`, 1e12 and 1e-12 |
|---|---|---|---|---|
| A: `+big` \| `−big +small` | panics | panics | exact | exact |
| B: `+big` \| `+small` \| `−big` | panics | panics | exact | exact |
| C: `+big +small −big` | panics | panics | exact | exact |
| D: `+big` \| `−big` \| `+small` | exact | exact | exact | exact |
| E: `+big −big +small` | exact | exact | exact | exact |

Each cell held on 1 and 3 workers alike. C and E are the same three updates in one epoch in two
arrival orders. `Fixed256` refuses 2¹⁰⁰⁰ at its door (`AboveCeiling`) before a dataflow starts,
which is the per-value check doing its job. Every run that completed read the same double on both
worker counts, so the order-independence of a result holds; what depends on order is whether
`Exact` gets to a result at all.

## Why one worker's panic hangs the rest

On 3 workers, worker 1 panicked. The main thread joins the workers in order (`WorkerGuards::drop`
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
else is in the total, and so on history and arrival order. `Fixed256` refuses at the door. `Exact`
places its 127 bits wherever the values are, which is what lets it take any double, and is also
why its refusal lands mid-dataflow.

## Options

1. **`Wide` as differential's diff.** Takes every double and counts NaN and ±∞; nothing to check,
   and its only limit is 2⁶³ updates the size of `f64::MAX`. Costs 296 bytes per update, 9.25×
   `Exact`'s 32, so a batch of 10⁷ updates carries 2.9 GB of diffs (derived); its speed inside
   differential is unmeasured. No `Wide × Wide`: a row's product goes in through
   `deposit_product`, refused outside 2⁻¹⁰⁷⁴ … 2¹⁰³⁷. Needs differential's weight traits on
   `Wide`; the scratch crate's newtype needed five (`IsZero`, `Semigroup`, `Monoid`, `Abelian`,
   `Multiply<isize>`).
2. **`Fixed256`, or another `Fixed` window, as differential's diff.** 32 bytes, `Copy`, a
   per-value check, and no panic after the door. Costs a declared window: a value outside is
   refused, or rounded onto the grid by `quantize` if the caller chooses; NaN and ±∞ are refused,
   not counted; no `Fixed × Fixed` into `Fixed`. Differential's `Multiply` may name a different
   output type, so `Fixed256 × Fixed256 → Wide` would be exact and panic-free (the product of two
   values on the 2⁻⁹⁶ grid below 2⁹⁵ lies on the 2⁻¹⁹² grid below 2¹⁹⁰, inside `Wide`); it is
   not built. Needs the same weight traits on `Fixed`.
3. **`Exact` with a declared window checked at the door.** Keeps `Exact`'s products and counted
   specials. To stay panic-free for 2⁶⁴ updates the window can span only 127 − 64 = 63 bits, and
   a typical decimal fills 53 of them, so the window holds such decimals across a factor of only
   2¹¹ ≈ 2,000 in size (for example 4.2·10⁶ to 8.6·10⁹), against `Fixed256`'s factor of about
   10⁴² at the same 32 bytes (derived). For sums it does strictly less than option 2, and a
   product of two values in the window leaves it.
4. **Options 1 and 2 together.** `Wide` for "any double", `Fixed` for "a known range in 32
   bytes", both door-checked; `Exact` stays for folds inside one thread, where the `checked_*`
   methods can return the cap instead of panicking.

Not recommended, each for a stated reason:

- **A wider `Exact`** (say 64 bytes and 255 bits) moves the line without removing it: still no
  per-value check, and still order-dependent at the edge.
- **A heap fallback past 127 bits** never panics, and is the arm CLAUDE.md's standing decisions
  dropped: "in general i approve dropping the heap arm", with zero-copy paramount.
- **A sticky "overflowed" state in place of the panic** is not a group, for the reason a sticky
  NaN is not (DESIGN.md § NaN and ±∞): nothing cancels it, so a retraction cannot restore the sum
  and `is_zero` cannot delete the key.

**Recommendation: option 4**, with `Wide` as the default weight for arbitrary doubles, subject to
measuring its cost inside differential first, and `Fixed256` for data with a known range. It is
the only option where a caller can know before inserting that a dataflow will not panic. Under any
option, the README tells multi-worker consumers to set `panic = "abort"`.
