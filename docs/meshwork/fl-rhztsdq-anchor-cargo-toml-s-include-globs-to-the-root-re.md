---
id: fl-rhztsdq
title: "Anchor Cargo.toml's include globs to the root: README.md and LICENSE-MIT match at any depth, so the .crate carries tests/data/bitrep/README.md and tests/data/bitrep/LICENSE-MIT"
status: open
category: crate/release
labels: [reproducible-floats]
parent: fl-m3cymxv
verify: "all(contains Cargo.toml /\"\\/README.md\"/, contains Cargo.toml /\"\\/LICENSE-MIT\"/)"
docs:
  - docs/DESIGN.md#release-sp-release
seq: 30
created: 2026-11-02T20:49Z
---

Found 2026-11-02 reading `cargo package --list --locked` for the owner (49 files). `include = ["src/**", "Cargo.toml", "README.md", "FORMAT.md", "LICENSE-MIT", "LICENSE-APACHE"]` uses gitignore-style globs, where a bare name matches at any depth, so the bitrep conformance fixtures' README.md and LICENSE-MIT under tests/data/bitrep/ ride into the .crate, two files the published crate has no use for. The fix is a leading slash on each file pattern (`/README.md`, `/LICENSE-MIT`, `/LICENSE-APACHE`, `/FORMAT.md`, `/Cargo.toml`); `cargo package --list --locked` then shows 47 files and no tests/ path. Harmless to consumers, so it lands before or after the first release as the owner likes.

## log
- 2026-11-02T20:49Z created
