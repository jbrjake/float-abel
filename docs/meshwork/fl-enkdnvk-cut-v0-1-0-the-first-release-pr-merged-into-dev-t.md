---
id: fl-enkdnvk
title: "Cut the first release: the release PR merged into dev, the dev→main PR merged, the tag and GitHub Release created at the version the conventional-commit log yields, the attested .crate attached and verified with gh attestation verify"
category: crate/release
labels: [reproducible-floats]
seq: 19
parent: fl-m3cymxv
needs: [fl-kpk3gd6, fl-bct35bq, fl-4fcbtkx]
relates: [fl-7en8z60, fl-fm78m8d]
docs:
  - README.md#getting-it
verify: contains CHANGELOG.md /^## \[?[0-9]+\.[0-9]+\.[0-9]+/
status: open
created: 2026-11-02T15:33Z
---

The owner's merges, twice, and the session's watch: release-plz's PR into `dev` bumps Cargo.toml
from 0.0.0 to whatever the conventional-commit log since the start yields under its rules for a 0.x
crate (owner, 2026-11-02: "IT'S NOT V0.1.0 IT'S WHAT THE CONVENTIONAL COMMITS SAY IT IS"); no
version is typed anywhere by hand. The owner merges it; the session opens `dev`→`main`, the owner
merges it; `release-plz release` tags `v<that version>` and creates the release; the attest job
uploads the `.crate`; `gh attestation verify float-abel-<version>.crate --repo jbrjake/float-abel`
passes from a clean machine. crates.io (fl-7en8z60) stays the owner's separate dispatch unless the
ruling moved it in. Done is the verify (a released version heading in CHANGELOG.md) plus the
release URL and the verify command's output in a comment here.

## log
- 2026-11-02T15:33Z created
