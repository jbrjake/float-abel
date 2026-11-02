---
id: fl-7en8z60
title: "Run the first crates.io publish by dispatching release.yml with the tag the release cut, then move the crate to Trusted Publishing"
category: crate/release
labels: [reproducible-floats]
seq: 180
discovered-from: fl-k3g1511
needs: [fl-5qbt792]
relates: [fl-x2k7fbb, fl-fm78m8d]
docs:
  - docs/DESIGN.md#release-sp-release
verify: contains docs/meshwork/fl-7en8z60-run-the-first-publish-0-1-0-by-dispatching-relea.md /2026-[0-9][0-9]-[0-9][0-9] owner published [0-9]+\.[0-9]+\.[0-9]+/
status: blocked
created: 2026-10-20T15:04Z
blocked-reason: "owner, 2026-10-16, in session: \"i don't want to publish yet\"; every step is the owner's act, and DESIGN.md § Release's ship list still lacks the AVX-512 kernel (fl-qv0vh39)"
parent: fl-m3cymxv
---

The owner's act, every step. Held on the owner's word of 2026-10-16, "i don't want to publish yet", and of 2026-11-02, "I WILL TELL YOU WHEN I WANT IT AND THEN YOU WILL ADD IT TO THE RELEASE PROCESS". The version and the tag are the release process's (fl-m3cymxv): the tag exists on main before this task starts, cut by cut-release.yml at the version the log counts to, and nothing here types a number. When the owner says: (1) on crates.io, a new API token scoped to publish-new and publish-update, crate float-abel, as the repository secret CARGO_REGISTRY_TOKEN; the first publish is the name reservation. (2) Dispatch release.yml with tag=<the release's tag>, auth=token; watch the run green and the crate on crates.io, its .crate byte-identical to the one the GitHub Release carries with its attestation. (3) On crates.io, Settings → Trusted Publishing → GitHub: owner jbrjake, repository float-abel, workflow release.yml, environment release; then delete the token on crates.io and the secret in the repository. (4) Fold the publish into the release process as the owner directs, and write the line this task's verify looks for, dated, with the version.

## log
- 2026-10-20T15:04Z created
- 2026-10-20T15:05Z open→blocked — owner, 2026-10-16, in session: "i don't want to publish yet"; every step is the owner's act, and DESIGN.md § Release's ship list still lacks the AVX-512 kernel (fl-qv0vh39)
