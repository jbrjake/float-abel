---
id: fl-ps58fd0
title: "Add release-plz: release-plz.toml with publish off, the release-PR job on dev and the release job on main, hash-pinned, with CHANGELOG.md's hand-written Unreleased folded into the generated form"
category: crate/release
labels: [reproducible-floats]
seq: 16
parent: fl-m3cymxv
needs: [fl-cnkjdzp, fl-k594v2g]
docs:
  - docs/DESIGN.md#release-sp-release
verify: "all(exists release-plz.toml, exists .github/workflows/release-plz.yml)"
status: open
created: 2026-11-02T15:33Z
---

`release-plz.toml`: `publish = false` (unless the ruling moves crates.io in), `git_release_enable =
true`, `git_tag_enable = true`, `semver_check = true`, changelog through git-cliff's conventional
preset with the repo's `type(scope): subject` form. `.github/workflows/release-plz.yml`: on push to
`dev`, `release-plz release-pr` (opens or updates the release PR against `dev`); on push to `main`,
`release-plz release` (tags `vX.Y.Z` and creates the GitHub Release when Cargo.toml's version has no
tag yet). The version is the tool's reading of the log, never typed: check how release-plz treats
a crate at 0.0.0 with no prior tag (its first-release rule) so the first PR carries the number the
commits earn and not 0.0.0 as-is. Both steps use MarcoIeni/release-plz-action pinned by commit hash
with the tag in the comment, and the pinning check covers it. The job needs `contents: write` and
`pull-requests: write`; a release PR opened by GITHUB_TOKEN does not trigger ci, so either a
fine-grained PAT or a GitHub App token (release-plz's docs) is the owner's choice, filed as a
comment here. CHANGELOG.md's hand-written Unreleased bullets become the body of the first generated
release entry rather than being lost, and fl-cr71vam closes on `semver_check = true`.

## log
- 2026-11-02T15:33Z created
