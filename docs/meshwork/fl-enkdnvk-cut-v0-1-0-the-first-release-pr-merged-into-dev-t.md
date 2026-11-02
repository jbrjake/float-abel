---
id: fl-enkdnvk
title: "Cut the first release: the release PR merged into dev, the dev→main PR merged, the tag and GitHub Release created at the version the conventional-commit log yields, the attested .crate attached and verified with gh attestation verify"
category: crate/release
labels: [reproducible-floats]
seq: 19
parent: fl-m3cymxv
needs: [fl-kpk3gd6, fl-bct35bq, fl-4fcbtkx, fl-d609gg2, fl-05nqrwd, fl-ebbxwd2, fl-xq4fp0j]
relates: [fl-7en8z60, fl-fm78m8d]
docs:
  - README.md#getting-it
verify: contains CHANGELOG.md /^## \[?[0-9]+\.[0-9]+\.[0-9]+/
status: open
created: 2026-11-02T15:33Z
handoff: |
  2026-11-02, end of the session that landed fl-ebbxwd2, fl-jkb64s3 and
  fl-xq4fp0j and saw the owner flip the repo public (fl-4fcbtkx closed),
  for a fresh session. Read fl-4fcbtkx's last comment (what the flip made
  settable) and the owner's words below before anything else.
  
  State on origin: dev at the commit this handoff was committed in (see
  git log), main at 3ac2723 (main moves only by the dev→main PR),
  default
  branch dev, public, tag v0.0.0 on d2f29de. A ruleset 'branch-protection'
  forbids deletion and force-push on every branch. The log counts to
  0.30.1 (`./scripts/version-from-log.sh`); `./scripts/release-pr.sh
  --dry-run` printed that version, a three-file diff stat and the entry
  this session and pushed nothing.
  
  The process (DESIGN.md § Release; the three release scripts'
  self-tests run in both gates): one pull request, dev into main (owner:
  "i want it to be a single PR, dev to main."). `./scripts/release-pr.sh`
  from this clone, on dev, clean, at origin/dev, under the owner's gh
  login: it commits `chore(release): vX.Y.Z` onto dev, pushes (hooks
  skipped, and it says so), and opens the dev→main PR with the entry as
  the body, or refreshes the open one. No token, no Actions setting, no
  runner (owner: "why can't it be something done from a clone using gh?";
  "OF COURSE FIX IT!"). The PR runs the whole suite; the owner merges it;
  the push to main runs release.yml's one job: refuse a Cargo.toml that
  disagrees with the count, `cargo package --locked`, attest, then the
  tag and the GitHub Release with the .crate and its sha256 attached.
  Nothing beyond git, cargo, gh and the two pinned GitHub-owned actions
  (owner: "THIS IS A MINIMALIST PROJECT THAT IS SUPPOSED TO HAVE NO
  DEPENDENCIES IT DOESN'T NEED").
  
  What proves the process, in order, each the owner's act or word:
  (1) `./scripts/release-pr.sh` on the owner's word; the owner reads the
  PR's number, entry and checks. (2) The owner merges it; watch the
  release job (`gh run list --workflow release.yml`, `gh run view <id>`
  to conclusion=success); then `gh attestation verify
  float-abel-<v>.crate --repo jbrjake/float-abel` from a clean download;
  comment the output here. The attest step has never run: the repo was
  private until today. Do not push to dev more than once per piece of
  work; do not add a tool; do not re-offer crates.io (owner: "I WILL TELL
  YOU WHEN I WANT IT").
---

The owner's merges, twice, and the session's watch. The release PR cut-release.yml keeps against dev carries the version the log counts to (owner: "YOU NEED TO BE COUNTING FROM 0.0.0 ON FROM _THE VERY FIRST COMMIT_"), Cargo.lock and the CHANGELOG entry; no version is typed anywhere by hand. The owner merges it (merge commit); the session opens dev→main (`gh pr create --base main --head dev --title "chore(release): cut the release the log yields"`) before anything else lands on dev; the owner merges it; the release job tags `v<that version>` on the merge commit, creates the GitHub Release with the entry as notes, builds the .crate with --locked, attests it and uploads .crate and .sha256. `gh attestation verify float-abel-<version>.crate --repo jbrjake/float-abel` passes from a clean machine. crates.io stays out until the owner says. Done is the verify (a released version heading in CHANGELOG.md) plus the release URL and the verify command's output in a comment here.

## log
- 2026-11-02T15:33Z created
- 2026-11-02T17:26Z handoff by claude (de9bd423-d8b5-4cf3-8a2c-44d698f4cc3e)
- 2026-11-02T17:51Z handoff by claude (de9bd423-d8b5-4cf3-8a2c-44d698f4cc3e)
- 2026-11-02T18:05Z handoff by claude (de9bd423-d8b5-4cf3-8a2c-44d698f4cc3e)
- 2026-11-02T20:06Z handoff by claude (987c488d-43f3-4192-b0e1-6b279735b89c)
- 2026-11-02T20:43Z handoff by claude (987c488d-43f3-4192-b0e1-6b279735b89c)

## comments
- 2026-11-02T18:34Z [claude (227b40e5-20bd-44b0-9b79-d44c1a3c3587)] 2026-11-02, fl-ezhk4cz at 4bec2d5: the release workflow is .github/workflows/release.yml (from cut-release.yml); the owner's dispatch is `gh workflow run release.yml --ref dev`. The release job now packages and attests the .crate before creating the tag and the GitHub Release, and attaches the .crate and its sha256 in the `gh release create` call, so a failed package leaves no tag behind. The release PR into dev runs no ci (ci.yml triggers on PRs into main alone); the dev→main PR runs the whole suite, fuzz and golden included. The RELEASE_PR_TOKEN or the Actions setting is still what the dispatch needs: `gh secret list` is empty and `can_approve_pull_request_reviews` is false at this writing.
