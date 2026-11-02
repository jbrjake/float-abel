---
id: fl-cr71vam
title: Arm cargo-semver-checks in CI from the first release on
category: crate/release
labels: [reproducible-floats]
seq: 190
discovered-from: fl-k3g1511
docs:
  - docs/DESIGN.md#release-sp-release
verify: contains .github/workflows/ci.yml semver-checks
status: open
created: 2026-10-20T15:04Z
parent: fl-m3cymxv
needs: [fl-enkdnvk]
---

DESIGN.md § Release: cargo-semver-checks runs on every release PR. The release-plz half is done: release-plz.toml has semver_check = true and release-plz.yml installs cargo-semver-checks 0.51.0 beside it, so the version a release PR proposes already reflects a detected break. The gate half is a ci.yml job on pull requests that runs cargo semver-checks check-release against the last release tag as the baseline (--baseline-rev, the tag release-plz made; crates.io is outside the release, so never the registry), on the features docs.rs builds (package.metadata.docs.rs), so a breaking change fails the gate rather than only raising the next version. Before the first release there is no tag to baseline on and the job would be noise; it lands right after the first cut (fl-enkdnvk). Hash-pin the action (obi1kenobi/cargo-semver-checks-action) or install the tool through taiki-e/install-action as release-plz.yml does.

## log
- 2026-10-20T15:04Z created
