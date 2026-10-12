---
id: fl-bzqkv7x
title: "Give Binned deposit_product and deposit_products — the 106-bit product sliced into the bins with the product's sign, refused by name past the ceiling"
status: open
category: crate/types
labels: [reproducible-floats, rf-tail]
relates: [fl-v7t6xhz, fl-b41y9kt]
verify: run cargo test package=float-abel binned_product_
docs:
  - docs/DESIGN.md#operations-and-conversions-sp-ops
seq: 115
created: 2026-10-12T15:10Z
handoff: |
  The tree is at a6a17f3 plus the store commit after it. Green this
  session: `scripts/smoke.sh`
  through the pre-commit hook on both code commits (143 tests each), and
  `./scripts/regression.sh`
  end to end once on the final tree: the three clippy legs including
  `--workspace --all-targets`,
  rustdoc with warnings denied, `cargo test -- --include-ignored`, the
  release `every_build` leg,
  the noalloc and golden members, the MSRV 1.74 build with the new
  lockfile, and cargo-deny.
  `cargo test -p float-abel-golden` also lands the committed hash on
  wasm32-wasip1 under wasmtime
  49.0.2 here (`cargo install wasmtime-cli --locked --root <dir>`, then
  `CARGO_TARGET_WASM32_WASIP1_RUNNER=wasmtime cargo test -p
  float-abel-golden --target
  wasm32-wasip1` with its own `CARGO_TARGET_DIR`). Not run:
  `.github/workflows/golden.yml` itself,
  which asserts the five other targets only once the owner pushes; Docker
  was down, so the cross
  legs were never exercised locally.
  
  Since the last handoff, "Prove bit-identity across architectures"
  (fl-9kp61j0) closed:
  `tests/golden/` is a second test workspace member (lib + serde_json +
  sha2 0.10), `main.rs` the
  transcript and the two `golden_*` tests, `vectors.rs` the decoders
  copied from tests/rounding.rs
  and tests/wide.rs, `stream.rs` a frozen copy of tests/support.rs's
  `Rng`. The claim is
  `tests/data/golden.sha256`; `ELEMENTS` and `BYTES` in main.rs pin the
  record count ahead of the
  hash. Its comment carries the two owner questions (the second member
  against CLAUDE.md override 3;
  sha2 and serde_json as that member's dependencies) and the simd
  follow-up: the member builds the
  lib on default features, so turn `simd` on there once kernels exist.
  "Fix the noalloc member's
  assign-op clippy error" (fl-w9mk4jc) closed on the way: the push gate at
  818f4a1 was red there.
  
  This task, per DESIGN.md § Operations and conversions:
  `deposit_product(a, b)` and
  `deposit_products(&[f64], &[f64])` on `Binned<K>` in `src/binned.rs`.
  The model is
  `Wide::deposit_product` in `src/wide.rs` (`ieee::factor` on both sides,
  the 106-bit mantissa
  product as a `u128`, the sign rule for specials, a zero factor
  depositing nothing); `Binned`'s own
  `deposit`, `extend_from_slice`, `into_wide` and `Wide::to_binned` show
  how a magnitude is sliced
  onto the bins at Wide's limb boundaries and how the window moves. The
  refusal type is
  `OutOfWindow` in `src/fixed.rs`. Tests go in `tests/binned.rs` as
  `binned_product_*` (the verify's
  filter); its `truncated_sum` oracle, `fold` and `window_bottom` already
  express "equal to Wide
  truncated below the window", which is the property a product deposit
  must meet too.
  
  Ask before building, through one `AskUserQuestion` with both semantics
  and a recommendation,
  because neither is in the spec: (1) a product with a set bit below
  2^-1074, which `Wide` refuses
  as `OffGrid`: truncate it like every other bit below the window (my
  recommendation: the tier
  drops bits by definition and the bound holds either way) or refuse as
  `Wide` does; (2) a product
  at or past 2^1037, `AboveCeiling` on `Wide`: refuse by name, since
  `into_wide` cannot place a
  carry above limb 33, which makes the method fallible. Build nothing on
  the first until the owner
  answers; the second has only one sound answer and can be stated as such.
  
  If the product deposit should enter the golden hash (it is not in the
  transcript, which calls
  nothing that does not exist), add a record to `absorb` in
  `tests/golden/main.rs`, bless the new
  hash on this machine (aarch64 macOS) and say so in the commit: an input
  change moves the claim,
  a target mismatch never does.
  
  After this: fl-b41y9kt (simd, with the golden `simd` follow-up),
  fl-06sggdc (dbsp, with the
  DBSP compile-fail case owed to it), fl-ajqrs9p, fl-c3ahdwc, then the
  rest of the tail. The
  portfolio ask fl-cfx001b is still out.
---

DESIGN.md § Operations and conversions: "Every tier has the bulk entry points
`extend_from_slice(&[f64])` and `deposit_products(&[f64], &[f64])`; they are the SIMD surface."
`Binned` has `extend_from_slice` and no product entry. Add `deposit_product(a, b)` and
`deposit_products(&[f64], &[f64])` in src/binned.rs: the 106-bit mantissa product as a `u128`
(`ieee::factor` on both sides, as `Wide::deposit_product` does), sliced at Wide's limb boundaries
into up to three bins with the product's sign, specials by the same sign rule as Wide's, and a
zero factor depositing nothing.

Two semantics to read back before building, since neither is in the spec:

- A product with a set bit below 2^-1074 (`Wide` refuses it as `OutOfWindow::OffGrid`). For a tier
  whose window never reaches below limb 0, those bits are below the window like any other dropped
  bits, so truncating them silently is consistent with the type; refusing them is consistent with
  `Wide`. The bound `n · 2^(−64(K−1)) · max|x|` holds either way.
- A product at or past 2^1037 (`AboveCeiling` on `Wide`): the top bin would sit at limb 33 or
  beyond, and `into_wide` cannot place its carry, so this one must be refused by name, which makes
  the method fallible as `Wide`'s is.

Tests `binned_product_*`: the deposit equals `Wide::deposit_product` truncated below the window on
random factor pairs and on the adversaries (a product straddling the window's bottom, a subnormal
times a subnormal, factors whose product crosses three limbs, specials and zeros), bit-identical
under shuffles, and the refusal leaves the state untouched.

## log
- 2026-10-12T15:10Z created
- 2026-10-12T15:34Z handoff by claude (687d89e4-ae5c-4805-bf5e-f65f0e4a2b1a)
