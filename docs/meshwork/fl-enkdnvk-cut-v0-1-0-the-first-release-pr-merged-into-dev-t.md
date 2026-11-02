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
  2026-11-02, end of session de9bd423, for a fresh session. Read
  fl-4fcbtkx's settings list and the owner's words below before anything
  else.
  
  State on origin: dev at the commit this handoff was committed in (see
  git log), main at 3ac2723 (the last main push; main now moves only by
  the dev→main PR), default branch dev, tag v0.0.0 on d2f29de, branch
  release/next at e1949b5 carrying "chore(release): v0.30.1" (stale once
  dev moves; the dispatch rebuilds it). The log counts to 0.30.1 plus
  whatever landed after 84a4936 (`./scripts/version-from-log.sh`).
  
  The process (DESIGN.md § Release, all three scripts with green
  self-tests): a push to dev runs no ci (owner: "A PUSH IS NOT SUPPOSED TO
  BE AN INTENSIVE TEST! A PUSH IS JUST A PUSH!"); `gh workflow run
  cut-release.yml --ref dev` on the owner's word opens or refreshes the
  release PR against dev (Cargo.toml, Cargo.lock, CHANGELOG.md entry); the
  PR runs every ci job (gate, msrv, deny, dbsp, miri, commits); the owner
  merges it, then the dev→main PR; the push to main runs cut-release's
  release job (tag, GitHub Release, .crate, attestation, upload) and ci.
  No tool beyond git, cargo, gh and the two GitHub-owned actions (owner:
  "THIS IS A MINIMALIST PROJECT THAT IS SUPPOSED TO HAVE NO DEPENDENCIES
  IT DOESN'T NEED").
  
  GitHub Actions is not green and the owner knows it: today's ci runs on
  dev were cancelled by successive pushes (cancel-in-progress) or were
  still running when the session ended (41495939738 dev, 41491375969 main;
  the session could not cancel them, the owner was given the command);
  cut-release failed three times at the PR step for want of
  RELEASE_PR_TOKEN or the Actions-may-create-PRs setting; four release-plz
  runs are a deleted workflow's history. Nothing of the release job (tag,
  attest, upload) has run yet, and the repo is private, so the attestation
  step is unproven.
  
  What proves the process, in order, each the owner's act or word: (1)
  RELEASE_PR_TOKEN (GitHub App, Contents + Pull requests read/write) or
  the Actions setting; (2) `gh workflow run cut-release.yml --ref dev`:
  the release PR opens, ci runs on it, the owner reads the number and the
  entry; (3) flip public; (4) merge the release PR, open and merge
  dev→main, watch the release job, `gh attestation verify
  float-abel-<v>.crate --repo jbrjake/float-abel`; comment the output
  here. Do not push to dev more than once per piece of work; do not add a
  tool; do not re-offer crates.io (owner: "I WILL TELL YOU WHEN I WANT
  IT").
---

The owner's merges, twice, and the session's watch. The release PR cut-release.yml keeps against dev carries the version the log counts to (owner: "YOU NEED TO BE COUNTING FROM 0.0.0 ON FROM _THE VERY FIRST COMMIT_"), Cargo.lock and the CHANGELOG entry; no version is typed anywhere by hand. The owner merges it (merge commit); the session opens dev→main (`gh pr create --base main --head dev --title "chore(release): cut the release the log yields"`) before anything else lands on dev; the owner merges it; the release job tags `v<that version>` on the merge commit, creates the GitHub Release with the entry as notes, builds the .crate with --locked, attests it and uploads .crate and .sha256. `gh attestation verify float-abel-<version>.crate --repo jbrjake/float-abel` passes from a clean machine. crates.io stays out until the owner says. Done is the verify (a released version heading in CHANGELOG.md) plus the release URL and the verify command's output in a comment here.

## log
- 2026-11-02T15:33Z created
- 2026-11-02T17:26Z handoff by claude (de9bd423-d8b5-4cf3-8a2c-44d698f4cc3e)
- 2026-11-02T17:51Z handoff by claude (de9bd423-d8b5-4cf3-8a2c-44d698f4cc3e)
- 2026-11-02T18:05Z handoff by claude (de9bd423-d8b5-4cf3-8a2c-44d698f4cc3e)
