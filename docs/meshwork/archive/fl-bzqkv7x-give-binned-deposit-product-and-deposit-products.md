---
id: fl-bzqkv7x
title: "Give Binned deposit_product and deposit_products — the 106-bit product sliced into the bins with the product's sign, refused by name past the ceiling"
status: done
category: crate/types
labels: [reproducible-floats, rf-tail]
relates: [fl-v7t6xhz, fl-b41y9kt]
verify: run cargo test package=float-abel binned_product_
docs:
  - docs/DESIGN.md#operations-and-conversions-sp-ops
seq: 115
created: 2026-10-12T15:10Z
covers:
  - ref: docs/DESIGN.md#sp-binned
    sha: 960e2e32f86d78705d8d12a15dea4450a64664b783ad3fce4b20e57e7fd1c6de
  - ref: docs/DESIGN.md#sp-ops
    sha: a706c03b7e0d89d927cbf913c6da1efc470bd10517ce36f7d534f0b6ae85f3b5
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
- 2026-10-12T19:37Z open→doing — claimed by claude (698bf172-c634-41d0-9a55-81a5fdd2e3db)
- 2026-10-12T19:54Z cover docs/DESIGN.md#sp-binned @960e2e32f86d
- 2026-10-12T19:54Z cover docs/DESIGN.md#sp-ops @a706c03b7e0d
- 2026-10-12T19:56Z doing→done — verify exit 0 @ 42c990c+1

## comments
- 2026-10-12T19:43Z [claude (698bf172-c634-41d0-9a55-81a5fdd2e3db)] Owner ruling, 2026-10-12, this transcript, through one AskUserQuestion on the sub-grid semantics: "Truncate below the grid (Recommended)". The option as put: "Consistent with Binned's definition: the window never reaches below limb 0, so the bits are dropped like any other below the window, and a fold stays order-independent. DESIGN.md § Binned's bound gains a floor: n · max(2^(−64(K−1)) · max|a·b|, 2^-1074). Products of two small normals, such as 2^-512 × 2^-512 with full mantissas, are accepted." The alternative put and not taken: refuse as OffGrid like Wide.
  
  Correction to the handoff's "The bound holds either way": it does not under truncation. A product wholly below 2^-1074 (5e-324 × 0.5 = 2^-1075) is dropped whole, so the loss is the product itself, above n · 2^(−64(K−1)) · |ab|. The bound for products carries the floor above; the floor is below any f64's resolution.
  
  Read back to the owner in the same message, unobjected: the ceiling is Wide's 2^1037, refused as AboveCeiling with the state untouched, because the top bin at limb 32 with a carry capped at 2^63 is what keeps into_wide inside Wide's width; specials by Wide's sign rule and a zero factor deposits nothing; deposit_products checks every pair first so a refusal deposits nothing, and a length mismatch panics by name; the Binned product joins the golden transcript and the claim is re-blessed here.
- 2026-10-12T19:56Z [claude (698bf172-c634-41d0-9a55-81a5fdd2e3db)] Closed at 42c990c. Built: `deposit_product` and `deposit_products` on `Binned<K>` in src/binned.rs, with `Product`, `product` and `finite_product` as the classifier the bulk entry runs over every pair before it deposits any; `deposit_finite` now takes a `u128` and slices three words through `limbs::set_shifted`, so the double path and the product path are one slicer, and that three-word slicer is the scalar reference kernel the simd task should mirror. `Wide`'s `CEILING_BITS` is `pub(crate)` for it.
  
  Tests: tests/binned_product.rs is a new topic (`mod binned_product;` in tests/main.rs; `shuffle` and `assert_same_bits` in tests/binned.rs made `pub(crate)`, `shuffle` generic). Nine `binned_product_*` tests, with a bit-by-bit oracle built from the factors' bits. Each was watched failing on a deliberate break: keeping the sub-grid bits failed three (sub-grid, bound, oracle), dropping the third slice failed three (three-limb, bound, oracle).
  
  Golden: `absorb` adds the Binned product of the first and last inputs and the multiset's dot product with its own reverse through `deposit_products`. BYTES 60,470,210 (ELEMENTS unchanged), 4,474 Binned product refusals in the stream, hash re-blessed on aarch64 macOS: 5e21d2e5857423dddfdacba60a71a4e94ad3127b7af0e8ad866e66658a629322.
  
  Spec: sp-binned and sp-ops are repinned to this task. `spec audit` lists the done tasks that covered them (fl-v7t6xhz on sp-binned; fl-atkbp6q, fl-te1748m, fl-zs79ct9 on sp-ops) as re-open candidates; their work is unchanged, the clauses only grew a Binned sentence each, so none is reopened.
  
  Gates at close: smoke through the pre-commit hook, 152 tests; `cargo clippy --workspace --all-targets -- -D warnings` and `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` clean by hand. Not run at close: `./scripts/regression.sh` and the wasm32 golden leg (wasmtime rebuilding); both go in the next handoff.
