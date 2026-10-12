---
id: fl-9kp61j0
title: "Prove bit-identity across architectures — one golden hash on x86_64 Linux and Windows, aarch64 macOS and Linux, wasm32, riscv64gc and s390x"
category: crate/hardening
labels: [reproducible-floats, rf-tail]
docs: [docs/DESIGN.md#bit-identical-across-architectures-sp-architecture]
needs: [fl-z91b7pq, fl-a2wrx7w]
seq: 110
verify: "all(exists .github/workflows/golden.yml, exists tests/data/golden.sha256)"
status: done
created: 2026-10-11T15:02Z
covers:
  - ref: docs/DESIGN.md#sp-architecture
    sha: ff2803a15c2d4731263c1a8b6586ab4f9ff060d1851f251a053e9980c7c79638
---
Per DESIGN.md § Bit-identical across architectures. The hash is a SHA-256 over `to_bytes` and
`to_f64().to_bits()` for every vector in the conformance and rounding sets, plus 10⁴ seeded random
multisets. riscv64gc and s390x run under QEMU via `cross`, and wasm32 under wasmtime. The committed
hash is the claim, and the workflow asserts it on every push.

## log
- 2026-10-11T15:02Z created
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-architecture @1ef714f4343a
- 2026-10-12T15:11Z handoff by claude (745519d7-d30a-4336-9fa7-1462279793db)
- 2026-10-12T15:18Z open→doing — claimed by claude (687d89e4-ae5c-4805-bf5e-f65f0e4a2b1a)
- 2026-10-12T15:32Z cover docs/DESIGN.md#sp-architecture @ff2803a15c2d
- 2026-10-12T15:33Z doing→done — verify exit 0 @ a6a17f3+2

## comments
- 2026-10-12T15:33Z [claude (687d89e4-ae5c-4805-bf5e-f65f0e4a2b1a)] Closed at a6a17f3. Verified this session: `cargo test -p float-abel-golden` green on aarch64 macOS (2 tests, 2.2 s) and on wasm32-wasip1 under wasmtime 49.0.2 (the same hash, 2.5 s); `./scripts/regression.sh` green end to end, cargo-deny included, after fixing the noalloc clippy error (fl-w9mk4jc). Not verified: the golden workflow itself has not run, so the five remaining targets (x86_64 Linux and Windows, aarch64 Linux, riscv64gc and s390x) are asserted only once the owner pushes. The QEMU legs pin cross v0.2.5, the current release, from 2023; if its images prove too old for the stable toolchain, the fix is a pinned cross `main` image, never a re-bless. Docker was down here, so cross was not exercised locally.
  
  Two owner questions, neither ratified: (1) a second test workspace member, tests/golden/, on the noalloc precedent, while CLAUDE.md override 3 says the one target is tests/main.rs because verifies name target main (this task's verify names files, so they do not collide); (2) sha2 0.10 and serde_json as dependencies of that member, outside the lib's zero-dependency posture.
  
  What enters the hash goes beyond the design's earlier sentence (to_bytes and to_f64), and DESIGN.md § Bit-identical across architectures now says so: try_to_f64, to_f32, div_f64, every refusal by name, Binned's index, limbs and carries with its into_wide, and the 106-bit product on every tier. Found on the way: Exact::checked_deposit_product on the zero never refuses (a two-double product fits 127 bits), so the transcript tags its result but keeps no refusal count for it.
  
  Follow-up for fl-b41y9kt (simd): the golden member depends on the lib with default features, so no simd kernel enters the hash yet; turn `simd` on in tests/golden/Cargo.toml once kernels exist, and the hash then covers the dispatched paths on x86_64 and aarch64 as DESIGN.md § Speed claims.
