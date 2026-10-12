---
id: fl-ceyy1h6
title: "Pin the trybuild snapshots to the toolchain that blessed them — tests/ui's .stderr files match rustc 1.99 and fail on 1.96.1, so the push gate is green only on one compiler"
status: open
category: crate/hardening
labels: [reproducible-floats]
relates: [fl-b41y9kt]
verify: contains tests/ui.rs /rustversion|rustc_version|skip/
docs:
  - CLAUDE.md#commands
seq: 165
created: 2026-10-12T21:09Z
---

Found 2026-10-12 running `./scripts/regression.sh` on jons-amd-mini (rustc 1.96.1): `cargo test -- --include-ignored` fails in the trybuild topic on tests/ui/fixed_to_bytes_wrong_length.rs, because the committed .stderr was blessed on rustc 1.99.0 and 1.96.1 words the const-assertion diagnostic differently. Every other test passes there. So the push gate is green only on the compiler that blessed the snapshots, and stable moving past 1.99 will break CI's gate job the same way.

Options: gate the ui topic on the rustc version the snapshots were blessed with (the `rustversion` crate, or `rustc --version` at test time) and skip loudly elsewhere; or pin the gate's toolchain in CI and say so in CLAUDE.md § Commands; or both. Whatever lands, the regression script on a machine with another stable must either pass or say why it skipped, never fail on wording.

## log
- 2026-10-12T21:09Z created
