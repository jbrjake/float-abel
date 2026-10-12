---
id: fl-fm78m8d
title: "Prepare the 0.1.0 release — CI publish job with --locked, a signed tag and provenance, cargo-semver-checks armed, and the CHANGELOG entry"
category: crate/release
labels: [reproducible-floats, rf-tail]
docs: [docs/DESIGN.md#release-sp-release]
needs: [fl-z91b7pq, fl-6shtzpj, fl-v6mh9cv, fl-06sggdc, fl-9kp61j0, fl-ajqrs9p, fl-c3ahdwc, fl-v7t6xhz, fl-b41y9kt, fl-jmjbjk5]
seq: 170
verify: "all(exists CHANGELOG.md, contains CHANGELOG.md /^## 0\\.1\\.0/)"
status: open
created: 2026-10-11T15:02Z
covers:
  - ref: docs/DESIGN.md#sp-release
    sha: 56381872cf6a1b1ceef187cdda6df4028cbdee7e10fddf87590a202cc3ee6f62
---
Per DESIGN.md § Release. This task makes 0.1.0 ready to ship: the CI publish job, the signed-tag
flow, provenance, `cargo-semver-checks` armed from here, and the `## 0.1.0` CHANGELOG entry. Running
`cargo publish`, and any announcement, are the owner's acts.

## log
- 2026-10-11T15:02Z created
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-release @56381872cf6a
