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
verify: "all(lacks .github/workflows/release.yml release-pr, lacks .github/workflows/release.yml workflow_dispatch, exists scripts/release-pr.sh, contains CLAUDE.md release-pr.sh, contains docs/DESIGN.md release-pr.sh)"
status: doing
created: 2026-11-02T19:55Z
claimed-by: claude (987c488d-43f3-4192-b0e1-6b279735b89c)
---
Owner, 2026-11-02, in session: "why do we need a workflow for this? why can't it be something done from a clone using gh?"; "like you're asking me to jump through all these hoops to give a cloud worker broad permissions to do something we could do right here with gh, no?"; "OF COURSE FIX IT!"

release.yml's release-pr job checked out dev, ran scripts/prepare-release.sh, committed the three files on release/next, force-pushed and ran `gh pr create` or `gh pr edit`. Every step runs from a clone under the developer's own gh login, which already opens pull requests on this repository. The runner added nothing and cost a RELEASE_PR_TOKEN secret or the repository-wide "Allow GitHub Actions to create and approve pull requests" toggle, the reason the job failed three times on 2026-11-02 (runs 41490353635, 41495586562 and the dispatch after fl-ezhk4cz). The shape was release-plz's, kept when the scripts replaced it (fl-05nqrwd) and not re-examined.

The fix: `scripts/release-pr.sh` does what the job did, from a temporary worktree at origin/dev so the developer's checkout and branches are untouched: prepare-release.sh, the commit `chore(release): vX.Y.Z` on a detached head, `git push --force origin HEAD:refs/heads/release/next`, then `gh pr edit` or `gh pr create` against dev with the entry as the body. `--dry-run` stops before the push and prints the version, the diff and the entry. `--self-test` runs the whole path against a bare fixture origin with gh stubbed through `$GH`: the PR opens, then refreshes, then nothing to do once dev carries the version, then a dry run pushes nothing. release.yml loses the job, the `workflow_dispatch` trigger and the token comment; the release job on the push to main stays, because the attestation signs with the workflow's OIDC identity, which a clone cannot produce. CLAUDE.md § Commands names the script in place of `gh workflow run release.yml --ref dev`; DESIGN.md § Release says who cuts the PR and why no runner does; fl-4fcbtkx's settings list loses item 1.

## log
- 2026-11-02T19:55Z created
- 2026-11-02T19:56Z open→doing — claimed by claude (987c488d-43f3-4192-b0e1-6b279735b89c)
