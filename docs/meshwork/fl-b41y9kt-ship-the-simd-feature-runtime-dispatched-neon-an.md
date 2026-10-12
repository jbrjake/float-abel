---
id: fl-b41y9kt
title: "Ship the simd feature — runtime-dispatched NEON and AVX2 (AVX-512 where present) bulk kernels for Fixed, Binned and Wide, a scalar reference kernel, and the golden hash over every path"
category: crate/simd
labels: [reproducible-floats, rf-tail]
docs: [docs/DESIGN.md#speed-sp-simd]
needs: [fl-n7beqqc, fl-at6nh41, fl-9kp61j0, fl-v7t6xhz]
seq: 120
verify: run cargo test package=float-abel simd_
status: open
created: 2026-10-11T15:02Z
covers:
  - ref: docs/DESIGN.md#sp-simd
    sha: 8ff7be5375be41fcd0546ebe39bb157963b16daf08be459475027b1bf597fd3d
handoff: |
  The tree is at 42c990c plus the store commit after it. Green this
  session on that tree:
  `scripts/smoke.sh` through the pre-commit hook (152 tests), and
  `./scripts/regression.sh` end to
  end, exit 0: fmt, the three clippy legs including `--workspace
  --all-targets`, rustdoc with
  warnings denied, `cargo test -- --include-ignored`, the release
  `every_build` leg, the noalloc and
  golden members, the MSRV 1.74 build, and cargo-deny (the gate needs the
  sandbox off for the
  advisory fetch). The golden member also landed the re-blessed hash on
  wasm32-wasip1 under
  wasmtime 49.0.2, rebuilt into this session's scratchpad with `cargo
  install wasmtime-cli --locked
  --root <dir> --target-dir <dir>` (two minutes; it dies with the session)
  and run with its own
  `CARGO_TARGET_DIR`. Not run: `.github/workflows/golden.yml`'s five other
  targets, which only the
  owner's push exercises.
  
  Since the last handoff, Binned's product deposit landed at 42c990c.
  What this task inherits from it:
  
  - `Binned::deposit_finite(negative, mant: u128, shift: u32)` in
  src/binned.rs is now the one
  slicer for a double and a product: three words through
  `limbs::set_shifted`, each added to its
  bin or dropped below the window. That is the scalar reference kernel's
  shape for `Binned`.
  - `Binned::deposit_product` and `Binned::deposit_products` exist, with
  the owner's ruling, on
  the archived product task's comment: bits below 2^-1074 are truncated
  like any bit below the window (the bound
  for products carries a 2^-1074 floor, DESIGN.md § Binned), the ceiling
  is `Wide`'s 2^1037
  refused as `AboveCeiling`, and the bulk entry checks every pair before
  depositing any, so a
  refusal leaves the state untouched. `product`/`finite_product` in
  src/binned.rs are the
  classifier a kernel's pre-pass can reuse; `Wide::CEILING_BITS` is
  `pub(crate)`.
  - `Wide` and `Fixed` still have neither `extend_from_slice` nor
  `deposit_products`; DESIGN.md
  § Operations and conversions says every tier has both, and this task's
  body owns them.
  - The golden transcript (tests/golden/main.rs) now records `Binned`'s
  product of the first and
  last inputs and the multiset's dot product with its own reverse: BYTES
  60,470,210, hash
  5e21d2e5857423dddfdacba60a71a4e94ad3127b7af0e8ad866e66658a629322,
  blessed on aarch64 macOS.
  The member builds the lib on default features; turn `simd` on there once
  kernels exist, so the
  hash covers every path, and bless the new hash here if the record layout
  changes.
  
  Scope, per DESIGN.md § Speed and CLAUDE.md:
  `src/simd/{mod,scalar,neon,avx2,avx512}.rs`, with
  `#![deny(unsafe_code)]` lifted only under src/simd/ and a `// SAFETY:`
  line per block; runtime
  dispatch (`is_x86_feature_detected!("avx2")`, AVX-512 where present,
  NEON unconditionally on
  aarch64); lane-parallel partials merged at the tile's end; both `Binned`
  kernels built and
  benchmarked with the loser deleted; tests `simd_*` (every kernel equals
  the scalar kernel bit for
  bit on the vectors and 10⁴ seeded multisets, lane-count independence,
  the golden hash over every
  kernel). The verify is `cargo test package=float-abel simd_`.
  
  Ask before building, through one `AskUserQuestion`, because each of
  these is the owner's:
  
  1. This machine is aarch64 macOS: NEON kernels run and test here, AVX2
  and AVX-512 kernels can
  only be compiled here and run on CI's x86 runner (Rosetta 2's AVX2
  support is untested).
  Build all three blind-to-x86 in one task, or ship scalar + NEON first
  and file AVX2/AVX-512 as
  a follow-up that lands only once the x86 runner is green? My
  recommendation: the latter, since
  "done means you watched it work".
  2. The ReproBLAS-shaped `Binned` kernel (ii), `q = (M + x) − M` per
  bin, executes floating-point
  adds inside a fold. The standing decision bans floating-point
  instructions in group ops and
  reads; `Binned` is not a group, but the golden hash claims architecture
  independence for its
  bits, and IEEE adds without FMA contraction are deterministic. Confirm
  (ii) may use them for
  the A/B, or restrict `Binned` to the integer kernel (i).
  3. No `benches/`, `bench-baseline.json` or BENCH.md exists yet, and the
  A/B needs criterion with
  a per-machine baseline (CLAUDE.md § Performance). Is the harness part
  of this task or of
  fl-e4jjqf1 ("Measured")? src/lib.rs's doc says `std` "adds the bench
  harness".
  4. MSRV: the crate asserts 1.74 on default features only.
  `#[target_feature(enable = "neon")]`
  and `is_x86_feature_detected!` are fine there; the AVX-512 intrinsics
  stabilised later, so the
  `simd` feature may need a newer compiler than the MSRV. Say so in
  Cargo.toml and CI if so.
  
  After this: fl-06sggdc (dbsp, with the DBSP compile-fail case owed to
  it), fl-ajqrs9p,
  fl-c3ahdwc, then the rest of the tail. The portfolio ask fl-cfx001b is
  still out.
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
