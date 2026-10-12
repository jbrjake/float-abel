---
id: fl-b41y9kt
title: "Ship the simd feature — runtime-dispatched NEON and AVX2 (AVX-512 where present) bulk kernels for Fixed, Binned and Wide, a scalar reference kernel, and the golden hash over every path"
category: crate/simd
labels: [reproducible-floats, rf-tail]
docs: [docs/DESIGN.md#speed-sp-simd]
needs: [fl-n7beqqc, fl-at6nh41, fl-9kp61j0, fl-v7t6xhz]
seq: 120
verify: run cargo test package=float-abel simd_
status: doing
created: 2026-10-11T15:02Z
covers:
  - ref: docs/DESIGN.md#sp-simd
    sha: 8ff7be5375be41fcd0546ebe39bb157963b16daf08be459475027b1bf597fd3d
handoff: |
  The tree is at 014aa26 plus the store commit after it. Green this
  session on that tree:
  `./scripts/regression.sh` end to end, exit 0 (fmt, the three clippy
  legs, rustdoc with warnings
  denied, `cargo test -- --include-ignored`, the release `every_build`
  leg, noalloc, golden, the MSRV
  1.74 build, cargo-deny with the sandbox off); every `simd_*` test on
  this aarch64 machine (NEON) and
  on jons-amd-mini (Ryzen 7 5700U, AVX2, rustc 1.96.1), where the whole
  suite but the ui snapshots
  (fl-ceyy1h6), release `every_build`, noalloc and golden also passed; and
  golden on wasm32-wasip1
  under wasmtime 49.0.2 (scratchpad install, see memory for the recipe).
  The golden hash
  b0ec15cefcb8dc9dcdad25bdab61ab08b87af84da163adaa2c054cf4039ea2e5, BYTES
  65_557_243, was blessed
  here and reproduced on the Ryzen through AVX2 and on wasm32 through the
  portable kernel.
  
  Four commits: b34ac9d the bulk entries (`extend_from_slice` and
  `deposit_products` on Wide, Fixed
  and Binned, with Fixed's and Wide's product entries checking every pair
  first) plus the portable and
  NEON kernels and tests/simd.rs; 0893692 the AVX2 kernel; 7a8428a the
  golden transcript with `simd`
  on and the bulk records; 014aa26 benches/simd.rs under criterion 0.8.2
  and BENCH.md.
  
  Shape of src/simd/: mod.rs holds `Partials` (per-limb 32-bit digit sums
  plus the count of
  negatives; `twos_complement()` for Fixed and Wide, `bin()` for Binned),
  `fold_tile`'s dispatch, the
  `span`/`top_limb` pre-passes with debug cross-checks against the
  portable ones, and the three drivers
  `wide_extend`, `fixed_extend`, `binned_extend` (TILE = 1024). scalar.rs
  is the specification:
  `Lane::decode` and `Lane::word`, `fold_tile<N, LIMBS, TRUNC>`, and unit
  tests pinning it to
  `set_shifted`, to a limb-arithmetic oracle, and to lane-count
  independence at 1, 2, 4 and 8. neon.rs
  places with USHL from a saturating byte chain centred on limb 1 and
  accumulates with UADDW; it takes
  LIMBS ≤ 4 exact and K ≤ 3 truncating. avx2.rs places with sllv|srlv
  and takes LIMBS ≤ 4; its safe
  entries assert `available()`. The kernel contract: a lane vouches for an
  input only when it
  certainly fits the window (the soundness test is conservative for
  subnormals near the ceiling), and
  a vector with any unvouched lane goes whole to the tier's scalar
  deposit, so a kernel never decides a
  refusal. Each kernel was broken four ways on purpose and tests/simd.rs
  caught each.
  
  Measured, BENCH.md: same-scale bulk over scalar, NEON 10.3× Wide, 4.9×
  Fixed256, 5.4× Binned<3>;
  AVX2 14.5×, 8.1×, 10.8×. The 3× kill condition is cleared
  everywhere; the absolute targets are
  missed (NEON 1.81, 1.59, 1.98 ns per value against 1.5, 1.0, 0.5). The
  2^±300 Wide fixture runs at
  scalar speed because tiles outgrow one window; a multi-window pass per
  tile could recover some and
  belongs to the Measure task.
  
  Remaining for this task, in order:
  
  1. Binned kernel (ii), the owner's ruling this session: "Build (ii) as a
  bench-only experiment".
  Plan: 2K FP bins of 32 bits on Wide's half-limb grid per tile, the index
  from `top_limb` as (i)
  takes it; per input, t_j(x) = x truncated toward zero at 2^(lsb +
  32(j+1)) through
  (M_j + x) − M_j with M_j = 1.5 · 2^(lsb + 32(j+1) + 52) and a fix-up
  when |t_j| > |x| (subtract
  sign(x) · 2^(lsb + 32(j+1))); pieces piece_j = t_{j−1} − t_j, with
  t_{−1} the truncation at
  2^lsb and t_{2K−1} = 0 under the index; each bin accumulates its
  pieces in f64, exact up to 2^21
  adds (TILE is 1024); at the tile's end each bin converts to an integer
  in units of
  2^(lsb + 32j) and the pairs combine into (limb, carry) through i128 as
  `Partials::bin` does.
  Specials and zeros go to the leftover path. Build it portable over
  `[f64; 4]` lanes in
  src/simd/fp_bins.rs (plain f64 add, sub, compare and select; LLVM
  vectorises them; on x86 the
  autovectoriser needs an avx2 `target_feature` fn to use 256-bit
  registers, else it is SSE2, and
  BENCH.md says which), hold it to `binned_loop` in tests/simd.rs with the
  same bit-identity test
  as (i), bench it beside (i) in benches/simd.rs on both machines, write
  both into BENCH.md, delete
  the loser as the design says, and amend DESIGN.md § Speed's Binned row
  to name the survivor and
  the ratio; `cover fl-b41y9kt docs/DESIGN.md#sp-simd --repin` after. The
  probe comment on this
  task estimates (ii) at about 16K flops per value against (i)'s 5K
  integer ops, so it should lose
  by 2–3×. If it cannot be made bit-identical, that is the finding, and
  the row says so.
  
  2. The SIMD product kernel: `deposit_products` on every tier runs the
  scalar check-then-deposit
  today, bit-identical by construction and covered by
  simd_deposit_products_* and the golden. The
  design's kernel table has no product row; vectorising the 106-bit
  product (four 32×32 multiplies
  a lane, three limbs placed per value) is the owner's call. Ask in one
  question, or file it beside
  the Measure task.
  
  3. Close with `close fl-b41y9kt` once (1) is in; the verify, `run cargo
  test package=float-abel
  simd_`, is green now at 11 tests.
  
  Filed on the way: fl-qv0vh39 (the AVX-512 kernel; no machine on hand has
  AVX-512, and its
  intrinsics need rustc 1.89) and fl-ceyy1h6 (tests/ui's .stderr files
  match rustc 1.99 only).
  
  After this: fl-06sggdc (dbsp, with the DBSP compile-fail case owed to
  it), fl-ajqrs9p,
  fl-c3ahdwc, then the rest of the tail; fl-e4jjqf1 (Measure) needs this
  task closed. The portfolio
  ask fl-cfx001b is still out.
claimed-by: claude (8fa1b77c-414e-4389-9cab-7906d84600b5)
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

## comments
- 2026-10-12T20:07Z [claude (8fa1b77c-414e-4389-9cab-7906d84600b5)] Probe, 2026-10-12, before building. The owner offered jons-amd-mini (ssh ubuntu@192.168.86.160) for x86 SIMD work: Ryzen, Ubuntu 24.04 kernel 6.17, 16 threads, 28 GB, rustc 1.96.1; cpu flags carry avx, avx2, fma, bmi1, bmi2 and no avx512* flag at all. So AVX2 kernels can run and be timed there; AVX-512 has no machine here or there, and this machine is aarch64 (NEON).
  
  Binned kernel (ii), ReproBLAS's `q = (M + x) − M`, cannot equal the scalar `Binned` fold bit for bit, independent of the floating-point ban: a bin here is a 64-bit limb, and an f64 holds 53 significant bits, so the sum of two slices of one bin (each up to 53 significant bits at different offsets inside the limb) already does not fit the FP accumulator; and `(M + x) − M` rounds to nearest at the limb boundary where `Binned` truncates toward zero, so the top piece of an input near a boundary rounds up and the next piece goes negative, which puts different integers in the bins for the same total. Matching the bins would need 32-bit FP bins (2K of them) with a truncation fix-up per piece, about 14K flops per value against the integer kernel's 5K ops, so (ii) loses on op count before it is built and could never pass the `simd_*` bit-identity test as specified.
  
  MSRV: with AVX-512 out, every intrinsic the kernels need (aarch64 NEON, stable 1.59; `_mm256_*` AVX2 and `is_x86_feature_detected!`, stable 1.27) is on 1.74, so the `simd` feature can keep the crate's MSRV; to be verified with `cargo +1.74.0 build --features simd`.
- 2026-10-12T21:09Z [claude (8fa1b77c-414e-4389-9cab-7906d84600b5)] Progress, 2026-10-12. Owner rulings this session, in the owner's words: on the kernel set, "try jrubin@turing.tailb10f9.ts.net" (probed: i7-5930K, AVX2 and FMA, no AVX-512; jons-amd-mini is a Ryzen 7 5700U, AVX2, no AVX-512; so scalar + NEON + AVX2 ship and AVX-512 is a follow-up task); on Binned kernel (ii), "Build (ii) as a bench-only experiment"; on the harness, "Minimal criterion bench here".
  
  Landed: b34ac9d the bulk entries and the portable + NEON kernels; 0893692 the AVX2 kernel; then the golden transcript with simd on and the bench harness with BENCH.md. Watched green: every simd_ test on aarch64 (NEON) and on the Ryzen (AVX2); the full suite, release every_build, noalloc and golden on the Ryzen; the golden hash b0ec15cefcb8dc9dcdad25bdab61ab08b87af84da163adaa2c054cf4039ea2e5 (65,557,243 bytes) blessed on aarch64 and reproduced on the Ryzen through AVX2 and on wasm32-wasip1 under wasmtime 49.0.2 through the portable kernel. Each kernel was broken on purpose four ways and the simd_ tests caught each.
  
  Measured (BENCH.md): same-scale bulk vs scalar, NEON 10.3× Wide, 4.9× Fixed256, 5.4× Binned<3>; AVX2 14.5×, 8.1×, 10.8×. Kill condition (3×) cleared everywhere; absolute targets missed (NEON 1.81, 1.59, 1.98 ns/value against 1.5, 1.0, 0.5). The 2^±300 Wide fixture runs at scalar speed: tiles outgrow the window, as the design says.
  
  Finding, not this task's: tests/ui's trybuild .stderr snapshots match rustc 1.99's diagnostics and fail on the Ryzen's 1.96.1 (fixed_to_bytes_wrong_length), so the push gate is green only on the compiler that blessed them; filed as its own task.
  
  Remaining here: the Binned kernel (ii) experiment and its A/B, the DESIGN.md § Speed row for whichever loses, the SIMD product kernel (deposit_products runs the scalar check-then-deposit on every tier today, bit-identical by construction), and the handoff.
