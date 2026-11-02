# float-abel — design

`float-abel` holds sums (and products) of IEEE doubles **exactly**, as a canonical abelian group with
a ring on top, so an incremental engine can add, retract and re-merge floats and land the same bits
under any order, worker count, merge tree, consolidation timing or architecture. It ships three exact
storage tiers and one batch-only lossy tier:

- `Exact`: a 32-byte word that carries the same-scale case and the special counts inline, and
  promotes to one allocation for everything past its 127-bit width, demoting as soon as a result
  fits again.
- `Wide` and `Fixed`: two `Copy` fixed-width carriers for engines whose aggregate slot can't hold a
  pointer.
- `Binned`: a batch-only adaptive accumulator on the same limb grid.

It also ships a property-test kit for the group laws, and feature-gated implementations of
differential-dataflow's and DBSP's weight traits. Either engine then takes `Exact` as a diff type the
way it takes `isize`, with no change to either project.

One correction to the usual premise comes first. The Demmel–Nguyen *adaptive* binned accumulator is
reproducible for a one-shot reduction, but it is **not a group**. Its state depends on the largest
value ever seen, so `a + b − b ≠ a`, retractions leave residue, and consolidation timing changes
results ([below](#why-the-adaptive-binned-accumulator-is-not-a-group-sp-not-a-group)). The
*fixed-anchor* form of the same idea is a group, because it is a fixed-point integer, and exact
accumulation is trivially a group. So the hierarchy is:

- exact by default;
- windowed fixed-point where constant size or a significance floor is wanted;
- Kulisch width where a `Copy` slot must cover the whole `f64` range.

The cheap common case, Shewchuk's role, is played by an exact 127-bit dyadic. It is faster to
canonicalise than a double-double, and bit-identical on every architecture because it never executes
a floating-point add.

## The problem

An incremental engine keeps a running answer and applies changes to it: an insert adds a weight, a
delete adds its negation, and a background merge may add any subset in any order. That only works if
the weight type is an exact abelian group. `f64` addition isn't one: it is not associative, so the
same multiset summed in two orders lands two answers, and `a + b − b` need not be `a`. Today an
engine that wants float weights does one of three things:

- scales to fixed-point integers and picks a range in advance;
- keeps the float in the data and re-folds the whole group on every change, which is O(group) and
  still differs in the last bits across worker counts;
- accepts nondeterminism and compares outputs at a tolerance.

Feldera, for one, refuses to make `SUM` over `DOUBLE` linear and documents why: the incremental
answer *"will produce incorrect results"* (`udf.md`). A float in the diff, held exactly, removes all
three workarounds.

## What it makes possible

| becomes possible | how, in differential-dataflow's own operators | replaces |
|---|---|---|
| `SUM`/`AVG`/weighted count over float columns at O(δ), retractions exact | `explode` the value into the diff, then `count_total`; a delete is `−1 × Exact` through the same `Multiply` | a re-fold per change, a refusal, or a quantisation chosen in advance |
| Bit-identical output across worker counts, exchange timing, merge trees and consolidation timing | the group laws; nothing else | comparison at a float tolerance |
| Hashable, signable, replayable outputs | a consequence of the row above: the output is a function of the input, not of the schedule | nothing; a float output can't be diffed between two deployments today |
| Float-weighted graph algorithms as native incremental dataflows | weights in the diff, per-node totals consolidating exactly, `iterate` with a float carrier (it needs `Abelian`) | fixed-point ranks and a range chosen in advance |
| Sparse linear algebra maintained incrementally, correctly rounded | `join` multiplies (`Exact × Exact` is exact and distributes) and `consolidate` sums, so an SpMV's entries are exact dot products | not expressible with exact results |
| Expected-count and probabilistic semantics | weights as probabilities or log-likelihoods, summed and retracted exactly | integer counts only |
| Exact moments that survive deletes | a `(count, Σx, Σx²)` tuple diff, with `deposit_product` for the square | recompute, or drift |
| A SQL engine whose incremental `SUM(DOUBLE)` equals its batch answer | the first row, behind a planner | the documented refusal |

**The honest boundary.** The group makes **sums and products** exact. Division, normalisation,
`exp`/`log` and a non-dyadic constant such as ⅓ are not group operations. They stay on the read side
(`div_f64` divides the exact numerator once), or ride an exact rational elsewhere, or take a fixed
round count. Nothing here makes arbitrary floating-point arithmetic reproducible. It makes the part an
incremental engine *accumulates* reproducible, which is the part that breaks.

## What an incremental engine asks of a weight {#sp-laws}

| law | where differential-dataflow 0.24.0 relies on it |
|---|---|
| associative and commutative, **bit for bit** | `consolidate_updates_slice_slow` sorts by `(data, time)` with `sort_unstable_by`, so the order in which equal-key diffs are added is unspecified (`consolidation.rs:119`); `VecMerger` adds in chain order (`merge_batcher.rs:287-289`) |
| identity and **exact inverse** (`a + (−a) = 0`, and `a + b − b = a` bit-exact) | `is_zero` deletes the update (`consolidation.rs:65`); `negate` in `iterate`/`Variable`, `antijoin`, `reduce_abelian` |
| canonical `Eq`/`Ord`/`Hash` | `UpdsBuilder::seal` dedups by `==` (`ord_neu.rs:217-226`); `Update::Diff: Ord` is structural (`mod.rs:74,214`) |
| `Multiply` distributes, exactly | `join.rs:3-5`: *"we will repeatedly evaluate (a + b) * c as (a * c) + (b * c)"* |

DBSP asks the same of a `WSet` weight: an exact `is_zero` decides which tuples survive consolidation,
and `aggregate_linear` scales and retracts its accumulator by integer weights.

## Why the adaptive binned accumulator is not a group {#sp-not-a-group}

ReproBLAS's indexed type keeps K bins whose exponents are pinned relative to an **index** I, chosen
from the largest magnitude seen so far. When a larger value arrives, the index rises and the lowest
bin is dropped. Bits below it are truncated at the same boundary for every input, past and future,
which is exactly why the result is order-independent. Addition on `(I, bins)` is associative and
commutative. But its zero is not unique: `a + (−a) = (I_a, 0)`, and
`(I_a, 0) + (J, b) = (max(I_a, J), trunc(b))`, which is not `(J, b)` when `I_a > J`. So there is no
inverse in the group sense, and the consequence is concrete:

- Insert `1e30`, insert `1e-10`, retract `1e30`. The state is `1e-10` truncated to the precision the
  `1e30` index allowed. At K=3, W=40 that is `1e30·2⁻⁸⁰ ≈ 8e5` of granularity, so the surviving
  `1e-10` reads as **zero**. A batch system that never saw the `1e30` reads `1e-10`. The incremental
  answer is a function of history, not of the current input.
- In differential-dataflow the failure is sharper. If `is_zero((I, 0))` is `true`, consolidation
  *deletes* it, and the other diffs under the key stop being truncated. The answer now depends on
  *when* consolidation happened to run.

The fixed-anchor variant (bins pinned to absolute exponents, no index) is a two's-complement
fixed-point integer with a declared window. That is a group, and it is `Fixed` below.

## The trilemma

Constant size, full `f64` dynamic range, exact abelian group: **pick two.** A constant-size exact
group over the full range does not exist (pigeonhole). Every honest design sits at one of the corners,
and the crate ships each:

| corner | type | size | domain | `Copy` | ring |
|---|---|---|---|---|---|
| exact + full range | `Exact` | 32 B; one allocation past a 127-bit span, with `alloc` | every `f64`, specials counted; every sum and product up to the limb cap | no: `Clone` is a copy in the compact form | yes: `×Exact` with dyadic special amounts, `×i64` |
| exact + constant size (wide) | `Wide` | 296 B | every `f64` | yes | `×i64` only |
| exact + constant size (window) | `Fixed<const LSB: i32, const LIMBS: usize>` | 8·LIMBS B (32 at the default) | finite doubles exactly on the 2^LSB grid inside the window; the rest **refused at the door**, or **quantised** by declaration | yes | `×i64` only |
| constant size + full range: **lossy, batch-only** | `Binned<const K: usize>` | 16·K B + index + counters (80 at K=3) | every `f64` | yes | none, and **no group**: `Add` only, no `Neg`, no engine weight traits |

The three groups carry both engines' weight traits behind the `differential` and `dbsp` features
(§ differential-dataflow, § DBSP): `Exact` with the ring, `Wide` and `Fixed` with `Multiply` by
`i64` and `isize` alone, since neither multiplies by itself. `Wide` is the diff for an engine whose weight
slot must be `Copy` or that casts over a mapping, and `Fixed` the same with a declared window, its
door check per value at the `explode`.

The last corner is the adaptive accumulator, and the previous section is why it is not a group. It
ships anyway, for **batch work only**. An engine that also runs one-shot folds over tiles wants
ReproBLAS's shape there: fewest bytes, fewest ops, order-independent. `Binned` lives on `Wide`'s limb
grid so the two convert in O(K). It implements addition (the Demmel–Nguyen reduction) and nothing
that resembles an inverse. Under the `differential` and `dbsp` features it implements none of their
weight traits, so a compile error, not a convention, keeps it out of the incremental calls.

## Exactness bounds {#sp-bounds}

- A finite `f64` spans 2⁻¹⁰⁷⁴ … 2¹⁰²⁴: 2098 bits. A sum of *n* of them needs 2098 + ⌈log₂ n⌉ bits.
  34 × 64 = 2176 gives 78 bits of headroom, i.e. ≥ 2⁶³ additions of the largest finite value
  (bitrep's bound, reused as-is).
- A product of two doubles has a 106-bit mantissa and spans 2⁻²¹⁴⁸ … 2²⁰⁴⁸: 4196 bits (Kulisch's
  4288 = 4196 + 92 headroom). A product of *k* sums grows linearly in *k*, so `Exact`'s memory is
  bounded by `k · 2148 + 64` bits for the deepest `float × float` join chain in a dataflow, and by
  2176 bits when weights only ever multiply integers. The limb cap, `EXACT_MAX_LIMBS` at 16 × 2148
  bits, is what makes the heap form safe against adversarial input: it is decided on the operands'
  spans before anything is allocated, except at the cap's edge, where a carry or a borrow settles
  whether the result fits and the result is computed into one limb past the cap and refused after
  normalisation. So a sum or product is refused exactly when its result would span more than the
  cap, and the group is closed under retraction up to it: every value the operations or the
  decoder produce retracts and scales by one (tests/exact_cap.rs, `exact_cap_closed_*`, against
  a `num-bigint` oracle, and the laws kit's closure and group laws on a strategy at the edge,
  tests/laws_cap.rs). Without `alloc` the width caps instead, and the memory is 32 bytes
  whatever the input.
- Every exponent is bounded: a held value is `m · 2^exp` with |exp| ≤ `EXACT_MAX_EXP` = 2³⁰, checked
  after every operation that moves an exponent, in every build. A sum's exponent stays inside its
  inputs' range plus the mantissa width, and a product of two doubles lies within 2^±2148, so only a
  chain of nested products reaches the bound: about a million products of the smallest subnormal.
  Past it is the cap, as past the limb cap. The limb cap bounds width and this bounds position; they
  are independent, and the number is as large as keeps every intermediate inside `i64` and every
  stored exponent inside `i32`, with room to spare on both sides.
- A compact 127-bit mantissa holds any exact sum whose bit-span is ≤ 127. A column of 10⁷ terms
  whose magnitudes spread over 2^±20 spans ≈ 40 + 53 + 24 = 117 bits, so it never promotes
  (measured below). Two full-mantissa decimals stay in the word while they are within about 10²²
  of each other in size: `1e11 + 1e-11` spans 126 bits and `1e12 + 1e-12` spans 132
  (docs/OVERFLOW.md § What fits), and a key of *n* rows narrows the allowance by about log₂ *n*
  bits. A uniform spread over 2^±300 is the adversary, and 77% of its pairwise adds
  promote (measured). Real columns look like the former; the heap form exists for the latter, and
  a `SUM` over it is correct on the plain idiom and allocation-free on the banded one
  (§ differential-dataflow), which keeps every consolidated diff inside one 32-binade band.
- `Fixed<−96, 4>`: unit 2⁻⁹⁶ ≈ 1.3·10⁻²⁹. 64 bits of headroom leave a magnitude ceiling of
  2⁹⁵ ≈ 4·10²⁸. Every double with |x| ≥ 2⁻⁴⁴ ≈ 5.7·10⁻¹⁴ is on the grid, and a smaller one is on it
  iff its trailing mantissa bits are zero. Money, counts, measurements and probabilities above 10⁻¹³
  are exact. Anything else is refused by name, or quantised by declaration.

## NaN and ±∞ {#sp-specials}

A group can't hold a sticky NaN, since nothing cancels it. It can hold **counts**. The state carries
the finite sum `f` plus a special vector `(n, p, q)`: how many NaN, +∞ and −∞ inputs are in the
multiset, signed so that a retraction subtracts. Addition, negation and integer scaling are
componentwise, so the group laws are inherited from ℤ³ ⊕ D (D = the dyadic rationals). Projection to
`f64`, applied once at read time:

```
n ≠ 0                     → NaN
p ≠ 0 and q ≠ 0           → NaN        (+∞ and −∞ both present)
p > 0 → +∞ · p < 0 → −∞   ·   q > 0 → −∞ · q < 0 → +∞
otherwise                 → RNE(f)     (canonical zero is +0.0; the sign of zero is not carried)
```

Specials are counted and projected, never refused, on `Wide` and `Exact`. So a sum over a column
containing NaN reads NaN, as SQL engines answer, and retracting the NaN row restores the finite sum
exactly.

For `Exact`, which multiplies, the product is the monoid algebra
D[M] of the four-element commutative monoid M = {1, P, Q, N}. `1` is the finite unit; `P·P = Q·Q = P`
and `P·Q = Q` (the sign group of the infinities, with `P` its identity); `N` absorbs. On the
coefficients `(f, n, p, q)`:

```
(f, n, p, q) · (g, n′, p′, q′) =
  ( f·g,
    f·n′ + g·n + n·(n′ + p′ + q′) + n′·(p + q),   -- anything times NaN is NaN
    f·p′ + g·p + p·p′ + q·q′,                      -- (+∞)(+∞) = (−∞)(−∞) = +∞
    f·q′ + g·q + p·q′ + q·p′ )                     -- (+∞)(−∞) = −∞
```

A monoid algebra is associative, commutative and bilinear by construction, so
`(a+b)·c = a·c + b·c`, and a retraction's product cancels its insertion's. The amounts are dyadic:
the compact form holds them as `i32` counts inline, and a product that gives a special a
non-integer amount, as `2.5 · NaN` does, or a count past `i32`, promotes to the heap form, whose
amounts are normalised dyadics as wide as the finite part, and demotes once the amounts are counts
again. Without `alloc` that product is the cap, by name. A row's product deposit, `product(a, b)`,
counts once per row by the sign rule, as `Wide` does. A finite negative factor
flips an infinity's sign through the projection rule above. The one divergence from IEEE follows from
bilinearity: an exact zero annihilates, so `0 · ∞` and `0 · NaN` project as `0`, not NaN. `Wide`
scales its integer counters by `i64` only. `Fixed` refuses non-finite input outright, because its
contract is *exact or refused*.

## The types

### Exact {#sp-exact}

The engine weight: a differential-dataflow diff, a DBSP weight, and the carrier for an exact
reference oracle, in a 32-byte word with two forms.

```
#[repr(C)] struct Exact {               // 32 bytes, no padding; Clone, not Copy
    lo: u64, hi: u64, exp: i32,         // compact: m = (hi:lo) as signed 128-bit, value = m · 2^exp
    nan: i32, pos_inf: i32, neg_inf: i32,   //   with the special vector (n, p, q) as signed i32 counts
}                                       // heap: lo = block capacity << 1 (even, nonzero), hi = pointer,
                                        //   the four trailing fields zero
```

The compact form is a canonical dyadic of at most 127 bits with `i32` counts inline: mantissa odd or
the zero form `(0, 0, 0)`, with the counts as they are. Its canonical mantissa is odd or zero, so an
even nonzero `lo` is no compact value, and the heap form takes that niche: `lo` carries the
capacity of one allocation, `hi` its pointer, and the block holds a four-entry header (sign,
exponent, limb count) for the finite part and the three special amounts, then the four runs of
normalised limbs, no leading zero limb, the lowest set bit at the exponent. The heap form is
reached by promotion only, when a result's finite part spans more than 127 bits or an amount is not
an `i32` count (a count past `i32`, or a dyadic amount such as `2.5 · NaN`), and every operation
demotes a result that fits the compact form again, so canonical form is **the smallest form that
holds the value**. Structural `Eq` is therefore numeric equality across both forms, `Hash` agrees,
and `Ord` is numeric on the finite part then `(n, p, q)` lexicographic, a total order. `Clone` is a
copy in the compact form and one allocation in the heap form, `Drop` frees the block, and
`is_compact()` reports the form. The layout is load-bearing and gets a `const` assertion: six
fields, 32 bytes, 8-aligned, and a differential update tuple with a `u64` key stays 48 bytes.
`src/exact/heap.rs` is the one module beside `src/simd/` that lifts `#![deny(unsafe_code)]`, each
block stating its invariant: the tag bit decides the form, a heap `hi` is a pointer that module
allocated with the capacity in `lo`, no other code reads a heap word's fields, and an archived
heap word's offset reaches the block its `Serialize` wrote beside it. The fields are private
because a word with an even mantissa is not a compact value: bytes from outside go through the
validating decoder.

The heap form is why `Exact` is `Clone` and not `Copy`: a word that can hold a pointer has drop
glue, so it leaves an engine's `Copy` weight slot and a cast over a mapping, and `Wide` and
`Fixed` are the `Copy` words for those (§ Wide, § Fixed), `Fixed` for a declared window besides.
What the trade buys is that every sum and every product is held: a dataflow with `Exact` as the
diff cannot panic on its input short of the two bounds below, and a sum of doubles nears neither.

The compact fast paths are the whole cost on data that never promotes: `+=` on a compact word runs
the compact add in place with no drop glue, `clone_from` between compact words is a copy, `is_zero`
is a test of the bits (the zero is the compact zero word), and `Eq` compares the words first.
Measured against the `Copy` word's own binary on same-scale data in a differential `SUM`, one
sitting, the form costs 1.08× typed and 1.09× serialized (BENCH.md § The end-to-end SUM, The heap
arm's budget; the probe put it at 1.11× and 1.17–1.20×, docs/OVERFLOW.md § Cost): the tag test,
the drop glue, and the moves of a non-`Copy` type through consolidation's sort, none of them
removable by the layout.

The caps are the limb cap (`EXACT_MAX_LIMBS`, 16 × 2148 bits ≈ 4.3 KB per run, § Exactness bounds,
a `const` raised by editing it) and the exponent bound (`EXACT_MAX_EXP`, 2³⁰). Past either the
operators panic with a named invariant, because `plus_equals` can't return an error to
differential-dataflow, and the checked methods (`checked_add`, `checked_sub`, `checked_mul`,
`checked_mul_i64`, `checked_deposit`, `checked_deposit_product`) return it as `PastCap` with the
state as it was. `From<f64>` and `product(a, b)` are total. Without the `alloc` feature the compact
form stands alone and the width (a finite magnitude whose span from its lowest set bit to its
highest exceeds 127 bits, so a sum is refused on what it is, not on what an operand needed on the
way) and the counts (an `i32`, never `i32::MIN`, so negation is total; and a non-integer amount)
are its caps, `PastCap::Width` and `PastCap::Count`; `PastCap` carries all four variants on every
feature set, and the two builds differ in which are ever returned. `to_bytes` encodes the compact
form in place and panics by name on the heap form, `encode` writes either form and `from_bytes`
reads both (§ Wire format); `specials()` returns the `(n, p, q)` counts as `i64` when every amount
is an integer `i64` holds, and `None` for a dyadic amount or one past `i64`, which only the heap
form has.

### Wide {#sp-wide}

`Copy` and `repr(C)`: `[u64; 34]` two's complement in units of 2⁻¹⁰⁷⁴ then `[i64; 3]` special
counters, 296 bytes with no padding and no invalid bit pattern, so an engine's cast over a
little-endian mapping is sound; a `const` assertion pins the size and alignment, and `from_parts` is
the safe inverse of `limbs` and `specials`. Its limb layout is bitrep's, so bitrep's finite-only
conformance vectors apply to `Wide`'s limbs directly. It is a group with `Mul<i64>`, and it never allocates; `From<f64>` is total, and the one
refusal is `deposit_product` on a product outside its window, by name: a set bit below 2⁻¹⁰⁷⁴ is
`OutOfWindow::OffGrid`, a magnitude at or past 2¹⁰³⁷ is `OutOfWindow::AboveCeiling`, and the state is
untouched. Behind `differential` it is a differential-dataflow diff, the `Copy` one: `IsZero`,
`Semigroup`, `Monoid`, `Abelian`, `Multiply<i64>` and `Multiply<isize>` on either side, and
`From<i8>` for `count_core`'s and `distinct_core`'s outputs (§ differential-dataflow). There is no
`Multiply<Wide> for Wide`, because the product of two words has no carrier, so a `join` of two
`Wide`-weighted collections does not compile and a row's product enters the diff through
`deposit_product` at the door. Behind `dbsp` it is a `DBWeight` the same way (§ DBSP), its
archived form the word itself, for the weighted batch and `aggregate_linear`'s accumulator; not a
`RingValue`, so a stream's `neg`, `minus` and `differentiate` are `Exact`'s alone. `Wide` has the
*fastest* add of any tier; its cost is bytes, not time. That suits a per-key aggregate slot, and
not a per-update diff, where a batch of 10⁷ updates would carry 2.9 GB of weights: as the diff of
a differential `SUM` over 10⁷ same-scale updates it runs 4.4–5.1× the `Exact` diff and 9.5–13.7×
`isize`, 179 ns an update typed at one worker against `Exact`'s 35 (BENCH.md § The end-to-end
SUM), so it is the diff where the slot must be `Copy` and not otherwise.

### Fixed {#sp-fixed}

`Copy` and `repr(C)`: `Fixed<const LSB: i32, const LIMBS: usize>` is `[u64; LIMBS]` two's complement
in units of 2^LSB, every bit pattern a value, so an engine's cast over a little-endian mapping is
sound; `from_limbs` is the safe inverse of `limbs`, and a `const` assertion pins `Fixed256`'s size and
alignment. `try_deposit(x) -> Result<(), OutOfWindow>` accepts a finite double iff it is exactly a
multiple of 2^LSB and |x| < 2^(64·LIMBS − 65 − (−LSB)). `quantize(x) -> f64` is round-to-nearest-even
onto the grid, for callers who *declare* a floor. It is deterministic per input, so it applies
identically on every engine before the deposit. Group ops are infallible inside the documented budget
of 2⁶⁴ deposits at the ceiling, and exhausting the headroom is a named panic in every build, release
included, because a wrapped sum is silently wrong. `Fixed256 = Fixed<−96, 4>` is the named default.
Behind `differential` it carries the weight traits `Wide` does (§ Wide), and behind `dbsp` it is a
`DBWeight` as `Wide` is, both as the declared window: the door check is per value,
`Fixed::try_from(x)` inside the `explode` or the aggregate's closure refusing by name before the
engine sees the row, so nothing downstream depends on what else is in the key; `From<i8>`, which
`count_core` and `distinct_core` put out, is pinned by a `const` assertion to the windows that hold
every `i8`, a grid at or below one with a ceiling above 2⁷, and `Fixed::<1, 4>::from(1i8)` does not
build.

### Binned {#sp-binned}

Batch-only. `K` bins on `Wide`'s limb grid, each an independent signed accumulator held carry-save as
a `u64` limb and an `i64` carry (`carry · 2⁶⁴ + limb`), an `index: i16` (the `Wide` limb number of
the top bin) and the `[i64; 3]` counters: `repr(C)`, 16·K + 32 bytes with the index padded to a
word, 80 at K=3. A deposit takes the input's magnitude apart at `Wide`'s limb boundaries, adds each
slice to its bin with the input's sign, and drops whatever falls below the lowest bin, so it
truncates toward zero. A larger input raises the index and drops the lowest bins. That is the
Demmel–Nguyen update on integer limbs. A bin's overflow stays in its own carry and never crosses
into the bin above, because a carry that had crossed would survive the drop of the bin it came from
while the same inputs deposited after the rise would lose it, and the fold would depend on the
order. With the bins independent, each holds the sum of its slices over the multiset whatever the
order or the merge tree, and so does the index, so `Add`, which aligns to the higher index and adds
bin-wise, is associative and commutative: a fold is order-independent, and any merge tree lands the
same bits. The carries are the one budget, `2⁶³` wraps of a bin, and past it the operators panic by
name.

The error bound, derived the way the paper derives its own, is
`|S − Σxⱼ| ≤ n · 2^(−64(K−1)) · max|xⱼ|` plus one final rounding: the lowest kept bit lies at most
64(K−1) below the top bit of the largest input, and each input loses less than one unit of it. At
K=3 that is `n·2⁻¹²⁸·max|xⱼ|`, tighter than the paper's `n·2⁻⁸⁰` at the same three words, because a
limb is 64 bits where a double bin is 40. `deposit_product(a, b)` and
`deposit_products(&[f64], &[f64])` slice the exact 106-bit product onto the bins the same way, with
the product's sign, into up to three bins. The unit is the floor of every window, so the bits below
2⁻¹⁰⁷⁴ that `Wide` refuses as `OffGrid` are dropped here like any other below the lowest bin, a
product wholly below the unit deposits nothing, and the bound for products carries that floor:
`n · max(2^(−64(K−1)) · max|aⱼbⱼ|, 2⁻¹⁰⁷⁴)`. The one refusal is the ceiling, `Wide`'s 2¹⁰³⁷, as
`AboveCeiling` with the state untouched, because under it the top bin is `Wide` limb 32 at most and
2⁶⁴ such products, where that bin's carry runs out, still fit `Wide`'s width; `deposit_products`
checks every pair first, so a refusal deposits nothing. `into_wide()` places the K bins at `index`,
exactly.
`Wide::to_binned::<K>()` keeps the top K limbs of the magnitude; it is lossy, and named so. There is
no `Neg`, no `Sub`, no `Semigroup` and no `HasZero`, and a `trybuild` compile-fail test pins that.
There is no wire form: a batch result crosses a boundary as the `Wide` it converts to.

## Operations and conversions {#sp-ops}

`Default, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash`; `Add, AddAssign, Sub, SubAssign, Neg,
Sum`; `Mul<i64>`, `Mul<isize>`; for `Exact` also `Mul<Exact>`. `From<f64>` is total on `Exact` and `Wide`, with
specials counted; `TryFrom<f64>` on `Fixed`. `deposit_product(a: f64, b: f64)` deposits the exact
106-bit product, the join-fold and dot-product shape, without a float multiply anywhere. It is
refused by name on `Fixed` and `Wide` when the product lies outside the tier's window: a set bit
below the unit is `OutOfWindow::OffGrid`, a magnitude at or past the ceiling is
`OutOfWindow::AboveCeiling`, and the state is untouched. `Fixed` also refuses a special factor as
`NotFinite`, where `Wide` counts it. `Binned` truncates below the unit as it truncates below its
window and refuses only the ceiling, `Wide`'s, by the same name, with `deposit_products` checking
every pair before it deposits any. `Exact` is the tier that holds every product: `deposit_product`
is total there, with only the named cap, and `checked_deposit_product` returns the cap instead.
`is_zero()`, `is_finite()` and `specials()` read the state.

Conversions between tiers: `Wide: From<Fixed<..>>` is exact, pinned by a `const` assertion to the
`(LSB, LIMBS)` pairs `Wide` holds; `Exact: TryFrom<&Wide>` and `Exact: TryFrom<&Fixed<..>>` take
every state, into the heap form past the width, and refuse only the exponent bound (without
`alloc`, the width too); `Wide::try_from(&Exact)` refuses below the unit or at the ceiling, where
`deposit_product` would, with a dyadic amount off its counters' grid and an amount past `i64` above
their ceiling. `Binned<K>::into_wide`
(exact) and `Wide::to_binned::<K>` (lossy) are named methods, and nothing lossy has a `From`. Every
tier has the bulk entry points `extend_from_slice(&[f64])` and `deposit_products(&[f64], &[f64])`;
they are the SIMD surface.

## Reads and rounding {#sp-rounding}

`to_f64()` rounds to nearest-even in integer arithmetic, with subnormals and overflow handled.
`to_f32()` rounds once from the exact state, never via `f64`. `try_to_f64()` returns `None` if the
value is inexact. **`div_f64(n: u64)`** is the correctly rounded quotient of the exact sum by an
integer, which is what an `AVG` calls, so every engine divides the same exact numerator once.

Correct rounding is proved against an independent oracle: a committed vector file,
`tests/data/rounding_vectors.json`, written by `scripts/gen_rounding_vectors.py` from Python's
`fractions.Fraction`. `f64` is `float(...)`, correctly rounded by CPython. `f32` is the Fraction
rounded to odd into a double and packed with `struct`, which then rounds once, because an odd-rounded
intermediate with spare bits makes the second rounding exact, where packing `float(...)` would
double-round at every `f32` tie. A third, integer rounder must agree with both or the generator dies.
The vectors cover ties at every tier boundary, subnormal results, overflow to ±∞, and the `to_f32`
cases where rounding via `f64` first would differ. A vector enters a tier through `Wide::from_parts`
or `Fixed::from_limbs` and the conversions, so every read is checked on a state that holds the value
exactly. bitrep's finite-only conformance vectors check `Wide`'s limbs, and bitrep itself is a
dev-dependency differential oracle for `Exact::to_f64` on random multisets. `Fixed256`, `Wide` and
`Exact` agree bit for bit on their shared domain.

## Wire format {#sp-wire}

One canonical little-endian encoding per type, a leading version byte then the body, emitted both by
the `serde` feature and by `to_bytes`/`from_bytes` without it:

- `Exact` in its compact form as its 32 bytes, two limbs, the exponent and three counts, which is
  that form's in-memory layout; in its heap form, under its own first byte, as the block the word
  holds, four header limbs (sign, limb count, exponent) for the finite part and the three amounts
  then the four runs of normalised limbs, which is the block's in-memory layout and its rkyv
  archive too. `to_bytes` is the compact form's fixed array, `encode` either form's bytes behind
  `alloc`, and `from_bytes` reads both;
- `Wide` as 272 limb bytes then 24 counter bytes, which is its in-memory layout;
- `Fixed` as its limbs, which is its in-memory layout.

The version byte names the type and its form as well as the revision (`0x01` `Exact`, `0x02`
`Wide`, `0x03` `Fixed`, `0x04` the heap `Exact`), so a `Fixed256` body is never read as an `Exact`
though both are 33 bytes, and a later revision of a type's encoding takes a fresh value. Each body
is the type's `repr(C)` bytes on a little-endian machine, with the size and alignment pinned by
`const` assertions, so an engine that frames its own extents casts a mapping to `Wide` or `Fixed`
without a copy, every bit pattern of theirs being a value. `Exact`'s bytes from outside still pass
the validating decoder, because an even mantissa is not a value.

**Decoding validates canonical form and refuses anything else.** A non-canonical value smuggled
through a batch would break `Eq`-substitutability in differential-dataflow's builder, so a decoder
that accepted one would be a correctness hole, not a leniency. A refusal is a typed error,
`WireError`, never a panic. The heap decoder checks the header's limb counts against
`EXACT_MAX_LIMBS` before the length and before it allocates, because an untrusted size gets a
ceiling, validates every run over the bytes, and refuses a body whose value fits the compact form,
so one value has one encoding across the forms; the archive's reader applies the same rules to a
block. Under `serde`, `Exact` and `Wide` serialise as one byte string, the
`to_bytes` bytes handed to `serialize_bytes`, so a format frames them with its own length (a `u64` on
bincode 1, a varint on postcard, an array of numbers on serde_json) and the serializer runs about
once per value rather than once per byte, which is what an engine's exchange pays per update, three
passes over; `tests/wire.rs` pins the framing with postcard, which is maintained. `Fixed` serialises
as a fixed-length tuple of its bytes, as a `[u8; N]` does, because its length is not a constant the
impl can spell, so a format writing elements back to back emits its `to_bytes` byte for byte. Through
differential-dataflow the encoding rides whatever timely frames exchanged data with, which at timely
0.31 is bincode 1.3; through DBSP it is rkyv 0.7, the word with a heap block beside it (§ DBSP).
The format freezes at 1.0 and is documented in `FORMAT.md`, in bitrep's style, with vectors.

## Crate posture {#sp-posture}

- MIT OR Apache-2.0. Edition 2021.
- `#![deny(unsafe_code)]` crate-wide, lifted only inside `src/simd/`, where the intrinsics live, and
  `src/exact/heap.rs`, where `Exact`'s heap form owns its block and its rkyv archive reaches it,
  with a `// SAFETY:` line on every block. With the `simd` and `alloc` features off, the crate is
  `unsafe`-free end to end.
- `#![no_std]`. `Wide`, `Fixed` and `Binned` are `Copy` plain words that never allocate; `Exact`
  allocates only in its heap form, behind `alloc`, and is the compact form alone without it. `std`
  (the default) implies `alloc` and adds the test kit and `std::error::Error`.
- **Zero required dependencies.** Everything else is a feature.
- MSRV **1.74** for the default feature set, asserted in CI on exactly that set. The `differential`
  feature inherits differential-dataflow 0.25's declared `rust-version = "1.86"`. The `dbsp` feature
  builds on rustc 1.99.0 and on 1.96.1, which dbsp declares; 1.97 and 1.98 ICE inside it, so its CI
  job pins 1.99.0.
- The crate lists **itself as a dev-dependency** with `serde`, `differential`, `laws`, `simd` and
  `arbitrary` on: `float-abel = { path = ".", features = [...] }` under
  `[dev-dependencies]`. That is
  the one Cargo spelling that turns features on for a bare `cargo test` with no flag. `dbsp` stays out
  of that list because it needs its own toolchain.
- One integration-test target, `tests/main.rs`, with each topic a `mod`.

## Features {#sp-features}

| feature | pulls in | provides |
|---|---|---|
| `alloc` | — | `Exact`'s heap form: promotion past the width, dyadic special amounts, the limb cap as the one cap beside the exponent bound |
| `std` (default) | `alloc` | `std::error::Error` impls, the bench harness |
| `serde` | serde 1 | `Serialize`/`Deserialize` in the canonical encoding, validated on decode; either form of `Exact` with `alloc` |
| `differential` | differential-dataflow 0.25 + `serde` | the differential-dataflow adapter |
| `dbsp` | dbsp 0.367 + rkyv 0.7 (`std`, `size_64`) + feldera-size-of 0.1.7 + `std` | the DBSP adapter |
| `laws` | proptest | the laws kit |
| `simd` | `std` (runtime detection is `std::arch`'s macro) | runtime-dispatched NEON/AVX2/AVX-512 bulk kernels, the `unsafe` beside the heap form's |
| `arbitrary` | arbitrary 1 | fuzz inputs |

## Adapters

The differential-dataflow and DBSP adapters live in this crate behind features, because the crate owns
the types and the orphan rule permits `impl ForeignTrait for LocalType`. Neither project needs a
change. An adapter for any other engine lives in that engine, against the public API, and the crate
names no engine of its own.

### differential-dataflow {#sp-differential}

A probe enum with a `Box` variant was compiled and run against differential-dataflow 0.24.0 (timely
0.30.0) on `CommunicationConfig::ProcessBinary(3)`, with serialized exchange. It exercised `map`,
`flat_map`, `explode`, `concat`, `negate`, `consolidate`, `arrange_by_key/self`, `reduce` (as `R`
and as `R2`), `count`, `distinct_core::<R>`, `threshold`, `semijoin`, `antijoin`, `iterate`, and a
three-type join `R × isize → R3`. Six negative probes each failed on the predicted bound. The surface
at 0.25.1 is unchanged.

What `Exact` implements under `differential`:

| trait | why |
|---|---|
| `IsZero`, `Semigroup` (`plus_equals`) | every operator; `is_zero` is what deletes an update |
| `Monoid` (`zero`), `Abelian` (`negate(&mut self)`) | `negate`; `iterate`/`Variable` (`iterate.rs:81` requires `Abelian`); `antijoin`; `reduce`/`threshold`/`count_core` **outputs** (`collection.rs:739,846,915`: `R2: Ord + Abelian`) |
| `Multiply<Exact, Output = Exact>`, `Multiply<isize>`, `Multiply<i64>`, and `impl Multiply<Exact> for isize`/`i64` (orphan-legal, since `Exact` is local) | `join`/`join_core`/`explode`/`join_function` accept any `Output: Semigroup`; **`antijoin` alone requires `Multiply<R2, Output = R>`** (`collection.rs:1211`), satisfied with `R2 = isize` |
| `From<i8>` | `count_core`/`distinct_core` outputs |
| `Ord + Debug + Clone + 'static + Send + Serialize + Deserialize` | `ExchangeData` (`lib.rs:88-93`, timely `lib.rs:128`); `Update::Diff: Ord` is structural in the default `OrdValSpine` (`mod.rs:74,214`) |

Not needed on the default path: `Columnation`, `columnar::Columnar`, `Copy`. The experimental
`dd::columnar` layout is out of scope; note that it assumes `Multiply<Output = Self>`.

`Wide` and `Fixed<LSB, LIMBS>` carry the same table less the ring row, in src/wide/differential.rs
and src/fixed/differential.rs: `IsZero`, `Semigroup`, `Monoid`, `Abelian`, `Multiply<i64>` and
`Multiply<isize>` with the orphan `Multiply<Wide> for i64`/`isize` and `Fixed`'s twins, and
`From<i8>`; `ExchangeData` is the serde impls behind `serde`, which `differential` implies. Neither
word has `Multiply<Self>`, so a `join` of two collections weighted by one word does not compile,
and a row's product enters the diff through `deposit_product` at the door. They are the diff for
the two cases `Exact` leaves: a weight slot that must be `Copy` or a cast over a mapping, which is
`Wide`, and a declared window, which is `Fixed`, its `try_from` inside the `explode` the per-value
door check. tests/differential_words.rs runs the five histories of docs/OVERFLOW.md on one worker
and three with `Wide` on every pair and `Fixed256` on the 1e12 pair, `Fixed256` refusing 2¹⁰⁰⁰ and
1e-300 at the door before any dataflow starts, a fold of row products through the door against a
`num-bigint` oracle with a retraction epoch, a retraction epoch netting to nothing, `join`,
`semijoin` and `antijoin` with `isize` in either order, and `count_core` and `distinct_core`
putting `From<i8>` out.

The idiom the crate's example documents: `explode` the float column into the diff and `count_total`,
i.e. `collection.explode(|row| Some((key, Exact::from(row.x))))` then `.count_total()`. Retractions
arrive as `−1 × Exact` through the same `Multiply<isize>`. The multi-worker integration test runs on
three workers and covers every operator listed above, an `antijoin` with `isize`, and a retraction
epoch whose netted output is empty.

What the heap form buys inside the engine, tests/differential_products.rs on one worker and three
with serialized exchange: the five histories of docs/OVERFLOW.md and its seeded 2^±300 multiset
complete through the plain idiom and land a `Wide` oracle per (key, epoch); a `join` of two
`Exact`-weighted collections lands on `Multiply<Exact, Output = Exact>`, so a `float × float` join
folded by `count_total` and a depth-3 join chain land a `num-bigint` oracle bit for bit, with
retractions; and `iterate` carries a float weight through three rounds of `Exact × Exact`, the
159- and 212-bit weights of the later rounds riding the feedback loop as heap values. A fold whose
partial sums span any width holds, with nothing to check at the door. The one panic left inside
a dataflow is the limb cap, sixteen products' worth of bits, reachable through nested products
alone, and the exponent bound; a consumer who can reach either keeps `panic = "abort"`, because
one worker's panic otherwise leaves the others spinning (docs/OVERFLOW.md § Why one worker's panic
hangs the rest).

The banded idiom, docs/OVERFLOW.md § The banded idiom, is the `SUM` fast path on the heap form.
`Exact::band(x)` is ⌊(exponent of the leading bit + 1074) / 32⌋, `0..=65` for a finite value and
255 for the specials, and `BandedSum::banded_sum` on a `VecCollection<(K, u64), isize>` of
`(key, amount as bits)` rows runs `explode` into the diff and `count_total` per `(key, band)`,
then a `reduce` of each key's band totals, at most 66, into one `Exact` emitted with multiplicity
one: the plain idiom's output exactly, a zero total emitting nothing. Inside a band every diff
spans at most `84 + log₂ n` bits, inside the compact word for `n < 2⁴³` live rows per
`(key, band)`, so nothing promotes under consolidation; the cross-band sum the reduce forms may
promote and is emitted as data, never consolidated. tests/differential_banded.rs lands the
fifteen histories and the seeded multiset through both idioms on one worker and three, every band
total compact where most key totals are heap values, and a cancellation across two bands emitting
nothing. The idiom sums: a row's product has a 106-bit mantissa and no band narrow enough, so a
`join` of two weighted collections stays on the plain idiom.

### DBSP {#sp-dbsp}

A probe weight with `Exact`'s layout was compiled and run against published dbsp on rustc 1.99.0
(its source is attached to the store's "Ship the dbsp feature" task), and the adapter's own tests
run at the feature's floor, dbsp 0.367.0, on the same compiler. The probe covered consolidation
of `0.1 + 0.2` on an `OrdIndexedWSet` and its exact cancellation on merge with `−0.2, −0.1`, and
`aggregate_linear` scaling by `MulByRef<i64>` and retracting a group on an exact zero. Three
deliberate breaks failed on `IsNone`/`SupportsRoaring`, `OrdRepr`, and `HasOne`/`Mul`/`Neg`
respectively; the third reaches `neg`, `minus` and `differentiate` on a custom-weight stream through
the leaf's `Neg`, which bounds the weight on `ZRingValue`, and no public operator names `ZRingValue`
directly. Nothing in dbsp is sealed: `DBData`, `DBWeight`, `Data`, `Weight`, `WeightTrait` and
`Erase` are all blanket impls over public bounds. A custom-weight batch has no `iter`, which
`IndexedZSetReader` keeps for `ZWeight`; its cursor reads it, with the typed weight derefing.
tests/dbsp.rs runs the probe's cases against the shipped adapter, and tests/dbsp_products.rs
promotes an account's accumulator past the width through `aggregate_linear`, scales it by the
`ZWeight`, demotes it on retraction, and folds a row's product, `Exact::product` of two columns,
against a `num-bigint` oracle: the dot-product accumulator a UDA over two columns holds.

| requirement | source | how |
|---|---|---|
| `Clone + Default + Debug + Eq + Ord + Hash + Send + Sync + 'static` | `DBData` | already |
| `SizeOf` | feldera-size-of 0.1.7 (`package = "feldera-size-of"`) | derive |
| rkyv 0.7 `Archive + Serialize + Deserialize`, with `Archived<Exact>: Ord` | `ArchivedDBData` | `Archive` and `Serialize` hand-written: the archived form is the 32-byte word, for a compact value the word itself (the wire body without its version byte), for a heap value the word with its heap tag in `lo` and in `hi` the offset of the block `Serialize` wrote 8-aligned beside it, the heap wire body behind its tag. `Deserialize`, `Eq` and `Ord` on the archived form hand-written through the wire decoder's two validating constructors, one per form, which refuse a non-canonical word with a panic naming the invariant, because dbsp's deserialiser has no error channel; a heap block is bounded by the cap before it is read and validated before it is used, and the comparisons read it in place without a copy. The one `unsafe fn`, `resolve`, and the block's reader live in `src/exact/heap.rs` |
| `OrdRepr<Exact> for Archived<Exact>`, agreeing with `Ord` | `ord_repr.rs`: *"an implementation that disagrees with Ord silently returns the wrong rows"* | hand-written through the same constructors, because `Ord` is hand-written; tested against `Ord` on 10⁵ values over mixed tiers, both forms and specials |
| `IsNone` + `SupportsRoaring` | `utils/{is_none,supports_roaring}.rs` | dbsp's own exported `never_none!` and `never_roaring_filter!`, which expand to what `#[derive(feldera_macros::IsNone)]` emits, so feldera-macros is not a dependency |
| `HasZero` (exact `is_zero`), `AddByRef`/`AddAssignByRef` | `algebra.rs`; blankets over `&T: Add` and `T: AddAssign<&T>` | std ops |
| `MulByRef<i64, Output = Exact>` | `aggregate_linear`'s accumulator bound; retractions arrive as `f(v).mul_by_ref(&w)` with negative `w` | explicit |
| `Neg`, `NegByRef`, `Mul`, `MulByRef<Exact>`, `HasOne` (= `ZRingValue`) | only for `minus`/`neg`/`differentiate` on a custom-weight stream | already a ring |

`Wide` and `Fixed<LSB, LIMBS>` are `DBWeight`s too, in src/wide/dbsp.rs and src/fixed/dbsp.rs:
`HasZero`, `MulByRef<i64>` and `MulByRef<isize>` written out, `AddByRef`, `AddAssignByRef` and
`NegByRef` from dbsp's blankets over the words' by-reference operators, `SizeOf` counting nothing
past the word, and `IsNone` and `SupportsRoaring` as `Exact`'s. The storage traits are rkyv's
derives on the struct with the archived form declared to be the word, `#[archive(as = "...")]`: a
`Copy` `repr(C)` word with no invalid bit pattern is its own archive, so storage writes the wire
body behind its version byte, `Ord` on the archived form is the word's, `OrdRepr` is that same
`cmp`, `Deserialize` is a copy, and no raw memory is touched. Neither word has `HasOne` or
`Mul<Self>`: the product of two words has no carrier, so neither is a `RingValue`, and `neg`,
`minus` and `differentiate` on a stream they weight do not compile, where the weighted batch, its
merge and `aggregate_linear` do, which is the `SUM` shape. tests/dbsp_words.rs runs the batch and
its merge, the aggregate with the `ZWeight` scaling and the zero group deleted, `Fixed256`'s door
inside the aggregate's closure, the archived form as the wire body ordering as `Ord` on 2 × 10⁴
values of either word, specials riding the `Wide` accumulator, and `MulByRef` as the group's
scaling.

What is **not** reachable from outside DBSP, stated so nobody files it as a crate bug: `join`,
`distinct`, `weighted_count`, every input handle, and `aggregate_linear`'s *input* are fixed to
`ZWeight = i64`. A custom weight is the accumulator or the output, which is precisely the `SUM`
shape. SQL `SUM(DOUBLE)` can't be made linear from outside, because the compiler refuses it by design.
Feldera's sanctioned route is a `CREATE LINEAR AGGREGATE` UDA whose accumulator type is ours (the
compiler wraps it in `Tup3<acc, i64, i64>`), and the crate ships that UDA as its DBSP example.

Constraints: the `dbsp` CI job pins rustc 1.99.0, and the push gate runs the feature's tests and
its example on that toolchain when it is installed. rkyv is used with
`default-features = false, features = ["std", "size_64"]` to match dbsp's. The feature takes
`dbsp = "0.367"`, a caret on the minor like differential's, so a consumer's own 0.367.x unifies
with it; the probe runs at the floor of the range, and moving the floor means re-running the probe
there first. The feature's closure is 513 crates against the crate's 114, carrying nine licences the
crate's own allow-list lacks (BSD-3-Clause, ISC, Zlib, MIT-0, CC0-1.0, BSL-1.0, CDLA-Permissive-2.0,
bzip2-1.0.6 and MPL-2.0, across 22 crates), 27 duplicate majors and rkyv 0.7's RUSTSEC-2026-0235, so
`deny.toml` checks every feature but `dbsp` and `deny-dbsp.toml` states that closure row by row, run
by the dbsp CI job and the push gate.

## The laws kit {#sp-laws-kit}

`float_abel::laws` (feature `laws`) is a module any engine author can point at *any* weight type,
theirs or ours. It checks with proptest:

- associativity and commutativity, bit-exact under 1,000 shuffles of a fold;
- identity;
- inverse: `a + (−a) == zero` and `a + b − b == a`, bit-exact;
- canonicality: structural `Eq` ⇔ numeric equality, with `Hash` consistent;
- distributivity of `Multiply`;
- **consolidation-timing invariance**: the same multiset summed in random partitions, with `is_zero`
  pruning between partitions, lands the same bits;
- **closure**: every pairwise sum, difference and product the strategy draws, and the fold in a
  thousand orders, returns. The first group axiom, and the one a type with a cap breaks: the kit
  catches the panic at the operation and reports it with the operands as the witness, so a capped
  type is seen whether or not it declares its cap. The other laws hold on any strategy inside a
  type's domain, so this one runs on the values the engine will see: `Exact` passes on every
  binade with the specials in, on row products, and at the limb cap's edge, on a strategy at the
  check's own budget under the cap (a weight under 2^(cap − 5), since a case folds thirty-two of
  them, with the multiplier inside those five bits), where a case's fold climbs onto the cap's
  last bit and retracts across it; a control refusing that bit fails there and holds on every
  binade, which is why the edge has a strategy of its own (tests/laws_cap.rs). The compact form
  alone, `Exact` without `alloc`, is not closed, by design, which a compact-only control asserts
  past its width.

The entry points are `check_abelian_group`, `check_ring`, `check_canonical`,
`check_consolidation_invariance` and `check_closure`, generic over a `Strategy`. A type opts in through `laws::Weight`
(the operators by value, structural `Eq` and `Hash`, `zero` and `is_zero`); each check takes a
proptest `TestRunner`, and `laws::runner(cases, seed)` is the fixed-seed one CI uses. A check returns
proptest's minimal failing case under a reason that names every law it broke with a witness for
each, rather than panicking, so the control's failures are asserted as values. The laws are the
contract, and the kit is how a stranger checks their own type against differential-dataflow's or
DBSP's expectations without reading either codebase. The kit runs against every tier, and against a
naive `f64` wrapper as a negative control that must **fail** associativity and inverse, and a
compact-only `Exact` that must fail closure past its width; a control that passes means the kit is
blind.

## Speed {#sp-simd}

Every tier is associative and commutative, so **lane-parallel partial accumulators** are legal
everywhere. N lanes each fold a strided subset of a tile, the N partials merge at the tile's end, and
the laws guarantee the same bits as a scalar fold. The golden hash asserts that across the scalar path
and every SIMD kernel on every target. That is the whole trick; the rest is which kernel each tier
runs per lane.

| tier | bulk kernel, per lane per value | scatter? | target, per core |
|---|---|---|---|
| `Fixed<LSB, LIMBS ≤ 4>` | integer field extract (`and`, `shr`); one per-lane variable shift per limb (`vshlq_u64` / `vpsllvq`), mask, add into carry-save limbs; sign via separate positive and negative accumulators merged at the tile's end | no: every lane updates the same `LIMBS` registers | ≤ 1.0 ns/value NEON, ≤ 0.6 ns AVX2 |
| `Binned<K>` | the `Fixed` kernel over `K` limbs at a **per-tile index**: one vector `max` pass fixes the window for the tile, and tiles then merge adaptively. The alternative, ReproBLAS's pinned-exponent FP kernel (`q = (M + x) − M` per bin, bins converted to integer limbs at the tile's end), lands the fold's bits only as `2K` half-bins of 32 bits with a truncation fix-up per level, and at that shape measures 1.6× (NEON) and 1.8× (AVX2) slower than the integer kernel, its nearest-rounding form slower too (BENCH.md § Binned kernel (ii)), so the integer kernel is the one kernel | no | ≤ 0.5 ns/value |
| `Wide` | a tile's min/max-exponent pass picks a window; a tile whose span fits 4 limbs runs the `Fixed` kernel *exactly* and lands in `Wide` at the end; a wider tile falls back to the scalar deposit (ExBLAS's expansion-then-superaccumulator hierarchy, on integer limbs) | no, on the fast path | ≤ 1.5 ns/value on real columns |
| `Exact` as an engine weight | **scalar by nature**: one value at a time through `plus_equals`, nothing to vectorise; the compact tier is one 128-bit add plus a `trailing_zeros` | — | 3.7 ns measured; ≤ 4 ns |

Mechanics: `std::arch` intrinsics behind runtime detection (`is_x86_feature_detected!("avx2")`,
AVX-512 where present, NEON unconditionally on `aarch64`), with `#[target_feature]` kernels in
`src/simd/{neon,avx2,avx512}.rs`. One scalar reference kernel *is* the definition of correct.
`unsafe` is confined to those files. NEON numbers come from an Apple M5 Max (128-bit vectors, two
`u64` lanes) and AVX2/AVX-512 numbers from an x86 CI runner; they are reported side by side, never
blended.

Products are not vectorised. `deposit_products` on every tier is the scalar check-then-deposit of
the exact 106-bit product, the dot-product shape, exact and bit-identical, at 7–23× the same tier's
bulk deposit kernel (BENCH.md § The product-kernel row); a kernel waits on a consumer with the bulk
dot-product shape, the owner's hold (fl-mj3tcta), since an engine's join multiplies one row at a
time.

What "as fast as possible" is bounded by: a naive vectorised `f64` sum runs near 0.1 ns/value and is
wrong in its last bits. Reading a double and placing 53 bits at a data-dependent offset costs at least
three integer ops per limb per value, so an exact tier's floor is a small multiple of the naive sum,
not parity with it. The targets above are 4–8× the naive vectorised sum for `Binned`, and 10–15× for
`Wide`. They are targets: the benchmarks measure them, and the kill conditions say what happens if
they are missed.

## Hardening {#sp-hardening}

- **No `unsafe` outside `src/simd/` and `src/exact/heap.rs`, and no panics on input.** Every
  constructor is total or returns `Err`, and group ops never fail. `Fixed` states its deposit budget
  and `Exact` its memory bound. The one panic is the named `Exact` cap.
- **Decode refuses non-canonical bytes**, and nothing allocates a size taken from untrusted input
  above the cap. `cargo-fuzz` targets cover the decoder (`decode_robustness`), sums against a
  `num-bigint` rational oracle (`sum_vs_oracle`), and subtraction round-trips (`sub_roundtrip`), run
  nightly for 10 minutes per target. Every crash found is committed as a regression test. `cargo miri
  test` runs in CI.
- **Supply chain:** zero required dependencies; `cargo-deny` for licenses, advisories and duplicate
  majors; `cargo-semver-checks` from the first published release on; signed tags; `cargo publish
  --locked` from CI with provenance; a `SECURITY.md` with a disclosure address; the MSRV asserted.

## Bit-identical across architectures {#sp-architecture}

No floating-point instruction executes in any group op or in `to_f64`: only `to_bits`, shifts and
adds with carry. So rounding modes, FTZ/DAZ, x87 excess precision and FMA contraction can't reach the
result. CI proves it the way bitrep does. One SHA-256 is taken over every tier's canonical bytes
(`to_bytes`, and `encode` for an `Exact` in its heap form) and reads (`to_f64`, `try_to_f64`,
`to_f32`, `div_f64`) for every vector in the rounding and conformance sets and for 10⁴ seeded random
multisets, each run through `Wide`, `Fixed`, `Exact` and `Binned` with every refusal entering by
name, the heap form entering through promoted sums, a fold of row products and a dyadic special
amount; `Binned` has no wire form, so its `index`, `limbs` and `carries` enter with the `Wide` it
converts to. The transcript is `tests/golden/`, its own workspace member,
because the main test target links engine dependencies wasm32 cannot run. It is asserted equal to
`tests/data/golden.sha256` on x86_64 Linux and Windows, aarch64 macOS and Linux, wasm32 (wasmtime),
and, through QEMU via `cross`, riscv64gc and s390x. s390x is big-endian, which is what catches a
byte-order slip in `from_bytes`. The committed hash is the claim, and the workflow asserts it on every
push.

## Verification

| level | what | how |
|---|---|---|
| laws | associativity/commutativity under shuffles, identity, exact inverse, canonicality, distributivity, consolidation-timing invariance, closure; the three negative controls fail | proptest, 256 cases in CI with a fixed seed, 32 at the limb cap's edge |
| rounding | `to_f64`/`to_f32`/`div_f64` against the CPython-generated vectors; ties, subnormals, overflow | committed vector file |
| cross-tier | `Fixed`, `Wide` and `Exact` agree bit for bit on their shared domain; `Exact::to_f64 == bitrep::SumF64::value` on random finite multisets | 10⁶ random multisets |
| decode | canonical round-trip; non-canonical bytes refused; a refusal names the first rule FORMAT.md gives; serde agrees with `from_bytes` | unit tests, `cargo-fuzz` nightly (`decode_robustness`) |
| sums | `Wide`'s limbs and counts equal a `num-bigint` sum of the multiset bit for bit, inserts and retracts alike; every read equals it rounded in independent integer arithmetic; `Fixed256` accepts exactly its window and holds the same integer; `Exact` agrees wherever the width holds the running sum; `Binned` lands the truncated sum; `a + b − b == a` on every tier over any two canonical `Exact` words and any two multisets | `cargo-fuzz` nightly (`sum_vs_oracle`, `sub_roundtrip`), ten minutes a target |
| UB | the lib's unit tests, the no-feature build and the suite under the interpreter, with the statistical sweeps at a hundredth, the cap-edge oracle cross on six of its eleven shapes, trybuild, the timely topics and the laws kit out, and every bulk entry on the portable kernel | `cargo miri test` in CI |
| architecture | one SHA-256 over every tier's bytes, reads and refusals on the vectors and 10⁴ seeded multisets, on 7 targets | the golden-hash workflow |
| SIMD | every kernel equals the scalar reference bit for bit on the vectors and 10⁴ seeded multisets (specials and subnormals included); lane-count independence at 2, 4 and 8 partials | the golden hash covers every kernel |
| batch tier | a `Binned` fold bit-identical under 1,000 shuffles and 1–64 merge partitions; equal to the sum of the inputs truncated below the window, and so within `n·2^(−64(K−1))·max\|x\|` of `Exact`; `Binned<K>` fails to compile as a differential-dataflow diff or a DBSP weight | `trybuild` |
| differential-dataflow | the three-worker integration test; join `Exact × isize`; antijoin with `isize`; `iterate` | feature `differential` |
| DBSP | `OrdIndexedWSet` consolidation and cancellation; `aggregate_linear` scale and retract; the UDA example builds | feature `dbsp`, rustc 1.99.0 |

## Measured, on a prototype {#sp-measured}

A prototype, not the crate: rustc 1.97.0, `-O -C target-cpu=native`, 10⁷ inputs, one thread, Apple
M5 Max. It implements the compact tier as `(i128, i32)` and a 34-limb Kulisch add, and it checks
round-trip, `a + b − b == a` and `a + (−a) == 0` bit-exact before timing. The crate's benchmarks
re-take this table.

| operation | magnitudes 2^±20 | magnitudes 2^±300 |
|---|--:|--:|
| `f64 +=` | 0.51–0.57 ns | 0.50 ns |
| compact deposit (`from_f64` + add) | 4.3–4.5 ns | 5.5 ns |
| compact add, both operands built | 3.7–4.0 ns | 3.6 ns |
| `to_f64`, correctly rounded | 4.0–4.4 ns | 6.2 ns |
| 34-limb wide add (`Wide`'s shape) | 2.1–2.3 ns | 1.2 ns |
| pairwise adds needing the exact tier | **0 of 10⁶** | 767,429 of 10⁶ (77%) |
| full fold leaving the compact tier | **never** | 88% of deposits |

Two readings. The compact tier is a same-scale optimisation, and the exact tier is mandatory for
correctness, not a rare fallback. And `Wide` is the *fastest* add.

## Targets and kill conditions {#sp-kills}

Targets, measured into `BENCH.md` and never gated, stated on the same-scale band: compact add ≤ 5 ns
on NEON, met at 4.9 ns a deposit and 3.8 ns an add with both operands built, with AVX2's 15.9 and
10.9 ns standing where its integer floor puts them; `to_f64` at its measured 9.8 ns (NEON) and
17.8 ns (AVX2), and `Wide`'s add at 15.6 and 29.5 ns, both accepted by the owner's ruling
(fl-cbtfmqz) as what the un-inlined calls reach, with a carry chain on `u128` intermediates and a
two-limb read path the leads should a consumer need less; and a differential-dataflow end-to-end
`SUM` over 10⁷ updates at 1.8–2.2× the `isize` baseline on the `Copy` word, accepted by the
owner's ruling (fl-bbevtyv), and at 1.9–2.4× on the heap form (NEON, BENCH.md § The end-to-end
SUM, 9714758), the 3 ns an update a non-`Copy` word costs, accepted by the owner's ruling
(fl-vy2nxc8). `BENCH.md` names the machine and the commit for every number, and
`docs/CROSSOVER.md` carries the adoption question, the group size past which a change through the
engine beats re-summing its floats.

| claim | measured by | kill / done |
|---|---|---|
| the compact form carries same-scale data inside the width | promotion rate on the same-scale fixture | > 5% promotions → widen the word before the adapters build on the type |
| the type is cheap enough to be a diff | differential-dataflow end-to-end `SUM`, 10⁷ updates, 1 and 8 workers, vs `isize` | 1.8–2.2× on the `Copy` word and 1.9–2.4× on the heap form, accepted (fl-bbevtyv, fl-vy2nxc8): the layout stands, the 32-byte word is the price of a float diff that needs no pre-chosen scale, the non-`Copy` form's 3 ns an update the price of holding every sum and product, and the adoption test is the crossover in `docs/CROSSOVER.md` |
| SIMD earns its feature | bulk deposit, each kernel vs the scalar reference, same-scale fixture, NEON and AVX2 | < 3× over scalar → 0.1.0 ships scalar-only, and the kernels stay behind the feature as experimental |
| `Binned` is worth a tier | `Binned<3>` bulk vs exact `Wide` bulk | < 2× faster → `Binned` remains a convenience, not a recommended default |
| bit-identical across architectures | the golden hash | any mismatch blocks a release |
| the laws hold | the laws kit on all three exact tiers, with the `f64` control failing | the control passing means the kit is blind |

## Documentation {#sp-docs}

The README carries the contract, the trilemma, the tiers table and, as its centrepiece, the argument
that the adaptive accumulator is not a group. Nobody else on crates.io says that. Every public item
has rustdoc: `missing_docs` warns in the crate and the gate's `-D warnings` fails on one, since
warnings fail the gate only. `SECURITY.md` names the disclosure address, and `CHANGELOG.md` records
each release.

## Release {#sp-release}

`dev` is where work lands and the default branch; `main` holds releases and accepts only a pull
request from `dev`, merged as a merge commit. The version is counted from 0.0.0 over every commit
since the first, in order, by Conventional Commits' own mapping: a breaking change (`!` after the
type, or a `BREAKING CHANGE:` footer) adds one to the major and zeroes the rest, else a `feat` adds
one to the minor and zeroes the patch, else a `fix` adds one to the patch, and every other type
leaves it alone; `scripts/version-from-log.sh` is that count, and nobody types a version. On every
push to `dev` the release workflow keeps a release PR open against `dev` that writes the count into
Cargo.toml and Cargo.lock and the entry for the commits since the last tag into CHANGELOG.md, grouped
by type by `scripts/changelog-entry.sh`. Merging it fixes the version, and merging `dev` into `main`
cuts the release: the workflow refuses a Cargo.toml that disagrees with the count, tags `vX.Y.Z` and
publishes the GitHub Release with the entry as its notes, builds the `.crate` from that commit with
`cargo package --locked`, attests its provenance with GitHub artifact attestations (Sigstore, bound
to this repository, the workflow and the commit) and attaches the `.crate`, its sha256 and the
attestation, which a reader checks with `gh attestation verify`. The build runs on git, cargo and gh
and the two GitHub-owned actions alone, because the crate's posture of no dependency it does not need
is the build's too. A ruleset keeps `v*` tags from being moved or deleted. Both merges are the
owner's acts; so is any announcement. crates.io is outside the release; the owner says when it joins.

## Non-goals

- **Adaptive binning in the incremental calls.** It ships as `Binned`, which the type system keeps out
  of the engines' weight traits.
- **`f32` as its own lane.** Every `f32` is exactly an `f64`, and `to_f32` single-rounds.
- **Order.** A float as a group key or join key, `MIN`/`MAX`, NaN in a range predicate: no rounding
  happens in any of them, so none needs a group.
- **Arbitrary float arithmetic.** Division, `exp`/`log` and non-dyadic constants stay outside the
  group.
- **GPU kernels.**

## Open questions

- **The `Exact` limb cap.** The design takes a named panic at `EXACT_MAX_LIMBS`, because
  `plus_equals` can't return an error. The alternative refuses at `deposit_product` and makes
  `Multiply` saturate to a sticky NaN amount: that is total, but non-linear.

## References

- P. Ahrens, H. D. Nguyen and J. Demmel, UC Berkeley technical report UCB/EECS-2016-121
  (ReproBLAS, the Demmel–Nguyen indexed accumulator).
- Kulisch's exact dot product; Shewchuk's adaptive-precision expansions; ExBLAS.
- bitrep 0.5.1: `FORMAT.md` and `conformance/vectors.json`.
- differential-dataflow 0.24.0 on timely 0.30.0; dbsp 0.364.0 and Feldera's `udf.md`.
