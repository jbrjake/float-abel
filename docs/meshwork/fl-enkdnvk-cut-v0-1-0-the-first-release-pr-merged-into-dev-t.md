---
id: fl-enkdnvk
title: "Cut the first release: the release PR merged into dev, the dev→main PR merged, the tag and GitHub Release created at the version the conventional-commit log yields, the attested .crate attached and verified with gh attestation verify"
category: crate/release
labels: [reproducible-floats]
seq: 19
parent: fl-m3cymxv
needs: [fl-kpk3gd6, fl-bct35bq, fl-4fcbtkx, fl-d609gg2, fl-05nqrwd]
relates: [fl-7en8z60, fl-fm78m8d]
docs:
  - README.md#getting-it
verify: contains CHANGELOG.md /^## \[?[0-9]+\.[0-9]+\.[0-9]+/
status: open
created: 2026-11-02T15:33Z
handoff: |
  2026-11-02, session de9bd423. The release process is built on the
  repository's own scripts, with no release tool or third-party action
  (owner: "THIS IS A MINIMALIST PROJECT THAT IS SUPPOSED TO HAVE NO
  DEPENDENCIES IT DOESN'T NEED"), and proven locally; the runner proof of
  the new workflow is the push after f00a030.
  
  Built: scripts/version-from-log.sh (the count from 0.0.0 over every
  commit since the first; --self-test over 19 sequences; 0.30.0 at
  f00a030), scripts/changelog-entry.sh (the entry grouped by type,
  --self-test against an expected block), scripts/prepare-release.sh
  (Cargo.toml, Cargo.lock via cargo update --workspace --offline, the
  entry under CHANGELOG.md's Unreleased line; idempotent; proven in a
  scratch worktree: 0.0.0 -> 0.29.3 at 3ac2723 and cargo build --locked
  --offline green after), .github/workflows/cut-release.yml (release-pr
  job on dev, release job on main), scripts/check-commit-message.sh in the
  commit-msg hook, both gates and ci's commits job, ci on dev, the
  annotated tag v0.0.0 on d2f29de as the changelog range's base,
  CLAUDE.md, DESIGN.md § Release, README § getting it, CHANGELOG.md's
  header. Default branch is dev.
  
  The sequence to cut the first release, once the owner has done
  fl-4fcbtkx's items 1 and 2 (RELEASE_PR_TOKEN or the Actions-may-open-PRs
  setting; flip public): (a) push to dev or re-run the dev cut-release
  job: the release PR "chore(release): v<count>" opens against dev from
  release/next with Cargo.toml, Cargo.lock and the CHANGELOG entry; the
  owner merges it with the merge button. (b) Right away, before anything
  else lands on dev: gh pr create --base main --head dev --title
  "chore(release): cut the release the log yields"; the owner merges it
  with the merge button. (c) The release job on main checks Cargo.toml
  against the count, tags v<count> on the merge commit, publishes the
  GitHub Release with the entry as notes, builds float-abel-<count>.crate
  with --locked, attests it and uploads .crate and .sha256. (d) From a
  clean machine: gh attestation verify float-abel-<count>.crate --repo
  jbrjake/float-abel; paste the output and the release URL as a comment
  here.
  
  Watch for: every feat on dev moves the count (the next feat after the
  release PR opens re-renders the PR at the new number, which is the rule
  working); the attest step fails on a private repo, so the flip comes
  before (b); the main ruleset's required checks must name jobs that run
  on a PR from dev; dependabot PRs #2 and #3 fail the commits job on their
  'Bump' titles (fl-hp7md5n); a Cargo.toml on main that disagrees with the
  count fails the release job with the instruction to merge the release PR
  first.
---

The owner's merges, twice, and the session's watch. The release PR cut-release.yml keeps against dev carries the version the log counts to (owner: "YOU NEED TO BE COUNTING FROM 0.0.0 ON FROM _THE VERY FIRST COMMIT_"), Cargo.lock and the CHANGELOG entry; no version is typed anywhere by hand. The owner merges it (merge commit); the session opens dev→main (`gh pr create --base main --head dev --title "chore(release): cut the release the log yields"`) before anything else lands on dev; the owner merges it; the release job tags `v<that version>` on the merge commit, creates the GitHub Release with the entry as notes, builds the .crate with --locked, attests it and uploads .crate and .sha256. `gh attestation verify float-abel-<version>.crate --repo jbrjake/float-abel` passes from a clean machine. crates.io stays out until the owner says. Done is the verify (a released version heading in CHANGELOG.md) plus the release URL and the verify command's output in a comment here.

## log
- 2026-11-02T15:33Z created
- 2026-11-02T17:26Z handoff by claude (de9bd423-d8b5-4cf3-8a2c-44d698f4cc3e)
- 2026-11-02T17:51Z handoff by claude (de9bd423-d8b5-4cf3-8a2c-44d698f4cc3e)
