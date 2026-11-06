---
id: fl-v56pe0k
title: "Scaffold examples/short-table as a standalone package: the neighbours pinned with =, debug assertions in every profile, excluded from the workspace with its own lockfile, and a log header whose versions come from that lockfile"
category: crate/proof
parent: fl-9p8yw9v
seq: 230
docs:
  - docs/DESIGN.md#crate-posture-sp-posture
verify: 'all(exists examples/short-table/Cargo.lock, exists examples/short-table/OUTPUT.md, contains Cargo.toml examples/short-table, contains .gitignore examples/short-table/target, contains examples/short-table/OUTPUT.md /xsum 0\.1\.6/)'
status: done
created: 2026-11-06T14:19Z
---
Create `examples/short-table/` as a package of its own, the way `fuzz/` is one.

- `Cargo.toml`: `float-abel-short-table`, `publish = false`, edition 2021; float-abel by path
  `../..`; pinned with `=`: xsum 0.1.6, fsum 0.1.4, bitrep 0.5.1, accurate 0.4.1, num-rational
  0.4.2 (with num-bigint 0.4, num-traits 0.2), bigdecimal 0.4.11, fixed 1.31.0, impls 1.0.3. The
  root's `[profile.dev]`, plus `[profile.release] debug-assertions = true`, and `[lints]` copied
  from the workspace's, as `fuzz/Cargo.toml` does.
- Root `Cargo.toml`: `"examples/short-table"` in `[workspace] exclude`. Root `.gitignore`:
  `/examples/short-table/target`, because `/target` is anchored at the root.
- `Cargo.lock` committed.
- `src/main.rs` prints the header: what the file is, the command that writes it, each neighbour's
  version read from this package's `Cargo.lock` by `include_str!`, and float-abel as "this
  checkout", never its version, which every release bumps.
- `OUTPUT.md` is that run's stdout.

Standalone because fixed 1.31.0 is edition 2024 with rust-version 1.93, and the MSRV note in
`Cargo.toml` says cargo 1.74 parses locked manifests it never compiles; deny.toml would also start
judging crates the library never ships. The price: a verify runs `cargo` at the root and `-p` cannot reach an
excluded package, so these verifies read committed files and the gates do the running.

Debug assertions in release because xsum 0.1.6 derives `Debug`, its one window on its state, only
under `cfg(debug_assertions)`, and accurate 0.4.1's zero-state `sum()` guards a one-past-the-end
write with a `debug_assert!`. With them on, every profile prints the same bytes and the run never
executes undefined behaviour.

`cargo metadata --no-deps` must show no new example target on the root package: autodiscovery
reads `examples/*.rs` and `examples/*/main.rs`, and this main is in `src/`.

## log
- 2026-11-06T14:19Z created
- 2026-11-06T14:24Z open→doing — claimed by claude (14a6c070-cc69-4944-b798-52354e82e5d7)
- 2026-11-06T14:30Z doing→done — verify exit 0 @ 492e923+2
