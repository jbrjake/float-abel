---
id: fl-fm78m8d
title: "Prepare the 0.1.0 release — CI publish job with --locked, a signed tag and provenance, cargo-semver-checks armed, and the CHANGELOG entry"
category: crate/release
labels: [reproducible-floats, rf-tail]
docs: [docs/DESIGN.md#release-sp-release]
needs: [fl-z91b7pq, fl-6shtzpj, fl-v6mh9cv, fl-06sggdc, fl-9kp61j0, fl-ajqrs9p, fl-c3ahdwc, fl-v7t6xhz, fl-b41y9kt, fl-jmjbjk5, fl-k3g1511]
seq: 170
verify: "all(exists CHANGELOG.md, contains CHANGELOG.md /^## 0\\.1\\.0/)"
status: open
created: 2026-10-11T15:02Z
covers:
  - ref: docs/DESIGN.md#sp-release
    sha: 3478ee626df5140b33eec38f959ee91cf3e5745abfb611a5e9d4941b60eec782
handoff: |
  The publish ruling is in (owner, 2026-10-20: "make it a ci publish on my
  manual trigger") and
  built: `.github/workflows/release.yml` at 9f651e2 publishes on
  `workflow_dispatch` with inputs `tag`
  and `auth` (`token` for the first publish, `trusted` after), packages
  with `--locked`, attests the
  `.crate` and attaches it to a GitHub release; `cargo publish --dry-run
  --locked` is green here. What
  this umbrella still waits on, each its own task: the tag-signature step,
  fl-x2k7fbb, held on the
  owner naming a key (nothing in the checkout signs today); the first
  publish itself, fl-7en8z60, the
  owner's act, blocked on "i don't want to publish yet" and the ship
  list's AVX-512 kernel
  (fl-qv0vh39); `cargo-semver-checks`, fl-cr71vam, behind the first
  publish; and this task's own
  verify, a `## 0.1.0` entry in CHANGELOG.md, which does not exist yet.
  dbsp's floor is 0.367 since
  db4e5d9 (zip 6's advisories), so the dbsp job's deny check is green
  again.
---
Per DESIGN.md § Release. This task makes 0.1.0 ready to ship: the CI publish job, the signed-tag
flow, provenance, `cargo-semver-checks` armed from here, and the `## 0.1.0` CHANGELOG entry. Running
`cargo publish`, and any announcement, are the owner's acts.

## log
- 2026-10-11T15:02Z created
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-release @56381872cf6a
- 2026-10-14T21:28Z handoff by claude (4e0e29d7-b85d-4a99-93e3-a9a5c00bb168)
- 2026-10-16T19:34Z handoff by claude (03cbad3b-1e00-455d-9d44-3b0d9decee6e)
- 2026-10-20T15:18Z cover docs/DESIGN.md#sp-release @3478ee626df5
- 2026-10-20T15:22Z cover docs/DESIGN.md#sp-release @3478ee626df5
- 2026-10-20T15:29Z handoff by claude (4a8ded2c-6eb4-4638-a7f1-ee6da67e86ba)
- 2026-10-20T15:29Z handoff by claude (4a8ded2c-6eb4-4638-a7f1-ee6da67e86ba)
