---
id: fl-gbgbmxm
title: Bump rustls in the lockfile past RUSTSEC-2026-0346 to 0349 so deny-dbsp.toml's advisory check passes again
status: done
category: crate/release
labels: [reproducible-floats]
discovered-from: fl-s9hcne6
verify: lacks Cargo.lock /"0.23.45"/
docs:
  - deny-dbsp.toml
  - scripts/regression.sh
seq: 130
created: 2026-10-14T14:02Z
---

The push gate at 2a82388 fails in its last leg, `cargo deny --config deny-dbsp.toml check`: rustls 0.23.45, reached only through dbsp's storage (feldera-storage → object_store → reqwest → hyper-rustls/tokio-rustls/rustls-platform-verifier), carries four advisories published 2026-10-14 (RUSTSEC-2026-0346 Debug impls expose TLS secrets, 0347 ECH rejection buffering, 0348 brotli certificate decompression allocation, 0349 unauthenticated ServerHello downgrade), each fixed in >=0.23.46. deny.toml, the crate's own posture without dbsp, passes. The fix is `cargo update -p rustls` and a lockfile commit, then the gate's three checks on it: deny-dbsp with the sandbox off, `cargo +1.74.0 build --locked` (cargo 1.74 parses every locked manifest), and `cargo +1.99.0 test --features dbsp --test main dbsp_`. No ignore: these are vulnerabilities with a patch, not unmaintained notices.

## log
- 2026-10-14T14:02Z created
- 2026-10-14T14:06Z open→done — verify exit 0 @ 412ac2e+1
