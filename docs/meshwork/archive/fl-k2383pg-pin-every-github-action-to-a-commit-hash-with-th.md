---
id: fl-k2383pg
title: "Pin every GitHub Action to a commit hash, with the tag in a comment, and gate it"
status: done
category: crate/hardening
labels: [reproducible-floats]
verify: "all(lacks .github/workflows/ci.yml /uses: [^ ]+@v[0-9]/, lacks .github/workflows/golden.yml /uses: [^ ]+@v[0-9]/, lacks .github/workflows/fuzz.yml /uses: [^ ]+@v[0-9]/, exists scripts/check-actions-pinned.sh)"
docs:
  - .github/workflows/ci.yml
seq: 135
created: 2026-10-13T13:43Z
---

Owner, in session on 2026-10-13: "make sure your github actions use git hashes to reference
dependencies not git labels that can be manipulated" and "security is absolutely critical here".

Every `uses:` in .github/workflows/*.yml names a full commit hash resolved from the GitHub API,
with the tag it stood for in a trailing comment, in ci.yml, golden.yml and fuzz.yml.
scripts/check-actions-pinned.sh fails on any `uses:` that names a tag or a branch, and both gates
run it, so the rule holds for every workflow that comes later. `cargo install cargo-fuzz` names
its version. The two release tarballs the golden workflow downloads (wasmtime, cross) are
version-pinned by URL and not hash-verified: that gap is its own task.

## log
- 2026-10-13T13:43Z created
- 2026-10-13T13:43Z open→doing — claimed by claude (2dfe7fb0-aa27-4caa-abdc-1dd0125d6691)
- 2026-10-13T14:44Z doing→done — verify exit 0 @ 1b076e9+4

## comments
- 2026-10-13T14:44Z [claude (2dfe7fb0-aa27-4caa-abdc-1dd0125d6691)] Pinned in session on the owner's words. Each floating major was resolved through the GitHub
  API to the commit it pointed at on 2026-10-13, and the version tag on that commit is the trailing
  comment: actions/checkout@v4 → 11d5960a326750d5838078e36cf38b85af677262 (v4.4.0), actions/cache@v4
  → 0057852bfaa89a56745cba8c7296529d2fc39830 (v4.3.0), actions/upload-artifact@v4 →
  ea165f8d65b6e75b540449e92b4886f43607fa02 (v4.6.2), EmbarkStudios/cargo-deny-action@v2 →
  3c6349835b2b7b196a839186cb8b78e02f7b5f25 (v2.1.1). So the workflows run exactly what they ran
  before, now immovably. scripts/check-actions-pinned.sh fails on any `uses:` whose ref is not
  forty hex digits (a local `./` action carries none), watched failing on a probe file naming
  `@v4` and `@main` before it was trusted; smoke runs it when the diff touches a workflow and
  regression always. fuzz.yml's `cargo install cargo-fuzz` names 0.13.2. The release tarballs the
  golden workflow downloads are fl-cvpya2q.
