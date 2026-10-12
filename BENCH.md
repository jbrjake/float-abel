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
