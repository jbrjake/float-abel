---
id: fl-vp9vk3r
title: "Make prepare-release.sh move fuzz/Cargo.lock with Cargo.lock: the fuzz package records the path dependency's version, so the release commit leaves it stale and the next gate run dirties the tree"
status: open
category: crate/release
labels: [reproducible-floats]
relates: [fl-xq4fp0j]
verify: contains scripts/release-pr.sh fuzz/Cargo.lock
docs:
  - docs/DESIGN.md#release-sp-release
seq: 40
created: 2026-11-02T22:06Z
---

Found 2026-11-02 after the first release: the push gate's fuzz leg (cargo clippy/test --manifest-path fuzz/Cargo.toml in scripts/regression.sh) rewrote fuzz/Cargo.lock's float-abel version from 0.30.1 to 0.30.2, because scripts/prepare-release.sh updates Cargo.toml and Cargo.lock alone. The release commit should carry fuzz/Cargo.lock too (cargo update --manifest-path fuzz/Cargo.toml --package float-abel --offline, or cargo generate-lockfile there), and release-pr.sh's FILES and its self-test's file list should name it, so a release leaves a clean tree. Committed by hand this time as chore(fuzz).

## log
- 2026-11-02T22:06Z created

## comments
- 2026-11-02T22:06Z [claude (aa073407-90a0-459f-92ea-439fb58eb49c)] Correction to the body: fuzz/Cargo.lock's record was 0.0.0, never 0.30.1; the crate's own version was 0.0.0 until the release commit, so this is the first time the fuzz lockfile moved. Committed as chore(fuzz) on dev.
