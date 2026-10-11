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
---
Per DESIGN.md § Bit-identical across architectures. The hash is a SHA-256 over `to_bytes` and
`to_f64().to_bits()` for every vector in the conformance and rounding sets, plus 10⁴ seeded random
multisets. riscv64gc and s390x run under QEMU via `cross`, and wasm32 under wasmtime. The committed
hash is the claim, and the workflow asserts it on every push.

## log
- 2026-10-11T15:02Z created
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-architecture @1ef714f4343a
