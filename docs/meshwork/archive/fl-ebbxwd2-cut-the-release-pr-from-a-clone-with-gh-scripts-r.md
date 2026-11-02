---
id: fl-ebbxwd2
title: "Cut the release PR from a clone with gh: scripts/release-pr.sh in place of release.yml's release-pr job, so no runner holds write access to the repository for a commit and a pull request"
category: crate/release
labels: [reproducible-floats]
seq: 13
parent: fl-m3cymxv
docs:
  - docs/DESIGN.md#release-sp-release
  - CLAUDE.md#commands
verify: "all(lacks .github/workflows/release.yml /release-pr:/, lacks .github/workflows/release.yml workflow_dispatch, exists scripts/release-pr.sh, contains CLAUDE.md release-pr.sh, contains docs/DESIGN.md release-pr.sh)"
status: done
created: 2026-11-02T19:55Z
---
Owner, 2026-11-02, in session: "why do we need a workflow for this? why can't it be something done from a clone using gh?"; "like you're asking me to jump through all these hoops to give a cloud worker broad permissions to do something we could do right here with gh, no?"; "OF COURSE FIX IT!"

release.yml's release-pr job checked out dev, ran scripts/prepare-release.sh, committed the three files on release/next, force-pushed and ran `gh pr create` or `gh pr edit`. Every step runs from a clone under the developer's own gh login, which already opens pull requests on this repository. The runner added nothing and cost a RELEASE_PR_TOKEN secret or the repository-wide "Allow GitHub Actions to create and approve pull requests" toggle, the reason the job failed three times on 2026-11-02 (runs 41490353635, 41495586562 and the dispatch after fl-ezhk4cz). The shape was release-plz's, kept when the scripts replaced it (fl-05nqrwd) and not re-examined.

The fix: `scripts/release-pr.sh` does what the job did, from a temporary worktree at origin/dev so the developer's checkout and branches are untouched: prepare-release.sh, the commit `chore(release): vX.Y.Z` on a detached head, `git push --force origin HEAD:refs/heads/release/next`, then `gh pr edit` or `gh pr create` against dev with the entry as the body. `--dry-run` stops before the push and prints the version, the diff and the entry. `--self-test` runs the whole path against a bare fixture origin with gh stubbed through `$GH`: the PR opens, then refreshes, then nothing to do once dev carries the version, then a dry run pushes nothing. release.yml loses the job, the `workflow_dispatch` trigger and the token comment; the release job on the push to main stays, because the attestation signs with the workflow's OIDC identity, which a clone cannot produce. CLAUDE.md § Commands names the script in place of `gh workflow run release.yml --ref dev`; DESIGN.md § Release says who cuts the PR and why no runner does; fl-4fcbtkx's settings list loses item 1.

## log
- 2026-11-02T19:55Z created
- 2026-11-02T19:56Z open→doing — claimed by claude (987c488d-43f3-4192-b0e1-6b279735b89c)
- 2026-11-02T20:06Z doing→done — verify exit 0 @ 329fa23+6

## comments
- 2026-11-02T20:06Z [claude (987c488d-43f3-4192-b0e1-6b279735b89c)] 2026-11-02 built at c9aaa03 (the script, the workflow, CLAUDE.md, DESIGN.md) and 329fa23 (the gates; the self-test made hermetic). Proven here: `./scripts/release-pr.sh --self-test` green (6 s), and red on three deliberate breaks in scratch copies (the commit subject reworded: "release/next carries 'chore: release v0.1.1'"; the push moved ahead of the dry-run check: "the dry run moved release/next"; the edit branch disabled: "no pr edit 7 in the gh log"). `./scripts/release-pr.sh --dry-run` against origin: prepare-release wrote 0.0.0 -> 0.30.1, CHANGELOG.md +134, Cargo.lock and Cargo.toml one line each, the body opening `## [0.30.1](.../compare/v0.0.0...v0.30.1) - 2026-11-02`; nothing pushed, `git worktree list` back to the one checkout, `git status` showing only the session's edits. release.yml parses (ruby -ryaml: one job, one trigger), check-actions-pinned.sh passes, no `release-pr:`, `workflow_dispatch`, `RELEASE_PR_TOKEN` or `pull-requests` left in it. Found on the gates commit: the commit hook exports GIT_INDEX_FILE and GIT_DIR, and the self-test's fixture repositories inherited them; with absolute paths (a scratch clone) the fixture's `git add -A` staged into the enclosing repository's index, and `git init` re-initialised it. The self-test now unsets every GIT_* before building fixtures, and passes plain, under relative and under absolute hook variables in a scratch clone with that clone's index and branches untouched. Not run: the script without --dry-run, which opens the real release PR and is the owner's word (fl-enkdnvk step 1); gh's behaviour from the detached worktree is avoided by running gh from the checkout root after the push.
