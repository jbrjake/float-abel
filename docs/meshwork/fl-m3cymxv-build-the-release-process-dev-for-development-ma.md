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
    sha: 3cb6c64138078bd1bc71557ce93de7dd5e9bcbd3f10f82a59d6306fa1adb40e4
---

Owner, 2026-11-02: "i don't understand why we don't have attested release builds available. how are you not seeing this as a massive gap, especially considering i tell people to use releases in the readme?"; "part of this needs to be moving to a dev branch for dev, main for releases, pr from dev to main to cut one, versions automatically created from conventional commit log tracking, like anyone would expect from a grown-up, mature project other people are going to use"; "IT'S NOT V0.1.0 IT'S WHAT THE CONVENTIONAL COMMITS SAY IT IS"; "YOU NEED TO BE COUNTING FROM 0.0.0 ON FROM _THE VERY FIRST COMMIT_!"; "THIS IS A MINIMALIST PROJECT THAT IS SUPPOSED TO HAVE NO DEPENDENCIES IT DOESN'T NEED!"; on the release PR, "why do we need a workflow for this? why can't it be something done from a clone using gh?"; "OF COURSE FIX IT!"; "i want it to be a single PR, dev to main."

The shape as built (fl-05nqrwd, fl-ezhk4cz, fl-ebbxwd2, fl-xq4fp0j), DESIGN.md § Release the spec:

- Branches. `dev` is where work lands and the default branch; `main` holds releases and accepts only the dev→main PR, merged as a merge commit.
- The version. Counted from 0.0.0 over every commit since the first, in order: breaking +major (and zero the rest), else feat +minor (and zero the patch), else fix +patch; other types leave it alone; merges skipped. `scripts/version-from-log.sh`, fixtures first. Nobody types a number.
- The one pull request. `scripts/release-pr.sh`, from a clone on dev, clean and at origin/dev, under the developer's own gh login: `scripts/prepare-release.sh` writes the count into Cargo.toml and Cargo.lock and the entry for the commits since the last `v*` tag (grouped by type by `scripts/changelog-entry.sh`) under CHANGELOG.md's `## [Unreleased]` line; the three files are committed onto dev as `chore(release): vX.Y.Z` and pushed; the pull request dev→main is opened with the entry as its body, or refreshed when open. `--dry-run` prints the version, the diff and the entry, puts the files back and pushes nothing. The release commit and its push skip the hooks, since the tree is the one the gate passed at origin/dev plus a version line, a lockfile record and a changelog entry, and the PR runs the suite. No runner holds write access to the repository for this: a commit and a pull request carry no identity a runner could add. The owner merges it.
- The release. On the dev→main merge release.yml's one job refuses a Cargo.toml that disagrees with the count, and when the version has no tag: `cargo package --locked`, sha256, actions/attest-build-provenance, then `gh release create vX.Y.Z --target <merge commit>` with the entry as notes and the .crate and its sha256 attached. The attestation carries the workflow's identity, which is why that step is the runner's. Readers verify with `gh attestation verify`.
- Dependencies. None beyond git, cargo and gh, on the developer's machine and on the runner, and the two GitHub-owned actions already pinned (checkout, attest-build-provenance); the scripts are the repository's, each with a `--self-test` that both gates run (fl-jkb64s3).
- crates.io. Outside the release until the owner says; publish.yml's dispatch stays dormant.
- Commit discipline. `scripts/check-commit-message.sh` in the commit-msg hook, both gates and ci's commits job, because the count reads the subjects.

Verify: the first cut writes a released version heading into CHANGELOG.md.

## log
- 2026-11-02T15:33Z created
- 2026-11-02T15:34Z cover docs/DESIGN.md#sp-release @3478ee626df5
- 2026-11-02T16:38Z cover docs/DESIGN.md#sp-release @d52b0c0dd2d2
- 2026-11-02T18:34Z cover docs/DESIGN.md#sp-release @5e919747f70b
- 2026-11-02T19:59Z cover docs/DESIGN.md#sp-release @0c55c448f94b
- 2026-11-02T20:41Z cover docs/DESIGN.md#sp-release @3cb6c6413807

## comments
- 2026-11-02T18:34Z [claude (227b40e5-20bd-44b0-9b79-d44c1a3c3587)] 2026-11-02, fl-ezhk4cz at 4bec2d5: the release is .github/workflows/release.yml (from cut-release.yml) and the crates.io publish .github/workflows/publish.yml (from release.yml); the body's 'cut-release.yml on every push to dev' is two revisions stale: the release PR opens on the owner's dispatch alone. DESIGN.md § Release now says the nightly and the dev→main PR are the two test points and the .crate is built and attested before the tag; the sp-release pin is re-read and repinned with this comment.
