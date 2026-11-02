---
id: fl-cr71vam
title: Arm cargo-semver-checks in CI from the first release on
category: crate/release
labels: [reproducible-floats]
seq: 190
discovered-from: fl-k3g1511
needs: [fl-7en8z60]
docs:
  - docs/DESIGN.md#release-sp-release
verify: contains .github/workflows/ci.yml semver-checks
status: open
created: 2026-10-20T15:04Z
parent: fl-m3cymxv
---

DESIGN.md § Release: "`cargo-semver-checks` is armed from then on". Two halves. release-plz runs cargo-semver-checks itself when it decides a bump (`semver_check = true`, fl-ps58fd0), so the version a release PR proposes already reflects a detected break. The gate half is a `ci.yml` job on pull requests and pushes that runs `cargo semver-checks check-release` against the last released baseline (`obi1kenobi/cargo-semver-checks-action`, hash-pinned), on the features docs.rs builds (`package.metadata.docs.rs`), so a breaking change fails the gate rather than only raising the next version. Before the first release there is no baseline and the job would be noise; it lands with or right after the first cut (fl-enkdnvk).

## log
- 2026-10-20T15:04Z created
