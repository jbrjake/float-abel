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

Owner, 2026-11-02: "i don't understand why we don't have attested release builds available. how are you not seeing this as a massive gap, especially considering i tell people to use releases in the readme?"; "part of this needs to be moving to a dev branch for dev, main for releases, pr from dev to main to cut one, versions automatically created from conventional commit log tracking, like anyone would expect from a grown-up, mature project other people are going to use"; "IT'S NOT V0.1.0 IT'S WHAT THE CONVENTIONAL COMMITS SAY IT IS"; "YOU NEED TO BE COUNTING FROM 0.0.0 ON FROM _THE VERY FIRST COMMIT_!"; "THIS IS A MINIMALIST PROJECT THAT IS SUPPOSED TO HAVE NO DEPENDENCIES IT DOESN'T NEED!"

The shape as built (fl-05nqrwd), DESIGN.md § Release the spec:

- Branches. `dev` is where work lands and the default branch; `main` holds releases and accepts only the dev→main PR, merged as a merge commit.
- The version. Counted from 0.0.0 over every commit since the first, in order: breaking +major (and zero the rest), else feat +minor (and zero the patch), else fix +patch; other types leave it alone; merges skipped. `scripts/version-from-log.sh`, fixtures first. Nobody types a number.
- The release PR. `cut-release.yml` on every push to dev runs `scripts/prepare-release.sh`: the count into Cargo.toml and Cargo.lock, the entry for the commits since the last `v*` tag (grouped by type by `scripts/changelog-entry.sh`) under CHANGELOG.md's `## [Unreleased]` line, committed as `chore(release): vX.Y.Z` on `release/next` and opened or refreshed as a PR against dev. The owner merges it.
- The release. On the dev→main merge the workflow refuses a Cargo.toml that disagrees with the count, and when the version has no tag: `gh release create vX.Y.Z --target <merge commit>` with the entry as notes, `cargo package --locked`, sha256, actions/attest-build-provenance, `gh release upload`. Readers verify with `gh attestation verify`.
- Dependencies. None beyond git, cargo and gh on the runner and the two GitHub-owned actions already pinned (checkout, attest-build-provenance); the scripts are the repository's.
- crates.io. Outside the release until the owner says; release.yml's dispatch publish stays dormant.
- Commit discipline. `scripts/check-commit-message.sh` in the commit-msg hook, both gates and ci's commits job, because the count reads the subjects.

Verify: the first cut writes a released version heading into CHANGELOG.md.

## log
- 2026-11-02T15:33Z created
- 2026-11-02T15:34Z cover docs/DESIGN.md#sp-release @3478ee626df5
- 2026-11-02T16:38Z cover docs/DESIGN.md#sp-release @d52b0c0dd2d2
