---
id: fl-xrdrn1y
title: "Make the release move examples/short-table/Cargo.lock with the other lockfiles: it records float-abel's version through the path dependency, so a release that leaves it fails the push gate's --locked run"
category: crate/release
parent: fl-9p8yw9v
needs: [fl-v56pe0k, fl-vp9vk3r]
seq: 350
docs:
  - docs/DESIGN.md#release-sp-release
verify: contains scripts/release-pr.sh examples/short-table/Cargo.lock
status: done
created: 2026-11-06T14:19Z
---
The same gap fl-vp9vk3r closes for `fuzz/Cargo.lock`. Once that task's mechanism exists, extend it
rather than writing a second: `scripts/prepare-release.sh` updates this lockfile's float-abel
record (`cargo update --manifest-path examples/short-table/Cargo.toml --package float-abel
--offline`), and `scripts/release-pr.sh`'s file list and its self-test's list name the file, so a
release leaves a clean tree.

## log
- 2026-11-06T14:19Z created
- 2026-11-06T15:16Z open→doing — claimed by claude (14a6c070-cc69-4944-b798-52354e82e5d7)
- 2026-11-06T15:16Z doing→done — verify exit 0 @ 46e848b+4
