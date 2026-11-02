---
id: fl-r0s86hg
title: "Clear RUSTSEC-2026-0385, -0391, -0398 and -0403 from the dbsp closure: update ctutils and actix-http, and ignore the zstd ids dbsp's own zstd ^0.12 carries, by id with dbsp named"
status: open
category: crate/hardening
labels: [reproducible-floats]
relates: [fl-4fcbtkx]
verify: "all(lacks Cargo.lock /name = \"ctutils\"\\s+version = \"0\\.4\\.3\"/, lacks Cargo.lock /name = \"zstd\"\\s+version = \"0\\.13\\.3\"/, contains deny-dbsp.toml RUSTSEC-2026-0403)"
docs:
  - CLAUDE.md#standing-decisions
  - docs/DESIGN.md#hardening-sp-hardening
seq: 10
created: 2026-11-02T15:05Z
parent: fl-b9qtzty
---

Four advisories published after f24b051 fail `cargo deny --config deny-dbsp.toml check` on main's
lockfile: ci's dbsp job went red on them on PR #4 (run 41005514396, whose Cargo.lock is main's), and
the same check fails on this checkout, so the push gate (scripts/regression.sh runs both deny
configs) refuses every push until they clear. deny.toml's closure is unaffected.

| id | crate in the lock | dragged by | route |
|---|---|---|---|
| RUSTSEC-2026-0385 | ctutils 0.4.3 | digest 0.11 ← hmac/md-5/sha1/sha2 ← zip 9, object_store, actix-http, lzma-rust2 | `cargo update -p ctutils` → 0.4.7 |
| RUSTSEC-2026-0391, -0398, -0403 | zstd 0.13.3, zstd-safe 7.3.0 | actix-http 3.18.13 ← actix-web ← feldera-types ← dbsp | `cargo update -p actix-http` → 3.20.0 (zstd ^0.14; drops 0.13.3 and 7.3.0, moves brotli 8→9) |
| RUSTSEC-2026-0391, -0398, -0403 | zstd 0.12.4, zstd-safe 6.0.6 | dbsp itself: `zstd ^0.12.0` in every release through 0.375.0 (2026-11-02) | no in-range fix; ignore by id in deny-dbsp.toml with dbsp named, the rkyv/h2 form |

The two updates are mechanical. The ignore is the owner's call (CLAUDE.md § Standing decisions: an
advisory in dbsp's closure is ignored by id with the dragging crate named, never wholesale), and a
dbsp bump does not help: the newest dbsp still requires zstd ^0.12, so the caret floor stays at
0.367 and the probe is not re-run. The three zstd advisories are about decompressing untrusted zstd
streams (a reused context leaking data, uninitialised memory on a failed legacy frame,
`Seekable::decompress` over-setting a Vec's length); this crate opens no zstd stream, and dbsp's
storage layer is the only path to the crate. When dbsp moves to zstd 0.14, cargo-deny's
`advisory-not-detected` warning on the ignore is the signal to remove it.

Done: `cargo update -p ctutils -p actix-http`, the three ids in deny-dbsp.toml's `ignore` with the
reason, both deny configs green locally, the MSRV job's `cargo build --locked` on 1.74 still green
(the lock is what it parses), then a push through the gate and `gh run view` on the ci dbsp job green.

## log
- 2026-11-02T15:05Z created
