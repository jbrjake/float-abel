---
id: fl-m3cymxv
title: "Build the release process: dev for development, main for releases, a dev→main PR cuts a release whose version and changelog come from the conventional-commit log, with the attested .crate on the GitHub Release"
category: crate/release
labels: [reproducible-floats]
seq: 12
parent: fl-b9qtzty
relates: [fl-fm78m8d]
docs:
  - docs/DESIGN.md#release-sp-release
  - README.md#getting-it
verify: contains CHANGELOG.md /^## \[?[0-9]+\.[0-9]+\.[0-9]+/
status: open
created: 2026-11-02T15:33Z
covers:
  - ref: docs/DESIGN.md#sp-release
    sha: d52b0c0dd2d2a5d6570b559791afc324d52c8051c4811f382dd12e56d5913d5b
---

Owner, 2026-11-02: "i don't understand why we don't have attested release builds available. how
are you not seeing this as a massive gap, especially considering i tell people to use releases in
the readme?" and "part of this needs to be moving to a dev branch for dev, main for releases, pr
from dev to main to cut one, versions automatically created from conventional commit log tracking,
like anyone would expect from a grown-up, mature project other people are going to use"; and on the
number: "IT'S NOT V0.1.0 IT'S WHAT THE CONVENTIONAL COMMITS SAY IT IS".

The gap as it stands: README.md § getting it says "For now, grab a release tag from here. crates.io
will come." The repo has no tag and no GitHub Release. release.yml creates a release only as the
last step of a crates.io publish on the owner's dispatch, which the owner has held ("i don't want to
publish yet", fl-7en8z60), so the README's install path is reachable by no task. Cargo.toml is at
0.0.0 and CHANGELOG.md has a hand-written Unreleased section.

The shape, for the ruling task's readback (the children build it once ratified):

- Branches. `dev` is where sessions push and PRs land; `main` holds only releases and accepts only
  a PR from `dev`. `dev` becomes the default branch so forks and PRs target it.
- Version and changelog from the log. release-plz (the Rust ecosystem's release-please: reads
  conventional commits since the last tag, bumps Cargo.toml, writes CHANGELOG.md with git-cliff,
  and runs cargo-semver-checks to decide the bump) opens and keeps current a release PR against
  `dev`. Merging it into `dev` fixes the version; the `dev`→`main` PR is then the release: on the
  merge, `release-plz release` tags it on `main` and creates the GitHub Release with the changelog
  as its notes. `main` never carries a commit `dev` lacks. No version is ever typed by hand.
- The artifact. On `release: published`, a job runs `cargo package --locked`, attests the `.crate`
  with actions/attest-build-provenance (Sigstore, bound to the workflow run; needs the repo public)
  and uploads the `.crate`, its sha256 and the attestation to the release. A reader verifies with
  `gh attestation verify float-abel-<version>.crate --repo jbrjake/float-abel`, and README says so.
- crates.io. Out of release-plz (`publish = false`); the owner's dispatch of release.yml stays the
  publish route per CLAUDE.md § Standing decisions, unless the owner moves it into the release.
- Commit discipline. The version is derived from commit messages, so the commit gate checks the
  Conventional Commits form and ci checks the PR title.

Verify: the first cut release writes a released version heading into CHANGELOG.md.

## log
- 2026-11-02T15:33Z created
- 2026-11-02T15:34Z cover docs/DESIGN.md#sp-release @3478ee626df5
- 2026-11-02T16:38Z cover docs/DESIGN.md#sp-release @d52b0c0dd2d2
