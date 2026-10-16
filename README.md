# float-abel

Exact abelian-group sums of `f64`, so an incremental engine can add, retract and re-merge floats and
land the same bits under any order, worker count or architecture.

The crate is under construction and not yet published.

## The contract

An incremental engine keeps a running answer and applies changes to it: an insert adds a weight, a
delete adds its negation, and a background merge may add any subset in any order. That only works if
the weight type is an exact abelian group, and `f64` addition is not one: it is not associative, so
the same multiset summed in two orders lands two answers, and `a + b − b` need not be `a`. Today an
engine that wants float weights scales to fixed-point integers and picks a range in advance, or
re-folds the whole group on every change, or compares its outputs at a tolerance. Feldera, for one,
refuses to make `SUM` over `DOUBLE` linear and documents why.

`float-abel` holds sums, and products, of IEEE doubles exactly, as a canonical abelian group with a
ring on top. No floating-point instruction executes in any group operation or read: only `to_bits`,
shifts and adds with carry. So:

- the same multiset lands the same bits under any order, worker count, merge tree, consolidation
  timing or architecture; a CI job asserts one SHA-256 over every tier's bytes and reads on seven
  targets, s390x's big-endian among them;
- a retraction cancels its insertion exactly, to the one canonical zero, so an engine's `is_zero`
  deletes the key and nothing is left behind;
- structural `Eq`, `Ord` and `Hash` are numeric, because every operation canonicalises, which is what
  an engine's consolidation and deduplication rely on;
- a read is the correctly rounded double of the exact sum, so every engine divides the same exact
  numerator once for an `AVG`.

The honest boundary: the group makes **sums and products** exact. Division, normalisation,
`exp`/`log` and a non-dyadic constant such as ⅓ are not group operations and stay on the read side.
Nothing here makes arbitrary floating-point arithmetic reproducible. It makes the part an
incremental engine *accumulates* reproducible, which is the part that breaks.

```rust
use float_abel::Exact;

// f64 adds these to 0.6000000000000001; the exact sum, read once, rounds to 0.6.
let mut total = Exact::ZERO;
for x in [0.1, 0.2, 0.3] {
    total += Exact::from(x);
}
assert_eq!(total.to_f64(), 0.6);

// A retraction is the negation, and it cancels bit for bit: no residue, no history.
total -= Exact::from(0.2);
total -= Exact::from(0.1);
assert_eq!(total, Exact::from(0.3));
total -= Exact::from(0.3);
assert!(total.is_zero());

// f64 loses the 1.0 between the two large terms; the group does not.
let swamped: Exact = [1e16, 1.0, -1e16].into_iter().map(Exact::from).sum();
assert_eq!(swamped.to_f64(), 1.0);
```

## Why the usual answer is not a group

The reproducible-summation literature's answer to order-dependence is the adaptive binned
accumulator (ReproBLAS, Demmel and Nguyen): keep K bins whose exponents are pinned relative to an
**index** chosen from the largest magnitude seen so far. When a larger value arrives, the index rises
and the lowest bin is dropped. Bits below it are truncated at the same boundary for every input,
past and future, which is exactly why a one-shot reduction is order-independent. Addition on
`(index, bins)` is associative and commutative.

But it is **not a group**. Its zero is not unique: `a + (−a)` leaves `(index_a, 0)`, and adding that
to `(J, b)` gives `(max(index_a, J), trunc(b))`, which is not `(J, b)` when `index_a > J`. There is no
inverse in the group sense, and the consequence is concrete:

- Insert `1e30`, insert `1e-10`, retract `1e30`. The state is `1e-10` truncated to the precision the
  `1e30` index allowed. At K=3, W=40 that is about `8e5` of granularity, so the surviving `1e-10`
  reads as **zero**. A batch system that never saw the `1e30` reads `1e-10`. The incremental answer
  is a function of history, not of the current input.
- In differential-dataflow the failure is sharper. If `is_zero((index, 0))` is `true`, consolidation
  *deletes* it, and the other diffs under the key stop being truncated. The answer now depends on
  *when* consolidation happened to run.

The fixed-anchor variant of the same idea, bins pinned to absolute exponents with no index, is a
two's-complement fixed-point integer with a declared window. That is a group, and it is `Fixed`
below. Exact accumulation is trivially a group, and it is `Wide` and `Exact`. The adaptive form ships
too, as `Binned`, for batch work only, and the type system keeps it out of the engines' weight
traits: a compile error, not a convention.

## The trilemma

Constant size, full `f64` dynamic range, exact abelian group: **pick two.** A constant-size exact
group over the full range does not exist, by pigeonhole. Every honest design sits at one corner, and
the crate ships each:

| corner | type | size | domain | ring |
|---|---|---|---|---|
| exact + full range | `Exact` | 32 B, plus one allocation past a 127-bit span | every `f64`, specials counted; every sum and product up to the limb cap | `× Exact` with dyadic special amounts, `× i64`, `× isize` |
| exact + constant size (wide) | `Wide` | 296 B | every `f64` | `× i64`, `× isize` |
| exact + constant size (window) | `Fixed<LSB, LIMBS>` | 8·LIMBS B (`Fixed256` is 32) | finite doubles exactly on the 2^LSB grid inside the window; the rest refused at the door, or quantised by declaration | `× i64`, `× isize` |
| constant size + full range: **lossy, batch-only** | `Binned<K>` | 16·K + 32 B (80 at K=3) | every `f64` | none, and **no group**: `Add` only, no `Neg`, no engine weight traits |

`Wide`, `Fixed` and `Binned` are `Copy` plain words with no pointer in them, and never allocate.
`Exact` is a `repr(C)` 32-byte word in two forms: the compact form holds a finite part of at most
127 bits with the special counts as `i32`, and a result past that promotes to the heap form, the
same word over one allocation of limbs, which every operation demotes again as soon as its result
fits, so one value has one form and `is_compact()` reports it. A same-scale column never promotes;
a sum of `1e300` and `1e-300`, or a fold of products, does. The caps are named: past the limb cap
(`EXACT_MAX_LIMBS`, sixteen products' worth of bits) or the exponent bound, `Exact` panics with a
named invariant, because an engine's `plus_equals` cannot return an error, and the `checked_*`
methods return the cap as `PastCap` instead with the state as it was. Without the `alloc` feature
`Exact` is the compact form alone and the width is its cap.

`Exact` is the engine weight. `Wide` has the fastest add of any tier and costs bytes, which suits a
per-key aggregate slot rather than a per-update diff. `Fixed` is for data whose scale the caller
declares: money, counts, measurements and probabilities above 10⁻¹³ are exact on `Fixed256`'s grid.
`Binned` folds a tile and converts to a `Wide` in O(K) rather than serving as a weight.

### NaN and ±∞

A group cannot hold a sticky NaN, since nothing cancels it. It can hold **counts**. `Exact` and
`Wide` carry the finite sum plus three signed counts of NaN, +∞ and −∞ inputs, so a retraction
subtracts, and project once at read time: any NaN reads NaN, both infinities read NaN, one infinity
reads as its sign, otherwise the finite sum correctly rounded. A column containing NaN sums to NaN, as
SQL engines answer, and retracting the NaN row restores the finite sum exactly. `Fixed` refuses a
non-finite input outright, because its contract is exact or refused.

## In an engine

Behind the `differential` feature, `Exact` implements differential-dataflow 0.25's weight traits
(`Semigroup`, `Monoid`, `Abelian`, and `Multiply` by `Exact`, `i64` or `isize` on either side), so a
collection takes it as a diff the way it takes `isize`. The idiom: `explode` the float column into
the diff, then `count_total` reads each key's exact total back out. A delete is the row with
multiplicity `-1`, which `Multiply<isize>` turns into `−1 × Exact`.

```rust
use differential_dataflow::input::Input;
use differential_dataflow::operators::CountTotal;
use float_abel::Exact;

timely::execute_directly(|worker| {
    // A row is (account, amount as bits): a collection's data must be Ord, and f64 is not.
    let (mut rows, probe) = worker.dataflow::<u64, _, _>(|scope| {
        let (input, rows) = scope.new_collection::<(u32, u64), isize>();
        let (probe, _totals) = rows
            .explode(|(account, bits)| Some((account, Exact::from(f64::from_bits(bits)))))
            .count_total()
            .inspect(|((account, total), epoch, change)| {
                println!("epoch {epoch}: account {account} total {} ({change:+})", total.to_f64());
            })
            .probe();
        (input, probe)
    });
    for amount in [0.1, 0.2, 0.3] {
        rows.insert((1, f64::to_bits(amount)));
    }
    rows.advance_to(1);
    rows.flush();
    worker.step_while(|| probe.less_than(rows.time()));

    // Retract one row: the total is re-read as 0.4, not re-summed from the other two.
    rows.remove((1, f64::to_bits(0.2)));
    rows.advance_to(2);
    rows.flush();
    worker.step_while(|| probe.less_than(rows.time()));
});
```

`examples/dd_sum.rs` is the same idiom on three workers with a retraction epoch that nets to
nothing. The multi-worker integration test covers `join`, `antijoin` with `isize`, `iterate`,
`reduce`, `threshold`, `distinct` and `semijoin` with `Exact` in the diff.

Behind the `dbsp` feature, `Exact` is a DBSP 0.364 `DBWeight`: a weighted batch takes it, and
`aggregate_linear` takes it as the accumulator. That is the shape of Feldera's `CREATE LINEAR
AGGREGATE`, the sanctioned route to an exact `SUM` over doubles, since the SQL compiler refuses to
make `SUM(DOUBLE)` linear by design. `examples/dbsp_uda/` holds that aggregate's three Rust items
beside its SQL. The feature builds on the toolchain its CI job pins, rustc 1.99.0, because 1.97 and
1.98 ICE inside dbsp.

`Binned` implements none of either engine's weight traits, and a compile-fail test pins that.

## Checking your own weight

Behind the `laws` feature, `float_abel::laws` checks with proptest what an incremental engine asks
of a weight, against any type that implements `laws::Weight`, yours or ours: associativity and
commutativity bit for bit under a thousand shuffles of a fold; identity; exact inverse; canonicality
(structural `Eq` is numeric equality, `Hash` consistent); distributivity of the ring; and
consolidation-timing invariance, the same multiset summed in random partitions with `is_zero`
pruning between them. A check returns proptest's minimal failing case under a reason that names every
law it broke, so a failure is a value, not a panic.

```rust
use float_abel::laws::{self, check_abelian_group, proptest::prelude::*};
use float_abel::Exact;

// The strategy draws the weight's domain: here same-scale doubles, which stay in the compact
// form; a multiset spread over the whole exponent range runs through the heap form and passes
// too, at the cost of an allocation per promoted sum.
let mut runner = laws::runner(64, 0);
check_abelian_group(&mut runner, (-1.0e6..1.0e6).prop_map(Exact::from)).unwrap();
```

The kit runs against every tier in CI, and against a naive `f64` wrapper as a negative control that
must **fail** associativity and inverse; a control that passes means the kit is blind.

## Bulk sums

`Wide`, `Fixed` and `Binned` take a whole column at once through `extend_from_slice(&[f64])` and
`deposit_products(&[f64], &[f64])`, the dot-product shape. Behind the `simd` feature those run on
runtime-dispatched NEON and AVX2 kernels: lane-parallel partial accumulators, legal because every tier
is associative and commutative, merged at the tile's end to the same bits as the scalar reference,
which the golden hash asserts on every target. The kernels are the only `unsafe` in the crate, and
with the feature off the crate is `unsafe`-free end to end. Products are not vectorised.

## What reproducibility costs

The price of the guarantee, measured against the `f64` arithmetic it replaces, on the same-scale
band, at commit 2a82388 (`BENCH.md` names the machine, the toolchain and the commit for every number):

| shape | `f64` | `Exact` | `Wide` | `Fixed256` | `Binned<3>` |
|---|--:|--:|--:|--:|--:|
| one value at a time into one slot, NEON (Apple M5 Max) | 0.58 ns | 4.94 ns, 8.5× | 18.70 ns, 32× | 7.89 ns, 14× | 10.22 ns, 18× |
| one value at a time into one slot, AVX2 (AMD Ryzen 7 5700U) | 0.73 ns | 15.89 ns, 22× | 36.36 ns, 50× | 16.18 ns, 22× | 18.51 ns, 25× |
| a column summed in bulk, NEON | 0.080 ns | | 1.82 ns, 23× | 1.60 ns, 20× | 2.00 ns, 25× |
| a column summed in bulk, AVX2 | 0.368 ns | | 2.52 ns, 6.9× | 2.02 ns, 5.5× | 1.68 ns, 4.6× |
| a `SUM` in differential-dataflow, 10⁷ updates, 8 workers, NEON | 43.0 ms (`isize`) | 79.7 ms, 1.85× | | | |
| a `SUM` in differential-dataflow, 10⁷ updates, 8 workers, AVX2 | 181.4 ms (`isize`) | 376.3 ms, 2.07× | | | |

Updating an aggregate one row at a time, the engine's shape, costs `Exact` five nanoseconds a row
where a float costs half of one; a consumer's `+=` on the word compiles to the integer ops in place,
every operator down to the compact tier being `#[inline]`. Summing a whole column, the SIMD kernels
bring every tier to about two nanoseconds a value. Inside differential-dataflow, where the diff's
cost is diluted by everything else the engine does per update, `Exact` runs at about twice the
native integer diff. Reading a total out (`to_f64`, 9.8 ns and 17.8 ns) happens once per aggregate,
not once per row.

The other number that matters is the group size past which one change through the engine beats
re-summing the group's floats with no engine at all (`docs/CROSSOVER.md`, `BENCH.md` § The
crossover): about 10⁴ rows a group on NEON and 2×10⁴ on AVX2 at one change an epoch, where the
engine's epoch cost dominates and the word's cost is invisible, and a few hundred rows once changes
are batched by the thousand. Above it the crate wins and the margin grows linearly with the group:
at a million rows one change is 4 µs through the engine against 77 to 499 µs of re-sum on NEON.
Below it the re-sum is faster, and the crate is bought for the same bits under any schedule and for
exact retraction, which the re-sum cannot offer at any size.

## Wire format

Each group type has one canonical little-endian encoding, `FORMAT.md`: a version byte that names the
type (`0x01` `Exact`, `0x02` `Wide`, `0x03` `Fixed`) then the type's `repr(C)` bytes, 33, 297 and
`1 + 8·LIMBS` in all. `to_bytes` and `from_bytes` are on every feature set, and behind `serde` the
impls carry the same bytes: `Exact` and `Wide` as one byte string, so bincode 1 frames the 33 with a
`u64` length and postcard with a varint, and `Fixed` as a tuple of its bytes. A decoder refuses
anything that is not a canonical value, as a `WireError`, never a panic, because a non-canonical
value smuggled through a batch would break `Eq`-substitutability in an engine's builder. `Exact`'s
heap form, behind `alloc`, encodes under its own first byte (`0x04`) as the block the word holds,
through `encode`, and `from_bytes` reads both forms; the decoder bounds the limb counts by the cap
before it allocates and refuses a heap body whose value the compact form holds. `Binned` has no
wire form and crosses a boundary as the `Wide` it converts to. The format freezes at 1.0.

## Features

| feature | pulls in | provides |
|---|---|---|
| `alloc` | — | `Exact`'s heap form: promotion past the width, dyadic special amounts, the limb cap |
| `std` (default) | `alloc` | `std::error::Error` impls, the bench harness |
| `serde` | serde 1 | `Serialize`/`Deserialize` in the canonical encoding, validated on decode |
| `differential` | differential-dataflow 0.25 + `serde` | the differential-dataflow adapter |
| `dbsp` | dbsp 0.364 + rkyv 0.7 + feldera-size-of + `std` | the DBSP adapter |
| `laws` | proptest | the laws kit |
| `simd` | `std` | runtime-dispatched NEON and AVX2 bulk kernels, the `unsafe` beside the heap form's |
| `arbitrary` | arbitrary 1 | fuzz inputs |

The crate is `no_std` with zero required dependencies. MSRV is 1.74 on the default features, asserted
in CI; `differential` inherits differential-dataflow's 1.86, and `dbsp` builds on 1.99.0 and 1.96.1.

## Verification

The laws kit on every tier with the `f64` control failing; correct rounding of `to_f64`, `to_f32`
and `div_f64` against vectors generated from CPython's `Fraction`, ties, subnormals and overflow
included; `Fixed256`, `Wide` and `Exact` agreeing bit for bit on their shared domain, and `Exact`
against bitrep's sum on random multisets; the decoder's refusals; sums against a `num-bigint`
oracle under `cargo-fuzz`, nightly, ten minutes a target; the suite under miri; one golden SHA-256
over every tier's bytes, reads and refusals on x86_64 Linux and Windows, aarch64 macOS and Linux,
wasm32, riscv64gc and s390x; every SIMD kernel equal to the scalar reference, lane count
notwithstanding. The design and its reasoning are in [`docs/DESIGN.md`](docs/DESIGN.md).

## Building

```bash
cargo test                                             # the self dev-dependency turns the test features on
cargo test -p float-abel-noalloc                       # Exact with no features: the compact form alone, the width its cap
cargo +1.99.0 test --features dbsp --test main dbsp_   # dbsp's toolchain; 1.97 and 1.98 ICE inside it
cargo run --example dd_sum -- -w 3                     # Exact as a differential-dataflow diff: SUM on three workers
cargo +nightly miri test --test main                   # the suite under miri (sweeps cut to a hundredth; trybuild and timely ignored)
cargo fuzz run sum_vs_oracle --target-dir "$CARGO_TARGET_DIR/fuzz" -- -max_total_time=60   # a fuzz target; nightly + cargo-fuzz
./scripts/smoke.sh                                     # commit gate
./scripts/regression.sh                                # push gate
git config core.hooksPath .githooks                    # run the gates from git
```

Keep `CARGO_TARGET_DIR` outside any Spotlight-indexed or cloud-synced tree. Every object cargo writes
into a watched directory is queued for indexing or upload, and on macOS that is what makes a warm
build feel cold. For example:

```bash
export CARGO_TARGET_DIR="$HOME/.cache/cargo-target/float-abel"
```

## Security

Report a vulnerability through GitHub's private vulnerability reporting on this repository, as
[`SECURITY.md`](SECURITY.md) says.

## License

MIT OR Apache-2.0, at your option.
