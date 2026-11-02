---
id: fl-05nqrwd
title: "Count the version from 0.0.0 over every commit since the first (breaking +major, feat +minor, fix +patch) in a script with fixtures, and drive the release PR and the release from it in place of release-plz"
status: doing
category: crate/release
labels: [reproducible-floats]
parent: fl-m3cymxv
verify: "all(exists scripts/version-from-log.sh, absent release-plz.toml, absent .github/workflows/release-plz.yml, lacks CLAUDE.md release-plz)"
docs:
  - docs/DESIGN.md#release-sp-release
  - CLAUDE.md#standing-decisions
seq: 15
created: 2026-11-02T17:38Z
claimed-by: claude (de9bd423-d8b5-4cf3-8a2c-44d698f4cc3e)
---

Owner, 2026-11-02, in session, on the release process as built with release-plz (one bump per release, 0.1.0 on today's log): "THERE IS ABSOLUTELY NO WAY THAT OUR VERSION IS 0.1.0! I HAVE TOLD YOU THIS OVER AND OVER! YOU NEED TO BE COUNTING FROM 0.0.0 ON FROM _THE VERY FIRST COMMIT_! I CAN'T BELIEVE YOU SPENT ALL THIS TIME WITHOUT TELLING ME YOU WERE IGNORING THE VERY THING I INSISTED YOU DO!"

The rule, Conventional Commits' own semver mapping applied to every commit in order from the first: start at 0.0.0; a breaking change (`!` after the type or a `BREAKING CHANGE:` footer) adds one to the major and zeroes the rest; else a `feat` adds one to the minor and zeroes the patch; else a `fix` adds one to the patch; every other type leaves the version alone. Merge commits are skipped; the walk is `git rev-list --reverse --topo-order --no-merges`. Today's log (227 commits) counts to 0.29.3. No release tool does this (release-plz, release-please, git-cliff --bump and cocogitto all take one bump per release from the strongest commit), so release-plz goes.

Build, red first: `scripts/version-from-log.sh` with a `--self-test` over fixture sequences (each a list of subjects and bodies with the version they must reach) and a default mode that walks the repo and prints the version; `.github/workflows/cut-release.yml` with (a) on push to dev: count V; if Cargo.toml's version or CHANGELOG.md's top entry is not V, write the version into Cargo.toml and Cargo.lock, render the entry for the commits since the last `v*` tag with git-cliff (hash-pinned through taiki-e/install-action's manifest) under `## [V]`, splice it under CHANGELOG.md's `## [Unreleased]` line, commit on a release branch and open or update the PR "chore(release): vV" against dev (RELEASE_PLZ_TOKEN or GITHUB_TOKEN, as before); (b) on push to main: count V, refuse unless Cargo.toml says V (the release PR was not merged first), skip if the tag exists, else `gh release create vV --target $GITHUB_SHA` with the entry as notes, then the package, attest and upload steps as they stand. release-plz.toml and release-plz.yml are removed; CLAUDE.md § Standing decisions, DESIGN.md § Release and CHANGELOG.md's header say the counting rule in the owner's words; the v0.0.0 tag stays as the changelog range's base. Done is the self-test green, the local count printing 0.29.3 at this tree, both jobs green on a dev push and a main push (the dev job ending at the token's 403 counts, as before), and the readback of the consequence to the owner: today's first release is 0.29.3, the next feat makes it 0.30.0, the next fix 0.29.4, and the first breaking change 1.0.0.

## log
- 2026-11-02T17:38Z created
- 2026-11-02T17:38Z open→doing — claimed by claude (de9bd423-d8b5-4cf3-8a2c-44d698f4cc3e)

## comments
- 2026-11-02T17:43Z [claude (de9bd423-d8b5-4cf3-8a2c-44d698f4cc3e)] 2026-11-02, mid-build, owner, in session: "STOP ADDING ALL THESE DEPENDENCIES! I NEVER TOLD YOU TO START INTRODUCING UNTRUSTED DEPENDENCIES INTO OUR BUILD PROCESS! WTF IS GIT-CLIFF? WE NEVER TALKED ABOUT THIS!" and "THIS IS A MINIMALIST PROJECT THAT IS SUPPOSED TO HAVE NO DEPENDENCIES IT DOESN'T NEED! READ. THE. ROOM! DID YOU EVEN READ YOUR OWN README? DO YOU EVEN UNDERSTAND THE PROJECT YOU'RE WORKING ON? STOP. VIOLATING. ITS. SPIRIT!" So: no git-cliff, no release-plz, no taiki-e/install-action, no cargo-semver-checks in the release workflow. The release runs on git, cargo and gh on the runner, actions/checkout and actions/attest-build-provenance (both already pinned here), and the repo's own scripts: version-from-log.sh (the count), changelog-entry.sh (the entry, grouped by type), prepare-release.sh (Cargo.toml, Cargo.lock, CHANGELOG.md). The local release-plz install was removed (cargo uninstall) and the scratch binaries deleted.
