---
id: fl-r0s86hg
title: "Clear RUSTSEC-2026-0385, -0391, -0398 and -0403 from the dbsp closure: update ctutils and actix-http, and ignore the zstd ids dbsp's own zstd ^0.12 carries, by id with dbsp named"
status: done
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
- 2026-11-02T15:21Z open→doing — claimed by claude (c90f1eae-8c6b-41ec-aa59-9d0990f41bc2)
- 2026-11-02T15:50Z doing→done — verify exit 0 @ 0ea6562+1

## comments
- 2026-11-02T15:25Z [claude (c90f1eae-8c6b-41ec-aa59-9d0990f41bc2)] 2026-11-02 owner, in session, on the ignore route: "yes but we need something to auto-trigger to remove the exception when dbsp updates zstd". Built as cargo-deny's own lint: both gates run `check -D advisory-not-detected` (scripts/regression.sh, and `command-arguments` on both cargo-deny-action steps in ci.yml), so an ignore whose crate has moved on is an error naming the row. Proven on a deliberate break: a scratch copy of deny-dbsp.toml with the cleared RUSTSEC-2026-0382 planted exits 1 with `error[advisory-not-detected]` under the flag and `advisories ok` without it.
  
  The flag then corrected the task's own plan on the first run: of the three zstd ids, only RUSTSEC-2026-0403 matches dbsp's zstd-safe 6.0.6; -0391 and -0398 reached the graph through actix-http's zstd 0.13.3 / zstd-safe 7.3.0 alone, which `cargo update -p actix-http` (3.18.13 → 3.20.0, zstd ^0.14) removed, so those two rows were stale the moment they were written and the gate said so. deny-dbsp.toml carries one new row, -0403, dbsp named. `cargo update -p ctutils` took 0.4.3 → 0.4.7 for -0385. Both deny configs are green locally with the flag; `cargo +1.74.0 build --locked` passes on the new lock with and without default features. CLAUDE.md § Standing decisions carries the ruling in the dbsp bullet.
- 2026-11-02T15:50Z [claude (c90f1eae-8c6b-41ec-aa59-9d0990f41bc2)] 2026-11-02 done on main@682db04: ci run 41472015205's sibling 41472015430 has the dbsp job (125367774501, null → null) green with its cargo-deny step on deny-dbsp.toml and -D advisory-not-detected, the cargo-deny job green on deny.toml with the same flag, the MSRV job green on the new lock, and the gate job green running regression.sh's two deny calls. miri is still in flight (interpretation, not affected by this change). The push gate ran the same two checks locally before the push.
- 2026-11-02T15:50Z [claude (c90f1eae-8c6b-41ec-aa59-9d0990f41bc2)] Correction to the line above: the dbsp job 125367774501 ran 2026-11-02T15:29:05Z → 2026-11-02T15:49:25Z (the earlier comment's "null → null" was a wrong field name in the query, not a missing job).
