# float-abel

*"Squeezing infinitely many real numbers into a finite number of bits requires an approximate
representation [....] [C]omputing with 13 digits gives an answer correct to 10 digits. By keeping these extra 3 digits hidden, the calculator presents a simple model to the operator."* —David Goldberg [1]

* [how it works](#how-it-works)
* [how to use it](#how-to-use-it)
* [why it exists](#why-it-exists)

## overview

float-abel's a Rust crate for adding up `f64`s exactly. As in, `0.1 + 0.2 - 0.2` actually equals `0.1`. Its data is bit-identical no matter the order values arrive in, how many workers split them, or which CPU adds them. It cleanly backs out values:

```rust
use float_abel::Exact;

// f64 adds 0.1 and 0.2, takes them both back, but doesn't go back to zero:
assert_eq!(0.1 + 0.2 - 0.1 - 0.2, 2.7755575615628914e-17);

// versus with float-abel:
let mut total = Exact::ZERO;
total += Exact::from(0.1);
total += Exact::from(0.2);
total -= Exact::from(0.1);
total -= Exact::from(0.2);
assert!(total.is_zero()); // true zero
```

There's no magic involved and no fundamental computer science breakthrough. The gist is, just use more memory to store numbers at a higher resolution: 256 bits at a minimum. The resolution still has a limit, it's just finer, like the way a HiDPI screen still has pixels if you use a magnifying glass. Some of the core principles were demonstrated in Kyle Clouthier's [bitrep](https://github.com/KyleClouthier/bitrep) project, which served as the inspiration for float-abel's `Wide` type.

Most of the implementation here is to make it use the least memory possible while running as quickly as it can. The memory cost is balanced by what it lets you do: keep an accurate running sum of f64 values [faster than you could if you recomputed the entire vector](#what-it-costs) each time a new value came in.

float-abel's thing, that makes it different than adjacent approaches, is that it's fully retractable and leaves behind no state residue. When you subtract a value in float-abel, it truly goes away. There is no difference in the bits regardless of how you arrived at the value.

float-abel's designed for incremental engines, which do just that: maintain a running total and update it as rows both come _and go_...even if events arrive out of order. Floats have always been a challenge for them, because you can't get consistent results across runs without consistent ordering, and you can't guarantee, as seen above, that removing all the numbers you've added will bring you back to zero.

float-abel comes with adapters to make that possible for differential dataflow [2] and DBSP [3], plus tools to help build more.

### getting it

For now, grab a release tag from here. crates.io will come.

## how it works

Every f64 is an integer multiple of 2⁻¹⁰⁷⁴ [4]. So float-abel never adds floats. It reads each number's bits, places the integer on that one fixed grid, and adds the integers column by column, carrying the 2s. It's just like how you carry the 10s when you're adding decimals up by columns.

Integer addition is associative (you can chunk a list any way you want and it adds up the same) and commutative (you can shuffle a list any way you want and it adds up the same) and every value has an inverse (you can remove any value by adding its opposite), which makes it what's called an "abelian group". That means float-abel can be used in an incremental engine, which usually excludes floating point numbers.

No floating point instruction runs in any operation, which is why the bits match across architectures. (To validate this, CI checks one SHA-256 over every type's bytes on seven targets, big-endian s390x among them.)

Rounding happens once, when you read the total, to the correctly rounded f64 of the exact sum. An `AVG` divides the exact total once:

```rust
use float_abel::Exact;

let xs = [0.1, 0.2, 0.3]; // these sum to 0.6, so divided by 3 should be 0.2
assert_eq!(xs.iter().sum::<f64>() / 3.0, 0.20000000000000004); // but f64's rounding adds error
let total: Exact = xs.into_iter().map(Exact::from).sum(); // while doing the same with float-abel
assert_eq!(total.div_f64(3), 0.2); // ...rounds once, correctly
```

[Every value has exactly one bit pattern](FORMAT.md), so `==`, `Hash` and `is_zero` compare numbers. NaN and ±∞ can't live in a group, since nothing cancels them out, so they're counted instead: a column holding a NaN reads NaN, and retracting the NaN row brings back the sum value exactly.

```rust
use float_abel::Exact;

let mut total: Exact = [1.5, f64::NAN, 2.0].into_iter().map(Exact::from).sum();
assert!(total.to_f64().is_nan()); // one NaN spoils the bunch
total -= Exact::from(f64::NAN); // weed it out
assert_eq!(total.to_f64(), 3.5); // now we're good
```

## how to use it

### pick three

* Small
  * As in, tens of bytes, not hundreds
* Fixed size
* The full range of f64s
  * Every f64 and every sum of f64s
* Exact

You get any three.

You can't have all four qualities at once, because the range gets so wide: an f64 can be as large as 2¹⁰²³ or as small as 2⁻¹⁰⁷⁴, which works out to 2,098 bits. (More, really. You need a few extra bits for headroom.) A type that's exact over the whole range is either that big all the time, like `Wide`, or small unless a sum needs more space, the way `Exact` works. The others stay small and fixed by giving up the range or the exactness, since sometimes you want that.

| type | small | fixed size | full range | exact |
|---|---|---|---|---|
| `Exact` | **yes**: 32 B most of the time | **no**: the size expands when values can't be expressed in 127 bits | effectively **yes**: reaching its caps by adding f64s would take about 2³²²⁷⁰ additions (there are fewer than 2²⁶⁶ atoms in the universe) | **yes** |
| `Wide` | **no**: 296 B | **yes** | effectively **yes**: every f64, and 2⁷⁷ additions of the largest one before its headroom runs out (millions of years, at a nanosecond each) | **yes** |
| `Fixed256` | **yes**: 32 B | **yes** | **no**: only values as small as 2⁻⁴⁴ (about 5.7 × 10⁻¹⁴, in meters far smaller than the width of a hydrogen atom) up to 2⁹⁵ (about 4 × 10²⁸, in meters roughly 45× the width of the observable universe) in size. | **yes** |
| `Binned<3>` | **yes**: 80 B | **yes** | **yes** | **no** |

Multiplication is a different story. It's not possible to handle every scenario. A product of two f64s can land anywhere from 2⁻²¹⁴⁸ to 2²⁰⁴⁸, twice an f64's range, and a product of products keeps growing, so nothing holds all of them.

`Exact` gets closest, because its heap grows to 34,368 bits, about 10,346 digits. That's enough for a chain of about 16 `float × float` products, and it can also handle about a million nested products of the smallest expressible f64. This should be enough for many use-cases.

`Wide` and `Fixed256` refuse products they can't express. `Binned` refuses products over `Wide`'s ceiling, and then truncates as designed.

**`Exact`** is the default because it's probably what you want. It's what the differential dataflow and DBSP adapters use for incrementing sums, and what an incremental `join` query needs to support multiplication. When small, it holds about 38 significant digits against an f64's 16, so ten million values anywhere between 10⁻⁶ (in meters, the length of a bacterium) and 10⁶ (in meters, the distance from NY to Chicago) can be added up using it. If you need more range, that's the only time it moves to the heap, and as soon as it can it drops back down to the small form:

```rust
use float_abel::Exact;

let a = Exact::from(1e11) + Exact::from(1e-11);
assert!(a.is_compact()); // only spans 126 bits of resolution so it stays small
let b = Exact::from(1e12) + Exact::from(1e-12);
assert!(!b.is_compact()); // spans 132 bits so it allocates
assert!((b - Exact::from(1e12)).is_compact()); // and then shrinks back off the heap automagically
```

The price of that pointer is that `Exact` is `Clone`, not `Copy`.

**`Wide`** is for situations where you can't have a pointer. Some engines keep each key's aggregate in a slot that must be `Copy`, or write it to disk and read it back by casting the bytes, so pointers are verboten. It's just a plain array of thirty-four 64-bit values forming a 2,176-bit fixed-point number. That gives it about 655 digits, reaching every place a sum of f64s can. It also has three counters for NaN and ±infinities. Every bit pattern is a valid value, it never allocates, and it never refuses an f64. It gives up being small instead.

**`Fixed256`** is `Wide` for when you know your input in advance. It's the same kind of plain `Copy` word in 32 bytes, about a ninth the size of `Wide`. Its range holds what most data is made of exactly: money, counts, measurements, probabilities above 10⁻¹³, etc. The catch is the refusal. A value outside the window is refused with an `OutOfWindow` error, so pick `Fixed256` only when you know every value that will ever come in fits, or declare a floor and round onto the grid first with `Fixed256::quantize`. `Fixed<LSB, LIMBS>` builds other grids and widths as needed.

```rust
use float_abel::{Fixed256, OutOfWindow};

assert!(Fixed256::try_from(0.1).is_ok()); // 0.1 is on the grid, it's held exactly
assert_eq!(Fixed256::try_from(1e-20), Err(OutOfWindow::OffGrid)); // too fine: refused, not rounded
assert!(Fixed256::try_from(Fixed256::quantize(1e-20)).is_ok()); // unless you round it onto the grid
```

**`Binned<3>`** is for one-shot batch sums, not incremental usage. It's [Demmel and Nguyen's binned summation](#why-binned-summation-isnt-enough): 80 bytes, the same bits under any order or split, and off by at most n × 2⁻¹²⁸ times the largest value ever seen.

### in an engine

#### differential dataflow

Differential dataflow stores a weight for every item in a collection. Normally the weight is an `isize` integer count: inserting a row adds `+1`, deleting it adds `-1`, and a row whose weights add up to zero goes away. The engine doesn't care what the weights mean. All it does is add them, in whatever order and on whichever worker, and it needs them to cancel out exactly when they should. Easy with integers. Not so easy with floats. Because `f64` isn't an abelian group, and integer `isize` is.

float-abel's `Exact` floating point type is an abelian group too. Behind the `differential` feature `Exact` can stand in anywhere an `isize` weight does, and the engine's own bookkeeping can do the adding:

```rust
use differential_dataflow::input::Input;
use differential_dataflow::operators::CountTotal;
use float_abel::Exact;

timely::execute_directly(|worker| {
    // declare the pipeline
    let (mut rows, probe) = worker.dataflow::<u64, _, _>(|scope| {
        // batch of rows of (u32 account ids, u64 float amount as opaque bits),
        // each with the standard isize weight: +1 in, -1 out
        let (input, rows) = scope.new_collection::<(u32, u64), isize>();
        let (probe, _totals) = rows
            // parse each amount out of its row and into the weight
            .explode(|(account, bits)| Some((account, Exact::from(f64::from_bits(bits)))))
            // then add up each account's weights
            .count_total()
            // and print output we'll see below
            .inspect(|((account, total), epoch, change)| {
                println!("epoch {epoch}: account {account} total {} ({change:+})", total.to_f64());
            })
            .probe();
        (input, probe)
    });

    // first epoch, add float data
    for amount in [0.1, 0.2, 0.3] {
        rows.insert((1, f64::to_bits(amount)));
    }

    // close the epoch and wait for it to be processed
    rows.advance_to(1);
    rows.flush();
    worker.step_while(|| probe.less_than(rows.time()));

    // second epoch, delete the 0.2
    // the engine subtracts it from the total rather than re-adding the other two.
    rows.remove((1, f64::to_bits(0.2)));

    // close and process the second epoch
    rows.advance_to(2);
    rows.flush();
    worker.step_while(|| probe.less_than(rows.time()));
});
```

`explode` takes each row's amount out of the row and makes it the weight, so the tuple row `(1, 0.2)` with weight `+1` becomes the key `1` with weight `0.2`. Then `count_total` does its thing, counting each key's rows by adding up their weights. When every weight is 1, adding them up gives you a count. When every weight is an amount, it gives you a sum.

The amounts go in as their bits, a `u64`, because the engine sorts rows and Rust won't sort an `f64` (NaN isn't less than, greater than, or equal to anything, not even itself). Here `explode` turns the bits back into an f64.

Deleting a row sends it in again with weight `-1`, and `explode` multiplies that through, so the 0.2 arrives as `-0.2` and comes straight back out of account 1's total. Nothing gets summed over again.

Here's the output:

```text
epoch 0: account 1 total 0.6 (+1)
epoch 1: account 1 total 0.6 (-1)
epoch 1: account 1 total 0.4 (+1)
```

So at epoch 1 the engine withdraws the 0.6 and puts 0.4 in its place.

There's a more elaborate `examples/dd_sum.rs` that does this with two accounts, and spreads the rows across workers by account, to demonstrate how this is order-independent. Run it on one worker, then on four, and the results are the same:

```bash
$ cargo run --example dd_sum -- -w 1 | sort # then again with -w 4
epoch 0: account 1 total 0.6 (+1)
epoch 0: account 2 total 1 (+1)
epoch 1: account 1 total 0.6 (-1)
epoch 1: account 2 total 1 (-1)
```

Both runs print exactly that output. With float-abel, epoch 1 retracts every row, both totals cancel to exactly zero, and the engine drops both accounts. There's no crumb left over to keep a dead account alive.

Contrast with what you'd see with naive f64 usage. Account 1 is `0.1 + 0.2 + 0.3`, which `f64` adds to `0.6000000000000001`. Meanwhile account 2 is `1e16 + 1.0 - 1e16`, which `f64`'s logic thinks add to `0.0`. (That's because `1e16` is past 2⁵³, where [the grid's resolution gap](#why-floats-dont-add-up) is `2.0`, so the `1.0` rounds away.) If you tried to use f64 like this, differential dataflow would wrongly prune account 2 based on that false `0.0`.

#### dbsp

The SQL compiler for DBSP, the engine under Feldera, won't make `SUM(DOUBLE)` linear, but it will let you declare an aggregate of your own as linear and write its accumulator in Rust [9]:

```sql
CREATE LINEAR AGGREGATE exact_sum(value DOUBLE) RETURNS DOUBLE;

CREATE TABLE ledger(account INT NOT NULL, amount DOUBLE NOT NULL);

CREATE MATERIALIZED VIEW totals AS
SELECT account, exact_sum(amount) AS total
FROM ledger
GROUP BY account;
```

Behind the `dbsp` feature, `Exact` is a DBSP weight, so the Rust side is three short items: the accumulator type, which is `Exact`; a function that turns one row's f64 into an `Exact`; and one that rounds the finished total back to an f64, once. `examples/dbsp_uda/` has all three beside that SQL, and a `main` that runs them through DBSP's `aggregate_linear` on two workers without Feldera, printing the same totals as `dd_sum`.

`Wide` and `Fixed` implement both engines' weight traits too, for engines whose weight slot has to be `Copy`.

### what it costs

Exactness isn't cheap. float-abel takes 15–30× more time than an imprecise hardware float add per row. If you don't care about exactness, just use f64 like a normal person.

If you're a sicko who needs this stuff, then the real comparison isn't adding single values. It's how float-abel compares against the way you'd get consistent results from f64: adding up the entire list whenever any value gets added.

float-abel's always faster than recompute once you're adding up streams of more than 120k items, and often pays off before then. Even float-abel's order of magnitude decrease in speed is dwarfed by how long a full recompute takes once collections grow that large. With float-abel, you just need to add the new items that come in.

| | M5 Max | Ryzen 7 |
|---|--:|--:|
| one add into a running total, `f64` | 0.57 ns | 0.74 ns |
| one add into a running total, `Exact` | 8.6 ns | 21.3 ns |
| a column summed in bulk with SIMD, `Wide` | 1.8 ns a value | 2.6 ns a value |
| a `SUM` over 10⁷ updates in differential dataflow on 8 workers, `Exact` against an `isize` weight | 2.2–2.3× | 1.8–2.0× |

*[Measured](BENCH.md) on an Apple M5 Max (NEON) and an AMD Ryzen 7 5700U (AVX2), at commit f24b051.*

At one change per epoch, float-abel starts to make sense after about 8,400 rows against a plain collection sum, and after 53,000 rows against a vectorized one on the M5. On the Ryzen the numbers are 22,000 and 115,000. If you batch a thousand changes per epoch, the tipover point drops to between 119 and 1,110 rows. The gap grows with collection size. At a million rows, on the M5 one change costs 4 µs incrementally against 77–489 µs to re-sum.

### how it's checked

* The [laws kit](#check-your-own-weight) runs on every type, with the `f64` control required to fail.
* `to_f64`, `to_f32` and `div_f64` are tested against correctly rounded vectors generated with CPython's `Fraction`, ties and subnormals included.
* `Fixed256`, `Wide` and `Exact` are validated to agree bit for bit where their domains overlap, and `Wide` is validated against bitrep as a reference implementation. Sums are checked against a `num-bigint` oracle under `cargo-fuzz`.
* The suite is tested under miri.
* Output is hash-validated to be bit-identical on x86_64 Linux and Windows, aarch64 macOS and Linux, wasm32, riscv64gc and s390x.
* Every SIMD kernel is validated against scalar loops.

```bash
cargo test                                             # the test features turn on by themselves
cargo +1.99.0 test --features dbsp --test main dbsp_   # dbsp's toolchain
cargo run --example dd_sum -- -w 3                     # SUM on three workers
```

### check your own weight

Maybe you're writing a weight type of your own, or bringing float-abel to a new engine. Either way, you'll want to know the weight really is an abelian group before an engine relies on it.

Behind the `laws` feature, the property tests float-abel runs on itself will run on any type that implements `laws::Weight`: `Clone`, `Eq`, `Hash`, `Debug`, `+`, `-` and negation, plus `zero()` and `is_zero()`. A property test doesn't merely check a handful of examples you picked. You give it a strategy, a recipe for random values, and it throws case after case of them at each law looking for one that breaks it:

```rust
use float_abel::laws::{self, check_abelian_group, proptest::prelude::*};
use float_abel::Exact;

let mut runner = laws::runner(64, 0); // 64 random cases from seed 0, so every run is the same run
let doubles = (-1.0e6..1.0e6).prop_map(Exact::from); // random f64s between ±1,000,000, as Exacts
check_abelian_group(&mut runner, doubles).unwrap(); // and every law holds
```

There are five checks, one for each kind of promise an engine relies on:

| check | the law | what the engine does that needs it |
|---|---|---|
| `check_abelian_group` | the same bits from 1,000 shuffles of one sum, adding zero changes nothing, and `a + b - b == a` | adds weights in any order and retracts by subtracting |
| `check_canonical` | `==` and `Hash` mean numeric equality: one number, one bit pattern | matches weights by `==` and `Hash`, and deletes the ones that are zero |
| `check_ring` | `(a + b) × k == a × k + b × k` | multiplies weights, by a row's count or by each other in a `join`, before or after adding them up |
| `check_consolidation_invariance` | the same values merged in random batches, zeros thrown out between merges, land the same bits | consolidates whenever it likes |
| `check_closure` | every sum, difference and product comes back instead of panicking | has no way to take an error back from a weight, so one that hits a cap can only panic |

When a check fails, proptest doesn't just hand you the random mess it tripped on. It distills it, trying smaller and simpler values until it finds the smallest case that still fails. The proptest suite also runs on plain `f64` in a `Naive` wrapper, with values drawn from thousandths between −1 and 1, and requires it to fail. proptest whittles it down to:

```text
broke
  inverse: Naive(0.001) + Naive(0.898) − Naive(0.898): Naive(0.0010000000000000009) is not Naive(0.001)
```

Essentially `0.1 + 1.0 - 1.0` from [why floats don't add up](#why-floats-dont-add-up), arrived at empirically.

These tools exist to make float-abel as open as possible. You can adapt float-abel to your own engine without needing to fork or contribute to the project. If you're carrying float-abel's value types to another engine, or to another language, you don't even need this crate's code to read them. Every value has exactly one byte encoding, specified in [`FORMAT.md`](FORMAT.md).

### limits

- **Sums and some products, nothing else.** Division, `exp`, `log` and constants like ⅓ aren't group operations, so they happen once, on the exact total, at read time. float-abel makes what an engine accumulates reproducible. It doesn't make arbitrary float code reproducible and [it can't handle an arbitrarily long chain of multiplications](#pick-three).
- **`Exact` has two named caps.** 34,368 bits of span, and an exponent bound of 2³⁰. Sums of f64s never get near either. Only long chains of nested `float × float` products do. Past a cap, `+=` panics with the cap's name, because an engine's `plus_equals` can't return an error, and the `checked_*` methods return `PastCap` instead. If you run more than one worker and can reach a cap, build with `panic = "abort"`, or one worker's panic leaves the rest spinning.
- **Ordering is out of scope.** `MIN`, `MAX` and float keys don't round, so they don't need a group.

If you're dealing with just a few hundred items, you have all your data up front, you don't need clean retraction, or you're only dealing with currency, options like bitrep, ReproBLAS, or fixed-width decimals may suit your needs better.

Those solutions can also be less complex, making them easier to reason about and maintain as dependencies. For example, because bitrep's guarantee is only that floating-point statistics are exact, it can achieve its goal by keeping track of what's been added and removed over time as stored state, in a count that never goes down. The numbers that come out are exact, but the variable storing them holds different bits based on what you've done with it. That prevents true equality between otherwise-exact variables, if they were arrived at in different ways. Because float-abel needs bit-exactness for incremental engines, which doesn't allow state history like that, it has to do more.

### features

Theoretically you can use float-abel with no dependencies at all, but you probably want some minor ones. For example, without `alloc`, which is enabled by default, the `Exact` type can't grow to fit all values, and the SIMD kernels are worth `std` when possible, but `no_std` is supported when needed. They're all optional behind features:

| feature | pulls in | provides |
|---|---|---|
| `std` (default) | `alloc` | `std::error::Error` impls |
| `alloc` | | `Exact`'s heap form; without it the 127-bit word is the cap |
| `serde` | serde 1 | the canonical encoding, validated on decode |
| `differential` | differential-dataflow 0.25, timely 0.31, `serde` | the differential dataflow adapter and `banded_sum` |
| `dbsp` | dbsp 0.367, rkyv 0.7, feldera-size-of, `std` | the DBSP adapter |
| `laws` | proptest, `std` | the laws kit |
| `simd` | `std` | NEON and AVX2 bulk kernels for `Wide`, `Fixed` and `Binned` |
| `arbitrary` | arbitrary 1, `std` | fuzz inputs |

MSRV is 1.74 on the default features. `differential` needs differential-dataflow's 1.86, and `dbsp` builds on 1.99.0.

## why it exists

### why floats don't add up

You'd think `a + b - b == a`. Try `0.1 + 1.0 - 1.0`:

```rust
assert_eq!(0.1 + 1.0 - 1.0, 0.10000000000000009);
```

A 64-bit float, or f64, has 53 significant bits plus an exponent that says where the binary point goes [4]. Yeah, binary point. Not decimal point. Remember, floats have to do this with raw bits in binary numbers.

The bits stay put and the point floats. That's easiest to see on a toy float that keeps just three significant bits, giving it 4 possible values, because there's a rule that the first digit has to always be 1, so: `100`, `101`, `110`, and `111`.

Those values express different numbers depending on the exponent that gets applied. To store a bigger number, you apply a bigger exponent. But as the exponent increases, the resolution decreases, because it has to cover a wider range of numbers in the same bits.

Between 1 and 2 it can write four numbers, all using the same exponent:

```text
  binary    1.00   1.01   1.10   1.11
  decimal   1      1.25   1.5    1.75     from 1 to 2, 0.25 apart
```

Let's break this down a bit more. How does `100` become `1.00`? The exponent comes out to zero, because it's calculated against a pre-determined bias for this toy float type of `-011` (-3). That bias shifts the range on the number line, centering exponents evenly between negative and positive powers, so it's equally useful for small and large values. It gets added to the exponent provided in the float value, which in this case is `011`, or (3). -3 + 3 = 0. Because we're using binary, the point shifts with powers of 2. 2 to the power of 0, like any number to 0, gives you 1. So the point is located 1 place in. Seriously, all float numbers are this convoluted.

Now, going from `101` to `1.25` is a little more tricky. It's the same exponent for all numbers in this range, so that's why it also becomes `1.01` instead of, say, `101.0` or `10.1`. Then it's just converting the binary fraction to a decimal one. Instead of going down by tenths (1/10th, 1/100th, 1/1000th), they go down by halves (1/2th, 1/4th, 1/8th). So the `.01` is 'zero halves and 1 quarter', decimal `.25`.

Slide the point one place right by changing the exponent from `011` to `100`, and instead of applying `-3 + 3` we're applying an exponent of `-3 + 4`, or `1`. 2 to the power of 1 is 2, so the binary point is after the second digit. `1.00` becomes `10.0`, etc. This means the same four bit patterns can now express the numbers between 2 and 4:

```text
  binary    10.0   10.1   11.0   11.1
  decimal   2      2.5    3      3.5      from 2 to 4, 0.5 apart
```

Slide it again with exponent `101`, we're shifting the binary point from 2 to 3. So now `10.0` becomes `100.` and the same bits cover 4 to 8:

```text
  binary    100.   101.   110.   111.
  decimal   4      5      6      7        from 4 to 8, 1 apart
```

Let's look at this all together. So far, we've been looking at it row by row, left to right.

```text
  binary    1.00   1.01   1.10   1.11
  decimal   1      1.25   1.5    1.75     from 1 to 2, 0.25 apart

  binary    10.0   10.1   11.0   11.1
  decimal   2      2.5    3      3.5      from 2 to 4, 0.5 apart

  binary    100.   101.   110.   111.
  decimal   4      5      6      7        from 4 to 8, 1 apart
```

Now read down any column top to bottom. The bits never change, only the point moves. Every row gets exactly four numbers, but each row covers twice the ground of the one above it, so its numbers sit twice as far apart. Put them all on one number line and you get a grid that coarsens as it moves away from zero, leaving larger and larger gaps of numbers that can't be expressed:

```text
  value  1       2       3       4       5       6       7       8
         ├─┬─┬─┬─┼───┬───┬───┬───┼───────┬───────┬───────┬───────┤
  gap       0.25        0.5                      1
```

If you slid it to the right again the gap'd be 2: between 8 and 16 it can express the numbers 8, 10, 12 and 14, with no way to write 9. If you slid it back left instead the gap would keep halving each time, getting finer and finer so that it's finest near zero.

An f64 works the same way, with 53 bits instead of 3 [4]. Every stretch between two powers of two holds 2⁵² evenly spaced f64s, so the gap between neighbors is the stretch's width divided by 2⁵²:

| near | stretch | gap |
|---|---|---|
| 0.1 | 2⁻⁴ to 2⁻³ | 2⁻⁵⁶ ≈ 1.4 × 10⁻¹⁷ |
| 1.0 | 2⁰ to 2¹ | 2⁻⁵² ≈ 2.2 × 10⁻¹⁶ |
| 2⁵³ ≈ 9.0 × 10¹⁵ | 2⁵³ to 2⁵⁴ | 2 |

So near 0.1 the grid is 16 times finer than near 1.0. At 2⁵³ the gap reaches 2 (it can express much larger numbers than the toy, which reaches this point at 8). Skipping every other number means odd integers stop existing altogether:

```rust
let big = 9007199254740992.0; // 2⁵³, the value when it starts skipping every other number
assert_eq!(big + 1.0, big); // So that means the next number up isn't on the grid, it just rounds back down
assert_eq!(big + 2.0, 9007199254740994.0); // ...but if you add 2.0, voilà! it can express that
```

And that's really the explanation for all of this:

Every floating point operation computes the _exact_ answer and then _rounds_ it to the nearest point on the grid [1, 4]! That's where `0.1 + 1.0 - 1.0` goes wrong. Write both numbers out in binary with their points lined up:

```text
         ┌────── 1.0's 53 bits ───────┐
  1.0  = 1.0000 0000 0000 ... 0000 0000
  0.1  = 0.0001 1001 1001 ... 1001 1001 1010
              └────── 0.1's 53 bits ───────┘
```

Each gets 53 bits. 1.0's bits start at the ones place. 0.1 is a smaller number, and so its bits start four places after the binary point, then run four places past the end of 1.0's bits. Being smaller lets it 'float' the point to be more precise, because it doesn't have to store the leading zeroes as bits. A larger number like 1.0 can't be so precise, because it's got non-zero digits to track as bits before the binary point. This is how that actually shakes out. Large numbers run out of space in the bitfield for really small numbers.

The sum is between 1 and 2, so it has to land on 1.0's grid, and that grid has no room for 0.1's last four bits. Those bits are `1010`, binary for 10, so the exact sum sits ten sixteenths of the way across one step of 1.0's grid:

```text
               1.0999999999999999                            1.1
  1.0's grid   ├───────────────────────────────────────────────┤
  0.1's grid   ├──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┤
               └───────────── 10 ────────────┴─────── 6 ───────┘
                                             ↑ 1.0 + 0.1, exactly
```

It's ten of 0.1's grid steps above the number below and six short of the one above, so it rounds up to the closer grid point 1.1, six steps higher than the truth. (Yes, when it rounds down it's pretty much the scheme from Office Space and Superman III.) Subtracting 1.0 is exact. It just can't bring back what the addition threw out, so the precision error remains at the end:

```rust
// 0.10000000000000009 is 0.1 plus six of 0.1's steps, 2⁻⁵⁶ each.
assert_eq!(0.1 + 1.0 - 1.0, 0.1 + 6.0 * 2f64.powi(-56));
```

That one rounding step is why order matters. It's why float addition is commutative, but not associative:

```rust
assert_eq!((0.1 + 0.2) + 0.3, 0.6000000000000001);
assert_eq!(0.1 + (0.2 + 0.3), 0.6);
```

This matters any time you divide and conquer by chunking work across multiple workers. Split a sum across workers and you've changed its grouping, so a parallel reduction's bits depend on how the work was scheduled [5, 6].

### why that breaks incremental engines

Incremental view maintenance keeps a result current by applying each change rather than recomputing from scratch [7]. Differential dataflow and DBSP rely on basic principles: inserts add values, deletes subtract them, and multiple workers' results get merged back together in arbitrary order. That only works if the values form what's called an abelian group: addition that's associative and commutative, and can be backed out exactly. Integers are an abelian group. 1 + 10 - 10 does equal 1. Floats aren't an abelian group, as we've already seen:

```rust
// insert 1e100, insert 1.0, delete the 1e100. it should be back to 1.0, right? ...right?
assert_eq!(1e100 + 1.0 - 1e100, 0.0); // nope. f64 says it's empty.
```

Incremental engines drop entries at zero, so leftover crumbs keep dead entries alive and false zeros delete live ones. Feldera/DBSP says as much in its docs: its linear aggregates "are actually non-linear when applied to `DOUBLE` or `FLOAT` values" [8], and a float `SUM` in one "will produce incorrect results or runtime crashes" [9].

I ran into this wanting personalized PageRank and graph Laplacians, among other things, maintained incrementally in differential dataflow. Their graph weights are floats. The usual ways around it each give something up:

| approach | same bits in any order | exact retraction | every f64 | fixed size |
|---|---|---|---|---|
| plain `f64` | no | no | yes | 8 B |
| compensated summation [10, 11] | no | no | yes | 16 B |
| scaled integers (fixed point) | yes | yes | no: the range is picked in advance | yes |
| binned summation, ReproBLAS [12, 13] | yes | no | yes | yes |
| a long accumulator [14, 15] | yes | yes | yes | yes, but about 300 B |

Only the last row passes every column, and it's old: Kulisch proposed a fixed-point register wide enough for every product of two f64s [14], and Neal's superaccumulators are a fast software form [15]. float-abel's `Wide` is that row, sized for sums of f64s rather than products. (It's also the approach the bitrep project uses.) `Exact` is the same group in 32 bytes that move to the heap only when a sum needs more than 127 bits.

### why binned summation isn't enough

The standard answer to order-dependent sums is Demmel and Nguyen's binned summation, implemented in ReproBLAS [12, 13]. It's fast, it's constant size, and a one-shot sum comes out the same in any order. It works by keeping a few bins positioned relative to the largest value seen so far, and cutting off everything below the lowest bin at the same place for every input.

That position is the problem. The state remembers the largest value it ever saw, and no retraction can make it forget, so it isn't an abelian group. float-abel ships this method as `Binned`, for batch work, so you can watch it happen:

```rust
use float_abel::{Binned, Exact};

let f64_sum = |xs: &[f64]| xs.iter().sum::<f64>();
let binned = |xs: &[f64]| xs.iter().copied().map(Binned::<3>::from).sum::<Binned<3>>().into_wide().to_f64();
let exact = |xs: &[f64]| xs.iter().copied().map(Exact::from).sum::<Exact>().to_f64();

// Insert 1e100 and 1.0 and retract the 1e100, three different ways
let (a, b) = ([1e100, 1.0, -1e100], [1e100, -1e100, 1.0]); // same values, different orders
assert_eq!((f64_sum(&a), f64_sum(&b)), (0.0, 1.0)); // f64: the answer changes with the order
assert_eq!((binned(&a), binned(&b)), (0.0, 0.0)); // binned: consistently wrong
assert_eq!((exact(&a), exact(&b)), (1.0, 1.0)); // exact: always right
```

## references

1. D. Goldberg. What every computer scientist should know about floating-point arithmetic. *ACM
   Computing Surveys* 23(1):5–48, 1991. [doi:10.1145/103162.103163](https://doi.org/10.1145/103162.103163)
2. F. McSherry, D. G. Murray, R. Isaacs, M. Isard. Differential dataflow. *CIDR*, 2013.
   [cidrdb.org](https://www.cidrdb.org/cidr2013/Papers/CIDR13_Paper111.pdf)
3. M. Budiu, T. Chajed, F. McSherry, L. Ryzhyk, V. Tannen. DBSP: Automatic incremental view
   maintenance for rich query languages. *PVLDB* 16(7):1601–1614, 2023.
   [doi:10.14778/3587136.3587137](https://doi.org/10.14778/3587136.3587137)
4. IEEE Standard for Floating-Point Arithmetic, IEEE Std 754-2019.
   [doi:10.1109/IEEESTD.2019.8766229](https://doi.org/10.1109/IEEESTD.2019.8766229)
5. C. Collange, D. Defour, S. Graillat, R. Iakymchuk. Numerical reproducibility for the parallel
   reduction on multi- and many-core architectures. *Parallel Computing* 49:83–97, 2015.
   [doi:10.1016/j.parco.2015.09.001](https://doi.org/10.1016/j.parco.2015.09.001)
6. W. Ahrens, J. Demmel, H. D. Nguyen. Algorithms for efficient reproducible floating point
   summation. *ACM TOMS* 46(3), 2020. [doi:10.1145/3389360](https://doi.org/10.1145/3389360)
7. J. A. Blakeley, P.-Å. Larson, F. W. Tompa. Efficiently updating materialized views. *SIGMOD*,
   61–71, 1986. [doi:10.1145/16894.16861](https://doi.org/10.1145/16894.16861)
8. Feldera. Aggregate operations. [docs.feldera.com/sql/aggregates](https://docs.feldera.com/sql/aggregates)
9. Feldera. User-defined functions. [docs.feldera.com/sql/udf](https://docs.feldera.com/sql/udf)
10. W. Kahan. Further remarks on reducing truncation errors. *CACM* 8(1):40, 1965.
    [doi:10.1145/363707.363723](https://doi.org/10.1145/363707.363723)
11. N. J. Higham. *Accuracy and Stability of Numerical Algorithms*, 2nd ed., ch. 4. SIAM, 2002.
    [doi:10.1137/1.9780898718027](https://doi.org/10.1137/1.9780898718027)
12. J. Demmel, H. D. Nguyen. Fast reproducible floating-point summation. *ARITH 21*, 163–172,
    2013. [doi:10.1109/ARITH.2013.9](https://doi.org/10.1109/ARITH.2013.9)
13. J. Demmel, H. D. Nguyen. Parallel reproducible summation. *IEEE Transactions on Computers*
    64(7):2060–2070, 2015. [doi:10.1109/TC.2014.2345391](https://doi.org/10.1109/TC.2014.2345391)
14. U. Kulisch. *Computer Arithmetic and Validity*, 2nd ed. De Gruyter, 2013.
    [doi:10.1515/9783110301793](https://doi.org/10.1515/9783110301793)
15. R. M. Neal. Fast exact summation using small and large superaccumulators. arXiv:1505.05571,
    2015. [arxiv.org/abs/1505.05571](https://arxiv.org/abs/1505.05571)

---

MIT OR Apache-2.0, at your option. Report a vulnerability as [`SECURITY.md`](SECURITY.md) says.
Developed with Claude.
