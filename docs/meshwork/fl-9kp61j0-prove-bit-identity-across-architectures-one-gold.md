---
id: fl-9kp61j0
title: "Prove bit-identity across architectures — one golden hash on x86_64 Linux and Windows, aarch64 macOS and Linux, wasm32, riscv64gc and s390x"
category: crate/hardening
labels: [reproducible-floats, rf-tail]
docs: [docs/DESIGN.md#bit-identical-across-architectures-sp-architecture]
needs: [fl-z91b7pq, fl-a2wrx7w]
seq: 110
verify: "all(exists .github/workflows/golden.yml, exists tests/data/golden.sha256)"
status: open
created: 2026-10-11T15:02Z
covers:
  - ref: docs/DESIGN.md#sp-architecture
    sha: 1ef714f4343a4567a0e79b71a746c6342080f8c460c45867c2de4191bd2a0194
handoff: |
  The tree is at d98adf7. Green this session: `scripts/smoke.sh` through
  the pre-commit hook (143
  tests), `cargo clippy --all-targets -- -D warnings`, `RUSTDOCFLAGS="-D
  warnings" cargo doc
  --no-deps`, a bare `cargo test` (31 lib, 111 + 1 ignored in
  `tests/main.rs`, the doctest) and
  `cargo test -p float-abel-noalloc` (6). Not re-run this session: the
  release headroom leg, the
  MSRV build and cargo-deny, i.e. `scripts/regression.sh` as a whole; the
  pre-push hook runs it.
  `CARGO_TARGET_DIR` is set outside ~/Documents by the repo's
  `.claude/settings.local.json`.
  
  Since the last handoff, "Build Binned<K>" closed: `src/binned.rs`,
  exported from `src/lib.rs`, with
  `tests/binned.rs` as `mod binned;` of `tests/main.rs` (14 `binned_*`
  tests, the trybuild one in
  `tests/ui.rs`). The shape differs from the design's sketch and the
  task's comment says why: each
  bin is a `u64` limb plus an `i64` carry that never crosses into the bin
  above, the slices are 64
  bits on Wide's literal limb grid, the bound is
  `n·2^(−64(K−1))·max|x|`, and the word is 16·K + 32
  bytes (80 at K=3). That byte count and the deferral of the DBSP
  compile-fail case to "Ship the
  dbsp feature" are the two owner questions on the record; neither is
  ratified. `deposit_product`
  on Binned is filed as "Give Binned deposit_product and deposit_products"
  (fl-bzqkv7x, seq 115).
  `differential-dataflow = "0.25"` is now also a dev-dependency, for
  trybuild.
  
  This task, per DESIGN.md § Bit-identical across architectures: one
  SHA-256 over `to_bytes` and
  `to_f64().to_bits()` of every vector in the rounding set
  (`tests/data/rounding_vectors.json`,
  loaded at `tests/rounding.rs`, which enters a vector through
  `Wide::from_parts` and
  `Fixed::from_limbs` and the conversions) and the conformance set
  (`tests/data/bitrep/vectors.json`,
  loaded at `tests/wide.rs`), plus 10⁴ seeded random multisets
  (`tests/support.rs` has the
  splitmix `Rng` with `finite`, `on_grid`, `any`), asserted equal on
  x86_64 Linux and Windows,
  aarch64 macOS and Linux, wasm32 under wasmtime, and riscv64gc and s390x
  under QEMU via `cross`.
  The verify wants `.github/workflows/golden.yml` and
  `tests/data/golden.sha256`. What the tree
  gives you and what to decide first:
  
  - Where the hashing code lives. `tests/main.rs` is the one integration
  target, but it links
  trybuild, timely, differential-dataflow, proptest and postcard, and its
  differential tests spawn
  worker threads and processes that wasm32 cannot run. `tests/noalloc/` is
  the precedent for a
  separate workspace member with its own manifest that reaches a build the
  main package cannot:
  a `tests/golden/` member depending on the lib alone plus a `sha2`
  dev-dependency (the crate's
  zero-required-dependencies posture is about the lib, and the noalloc
  member already lists only
  the lib) would cross-compile in seconds and keep the hash's inputs to
  the lib's own bytes. A
  `golden_` test there asserts the computed hex against the committed
  file, and a `--nocapture`
  print, or a tiny `--ignored` writer, is how the file is first produced.
  Read that back before
  building: the standing override says the one target is `tests/main.rs`
  "because task verifies
  name target `main`", and this task's verify names files, not a target,
  so a second member does
  not collide with it, but the owner has not ruled on a second test member
  for this purpose.
  - What enters the hash. `Exact`, `Wide` and `Fixed` have `to_bytes`.
  `Binned` has no wire form by
  design (DESIGN.md § Binned, last sentence): hash
  `into_wide().to_bytes()` of its folds, and its
  `index()`, `limbs()` and `carries()`, so the batch tier's bits are
  claimed too, and say so in
  the design's § architecture sentence when you do. Specials and
  subnormals must be in the random
  multisets (`Rng::any` gives both), and the multisets should run through
  every tier, with
  `Fixed256` taking the `on_grid(-44, 95)` subset it holds.
  - The workflow. `ci.yml` is the model for toolchain install and the
  cache key; the golden
  workflow is its own file so that a target's runner outage does not block
  the functional gate.
  Native jobs: `ubuntu-latest`, `windows-latest`, `macos-latest` (Apple
  silicon, aarch64),
  `ubuntu-24.04-arm` for aarch64 Linux. wasm32: `rustup target add
  wasm32-wasip1`, install
  wasmtime, and `CARGO_TARGET_WASM32_WASIP1_RUNNER=wasmtime cargo test
  --target wasm32-wasip1`.
  riscv64gc and s390x: `cross test --target riscv64gc-unknown-linux-gnu`
  and
  `s390x-unknown-linux-gnu`, which needs Docker on the runner and a pinned
  `cross` release. s390x
  is big-endian and is what catches a byte-order slip in `from_bytes`; the
  limb reads already go
  through `to_le_bytes`/`from_le_bytes`, never a cast, so expect it to
  pass, and if it does not
  the bug is real.
  - The committed hash is the claim. Compute it locally on this machine
  (aarch64 macOS) first,
  commit it, and let the matrix assert it; a mismatch on any target is a
  finding, never a reason
  to re-bless.
  
  After this: fl-b41y9kt (simd, now with Binned's bins as a bulk surface),
  fl-06sggdc (dbsp, with the
  DBSP compile-fail case owed to it), fl-ajqrs9p, fl-e4jjqf1, fl-c3ahdwc,
  fl-fm78m8d. The portfolio
  ask fl-cfx001b is still out.
---
Per DESIGN.md § Bit-identical across architectures. The hash is a SHA-256 over `to_bytes` and
`to_f64().to_bits()` for every vector in the conformance and rounding sets, plus 10⁴ seeded random
multisets. riscv64gc and s390x run under QEMU via `cross`, and wasm32 under wasmtime. The committed
hash is the claim, and the workflow asserts it on every push.

## log
- 2026-10-11T15:02Z created
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-architecture @1ef714f4343a
- 2026-10-12T15:11Z handoff by claude (745519d7-d30a-4336-9fa7-1462279793db)
