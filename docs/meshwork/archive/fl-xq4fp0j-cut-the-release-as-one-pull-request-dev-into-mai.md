---
id: fl-xq4fp0j
title: "Cut the release as one pull request, dev into main: release-pr.sh commits the version and the entry onto dev, pushes, and opens or refreshes the dev→main PR; no release/next, no PR against dev"
status: done
category: crate/release
labels: [reproducible-floats]
parent: fl-m3cymxv
relates: [fl-9hcnr6q, fl-ebbxwd2]
verify: "all(lacks scripts/release-pr.sh release/next, contains scripts/release-pr.sh /--base main --head dev/, lacks CLAUDE.md /gh pr create --base main/, lacks docs/DESIGN.md /PR against .dev./, lacks .github/workflows/release.yml /PR against dev/)"
docs:
  - docs/DESIGN.md#release-sp-release
  - CLAUDE.md#commands
seq: 13
created: 2026-11-02T20:39Z
---

Owner, 2026-11-02, in session, ruling fl-9hcnr6q: "i want it to be a single PR, dev to main."

The release PR against dev (release/next, the three generated files, merged by the owner before the dev→main PR) was release-plz's shape, kept through fl-05nqrwd and fl-ebbxwd2. It goes. `scripts/release-pr.sh` now runs in place, from a clone on dev, clean and at origin/dev, under the developer's own gh login: `scripts/prepare-release.sh` writes the version the log counts to into Cargo.toml and Cargo.lock and the entry into CHANGELOG.md; the three files are committed on dev as `chore(release): vX.Y.Z` and pushed; the one pull request, dev into main, is then opened, or refreshed with the new title and body when one is open. Merging it is the release: release.yml on the push to main refuses a Cargo.toml that disagrees with the count, packages, attests, tags and publishes, unchanged. With dev already carrying the version the commit is skipped and the PR still opened or refreshed; with the version tagged there is nothing to do. `--dry-run` prints the version, the diff stat and the entry, puts the three files back and pushes nothing. A dirty tree, another branch, or a dev that is not at origin/dev is refused before anything is written.

The release commit and its push skip the hooks: the content of the tree is what the push gate passed at origin/dev, and the three files differ from it by a version line, the lockfile's own-package record and a changelog entry, none of which the gate's checks read; the dev→main pull request runs the whole suite on the runner. The script says so when it skips them.

The self-test rebuilds the fixture for this shape: a bare origin carrying dev and main; the PR opens on the first run, refreshes without a second commit on the next, has nothing to do once the version is tagged, a dry run moves nothing and leaves the tree clean, and a dirty tree or another branch is refused with no push and no gh call. CLAUDE.md § Commands loses the separate `gh pr create --base main --head dev` line; DESIGN.md § Release and release.yml's comments say one pull request; the release job's refusal message names the script. The orphaned release/next on origin (the runner's ae1c010) is deleted if the repository's ruleset, which forbids deletion on every branch, lets the owner's token do it; otherwise it is the owner's to delete.

## log
- 2026-11-02T20:39Z created
- 2026-11-02T20:39Z open→doing — claimed by claude (987c488d-43f3-4192-b0e1-6b279735b89c)
- 2026-11-02T20:43Z doing→done — verify exit 0 @ 30626fe+8

## comments
- 2026-11-02T20:43Z [claude (987c488d-43f3-4192-b0e1-6b279735b89c)] 2026-11-02 built at 30626fe (the script, release.yml's comments and refusal message, CLAUDE.md § Commands and the standing decision, DESIGN.md § Release; the sp-release pin re-read and repinned). Proven here: `./scripts/release-pr.sh --self-test` green in 2 s, covering the open, the refresh without a second commit, nothing to do once tagged, a dry run that leaves HEAD, origin/dev and the tree untouched, and the three refusals (dirty tree, another branch, dev behind origin), and red on three deliberate breaks in scratch copies (the push removed: "dev was not pushed"; the dry run's restore removed: "the dry run left the tree dirty: M CHANGELOG.md"; the clean-tree check removed: "a dirty tree was not refused" and "a refusal moved origin/dev"). release.yml parses with one job on one trigger and check-actions-pinned.sh passes; no `release/next`, `PR against dev`, `merge the release PR`, `both merges` or `gh pr create --base main` is left outside the store. The commit gate ran all three release self-tests on this commit. The repository's 'branch-protection' ruleset (deletion and non_fast_forward on every branch) is satisfied by this shape, since the release commit is a fast-forward onto dev; it would have refused the old force-push to release/next. The dry run against origin on this shape follows in the next comment, after the store commit cleans the tree the script requires.
- 2026-11-02T20:45Z [claude (987c488d-43f3-4192-b0e1-6b279735b89c)] 2026-11-02 dry run against origin at 729362e, on a clean dev equal to origin/dev: prepare-release wrote 0.0.0 -> 0.30.1, CHANGELOG.md +137, Cargo.lock and Cargo.toml one line each; the script printed 'would commit chore(release): v0.30.1 onto dev, push it, and open or refresh the pull request dev → main' with the entry opening '## [0.30.1](.../compare/v0.0.0...v0.30.1) - 2026-11-02'; 1.4 s. Afterwards: git status empty, HEAD unchanged and equal to origin/dev, no open dev→main pull request. Pushed to origin through the regression gate at 729362e (regression: OK). Not run: the script without --dry-run, the owner's word. Not done: deleting release/next on origin (the runner's ae1c010); the API delete was refused by the session's permission classifier as a destructive git action and the 'branch-protection' ruleset forbids deletion on every branch, so it is the owner's, with a bypass.
