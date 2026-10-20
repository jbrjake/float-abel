---
id: fl-cr71vam
title: "Arm cargo-semver-checks in CI from 0.1.0 on"
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
---
DESIGN.md § Release: "`cargo-semver-checks` is armed from then on". A `ci.yml` job on pull
requests and pushes to main that runs `cargo semver-checks check-release` against the published
baseline (`obi1kenobi/cargo-semver-checks-action`, hash-pinned), on the features docs.rs builds
(`package.metadata.docs.rs`), so a breaking change against the last published version fails the
gate rather than the next release. Before 0.1.0 there is no baseline and the job would be noise.

## log
- 2026-10-20T15:04Z created
