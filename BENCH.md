# Measured

The numbers DESIGN.md § Targets and kill conditions reads from. Advisory, never a gate: a missed
target is a design question, not a broken build. Every number names the machine, the toolchain and
the commit it was taken on, and the two architectures are reported side by side, never blended.

The harness is `benches/simd.rs` under criterion 0.8.2 with its defaults (3 s warm-up, 5 s of
samples, 100 samples); a figure is criterion's point estimate of the time per run, divided by the
run's 2²⁰ elements. Peak RSS is not measured here. `cargo bench --bench simd` re-takes the table.

## SIMD earns its feature

Each tier's bulk entry against the one-at-a-time deposits that define it, on the design's same-scale
fixture: 2²⁰ normal doubles with top bits in [2⁻²⁰, 2²⁰), full random fractions, random signs. The
kill condition is a kernel under 3× its scalar reference; the design's absolute targets are 1.0 ns
(`Fixed`), 0.5 ns (`Binned`) and 1.5 ns (`Wide`) per value.

| tier, same-scale | NEON: one at a time | NEON: bulk | NEON gain | AVX2: one at a time | AVX2: bulk | AVX2 gain |
|---|--:|--:|--:|--:|--:|--:|
| `Wide` | 18.62 ns | 1.81 ns | 10.3× | 36.55 ns | 2.52 ns | 14.5× |
| `Fixed256` | 7.87 ns | 1.59 ns | 4.9× | 16.34 ns | 2.02 ns | 8.1× |
| `Binned<3>` | 10.63 ns | 1.98 ns | 5.4× | 18.17 ns | 1.69 ns | 10.8× |
| vectorised naive `f64` sum, the floor | | 0.076 ns | | | 0.366 ns | |

Every kernel clears the kill condition on both architectures. No kernel reaches its absolute
target: `Fixed` lands at 1.6× its NEON target, `Wide` at 1.2×, and `Binned` at 4× (NEON) and 3.4×
(AVX2) of a 0.5 ns target that assumed five vector ops per limb and value, where the kernels spend
about seven: a per-limb shift count that has to stay inside a signed byte on NEON, a left and a right
shift on AVX2, and the carry-free digit accumulation.

- NEON: Apple M5 Max, macOS, rustc 1.99.0, commit 0893692. Two `u64` lanes a step.
- AVX2: AMD Ryzen 7 5700U, Ubuntu 24.04, rustc 1.96.1, commit 0893692. Four `u64` lanes a step.

## Where the window falls back

`Wide` on 2²⁰ doubles with top bits uniform in [2⁻³⁰⁰, 2³⁰⁰): a tile spans far more than the
four-limb window, so almost every vector carries a lane outside it and goes to the scalar deposit.

| `Wide`, 2^±300 | one at a time | bulk |
|---|--:|--:|
| NEON | 18.35 ns | 18.12 ns |
| AVX2 | 35.88 ns | 37.86 ns |

The bulk entry is the scalar fold plus the span pass here; AVX2 pays 5% for it. That fixture is not
a column, and the design's `Wide` target is stated on real columns, but it bounds the cost of the
fallback.
