---
id: fl-b41y9kt
title: "Ship the simd feature — runtime-dispatched NEON and AVX2 (AVX-512 where present) bulk kernels for Fixed, Binned and Wide, a scalar reference kernel, and the golden hash over every path"
category: crate/simd
labels: [reproducible-floats, rf-tail]
docs: [docs/DESIGN.md#speed-sp-simd]
needs: [fl-n7beqqc, fl-at6nh41, fl-9kp61j0, fl-v7t6xhz]
seq: 120
verify: run cargo test package=float-abel simd_
status: done
created: 2026-10-11T15:02Z
covers:
  - ref: docs/DESIGN.md#sp-simd
    sha: a53015669131214311a8448051b1429446d34946c9595882c74ee575aa364115
---
Per DESIGN.md § Speed. `src/simd/{mod,scalar,neon,avx2,avx512}.rs`, with `unsafe` only here and a
`// SAFETY:` line per block. `extend_from_slice` and `deposit_products` on the three tiers;
lane-parallel partials merged at the tile's end. Both `Binned` kernels are built and benchmarked, and
the loser is deleted.

Tests `simd_*`:
- every kernel equals the scalar kernel bit for bit on the conformance and rounding vectors and on
  10⁴ seeded multisets, specials and subnormals included;
- lane-count independence;
- the golden hash extended to every kernel.

## log
- 2026-10-11T15:02Z created
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-simd @8ff7be5375be
- 2026-10-12T19:58Z handoff by claude (698bf172-c634-41d0-9a55-81a5fdd2e3db)
- 2026-10-12T19:58Z handoff by claude (698bf172-c634-41d0-9a55-81a5fdd2e3db)
- 2026-10-12T20:07Z open→doing — claimed by claude (8fa1b77c-414e-4389-9cab-7906d84600b5)
- 2026-10-12T21:12Z handoff by claude (8fa1b77c-414e-4389-9cab-7906d84600b5)
- 2026-10-12T21:35Z cover docs/DESIGN.md#sp-simd @a53015669131
- 2026-10-12T21:36Z doing→done — verify exit 0 @ 0ae1731+2

## comments
- 2026-10-12T20:07Z [claude (8fa1b77c-414e-4389-9cab-7906d84600b5)] Probe, 2026-10-12, before building. The owner offered jons-amd-mini (ssh ubuntu@192.168.86.160) for x86 SIMD work: Ryzen, Ubuntu 24.04 kernel 6.17, 16 threads, 28 GB, rustc 1.96.1; cpu flags carry avx, avx2, fma, bmi1, bmi2 and no avx512* flag at all. So AVX2 kernels can run and be timed there; AVX-512 has no machine here or there, and this machine is aarch64 (NEON).
  
  Binned kernel (ii), ReproBLAS's `q = (M + x) − M`, cannot equal the scalar `Binned` fold bit for bit, independent of the floating-point ban: a bin here is a 64-bit limb, and an f64 holds 53 significant bits, so the sum of two slices of one bin (each up to 53 significant bits at different offsets inside the limb) already does not fit the FP accumulator; and `(M + x) − M` rounds to nearest at the limb boundary where `Binned` truncates toward zero, so the top piece of an input near a boundary rounds up and the next piece goes negative, which puts different integers in the bins for the same total. Matching the bins would need 32-bit FP bins (2K of them) with a truncation fix-up per piece, about 14K flops per value against the integer kernel's 5K ops, so (ii) loses on op count before it is built and could never pass the `simd_*` bit-identity test as specified.
  
  MSRV: with AVX-512 out, every intrinsic the kernels need (aarch64 NEON, stable 1.59; `_mm256_*` AVX2 and `is_x86_feature_detected!`, stable 1.27) is on 1.74, so the `simd` feature can keep the crate's MSRV; to be verified with `cargo +1.74.0 build --features simd`.
- 2026-10-12T21:09Z [claude (8fa1b77c-414e-4389-9cab-7906d84600b5)] Progress, 2026-10-12. Owner rulings this session, in the owner's words: on the kernel set, "try jrubin@turing.tailb10f9.ts.net" (probed: i7-5930K, AVX2 and FMA, no AVX-512; jons-amd-mini is a Ryzen 7 5700U, AVX2, no AVX-512; so scalar + NEON + AVX2 ship and AVX-512 is a follow-up task); on Binned kernel (ii), "Build (ii) as a bench-only experiment"; on the harness, "Minimal criterion bench here".
  
  Landed: b34ac9d the bulk entries and the portable + NEON kernels; 0893692 the AVX2 kernel; then the golden transcript with simd on and the bench harness with BENCH.md. Watched green: every simd_ test on aarch64 (NEON) and on the Ryzen (AVX2); the full suite, release every_build, noalloc and golden on the Ryzen; the golden hash b0ec15cefcb8dc9dcdad25bdab61ab08b87af84da163adaa2c054cf4039ea2e5 (65,557,243 bytes) blessed on aarch64 and reproduced on the Ryzen through AVX2 and on wasm32-wasip1 under wasmtime 49.0.2 through the portable kernel. Each kernel was broken on purpose four ways and the simd_ tests caught each.
  
  Measured (BENCH.md): same-scale bulk vs scalar, NEON 10.3× Wide, 4.9× Fixed256, 5.4× Binned<3>; AVX2 14.5×, 8.1×, 10.8×. Kill condition (3×) cleared everywhere; absolute targets missed (NEON 1.81, 1.59, 1.98 ns/value against 1.5, 1.0, 0.5). The 2^±300 Wide fixture runs at scalar speed: tiles outgrow the window, as the design says.
  
  Finding, not this task's: tests/ui's trybuild .stderr snapshots match rustc 1.99's diagnostics and fail on the Ryzen's 1.96.1 (fixed_to_bytes_wrong_length), so the push gate is green only on the compiler that blessed them; filed as its own task.
  
  Remaining here: the Binned kernel (ii) experiment and its A/B, the DESIGN.md § Speed row for whichever loses, the SIMD product kernel (deposit_products runs the scalar check-then-deposit on every tier today, bit-identical by construction), and the handoff.
- 2026-10-12T21:24Z [claude (357feee9-5111-41e9-8f17-91f2ac458b5c)] Plan for Binned kernel (ii), 2026-10-12, before building. The owner's ruling stands as recorded: "Build (ii) as a bench-only experiment".
  
  Shape that can be bit-identical to the scalar fold: the tile's index and lsb as (i) takes them; per input, the magnitude is peeled top-down through 2K half-bins of 32 bits on Wide's half-limb grid, each level q = (M + m) - M with M = 1.5 * 2^(e + 52) (nearest, exact by Sterbenz), then truncation toward zero by one fix-up (q > m means q -= 2^e), the piece signed by one xor and added to a per-lane f64 bin, the remainder m -= q. Every piece is a multiple of 2^e under 2^32 of them, so 1024 adds stay exact; at the tile's end each half-bin's sum is read as an integer through its bits (no FP op), the two halves of a limb combine through i128 as Partials::bin does, and the tile merges through Add. A vector with a special goes whole to the scalar deposit, as in (i); zeros ride the lanes. The constants stay finite only while the top half-bin's rounding constant does: index <= 31, so a tile holding anything at or above 2^974 goes whole to the scalar deposit.
  
  Beside it, a nearest-rounding variant without the fix-up (ReproBLAS's actual kernel shape, 5 ops a level), bench-only and never bit-identical, so BENCH.md can say whether the FP shape loses on its own or on the truncation.
  
  Files: src/simd/fp_bins.rs (portable over [f64; 4] lanes, autovectorised; on x86_64 an avx2 target_feature wrapper so the vectoriser has 256-bit registers), binned_extend_fp in src/simd/mod.rs, two #[doc(hidden)] entries on Binned for the test and the bench to reach, one simd_ test holding (ii) to binned_loop at K = 1, 2, 3, 5, onto a held state, and on the mixing fixture (subnormals, zeros, specials, inputs past the index bound); two bench rows in binned3/same-scale. Then NEON here, AVX2 on jons-amd-mini, BENCH.md, DESIGN.md's Binned row, delete the loser, repin.
