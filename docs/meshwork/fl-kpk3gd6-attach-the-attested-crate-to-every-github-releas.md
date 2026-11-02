---
id: fl-kpk3gd6
title: "Attach the attested .crate to every GitHub Release: on release published, cargo package --locked, sha256, actions/attest-build-provenance and upload; README's install section says gh attestation verify"
category: crate/release
labels: [reproducible-floats]
seq: 17
parent: fl-m3cymxv
needs: [fl-ps58fd0]
docs:
  - README.md#getting-it
  - docs/DESIGN.md#release-sp-release
verify: "all(contains .github/workflows/release-plz.yml attest-build-provenance, contains README.md /gh attestation verify/)"
status: open
created: 2026-11-02T15:33Z
---
release.yml today packages, attests and uploads only inside the crates.io publish job. Split: a
job on `release: types: [published]` that checks out the release's tag, `cargo package --locked`,
`sha256sum`, `actions/attest-build-provenance` (already pinned in the file) on the `.crate`, and
`gh release upload` of the `.crate`, the `.sha256` and nothing else; the publish job stays on
`workflow_dispatch` and reuses the same packaging. Attestation needs the repo public (the action
refuses on a private free-plan repo), so this is proven after the flip. README's install section:
the release page, the `.crate` and `gh attestation verify float-abel-X.Y.Z.crate --repo
jbrjake/float-abel`, and the git-tag form of the dependency for cargo; "For now, grab a release tag
from here" becomes a sentence that is true.

## log
- 2026-11-02T15:33Z created
