---
id: fl-4fcbtkx
title: "Push main to origin and make the repo readable by leras's CI — git push origin main through the push gate, then flip jbrjake/float-abel public or add it to leras's ORESEUR_TOKEN PAT, both the owner's acts"
status: blocked
category: rulings
labels: [reproducible-floats]
answers: leras#le-yxhtwgw
docs:
  - CLAUDE.md#standing-decisions
  - docs/DESIGN.md#release-sp-release
seq: 56
created: 2026-10-16T16:51Z
verify: contains docs/meshwork/fl-4fcbtkx-push-main-to-origin-and-make-the-repo-readable-b.md /2026-[0-9][0-9]-[0-9][0-9] owner pushed main to origin/
blocked-reason: "step 2, the repo's readability for leras's runner (flip public or add to the ORESEUR_TOKEN PAT), is a GitHub setting on the owner's account; main is on origin at b14d830"
claimed-by: claude (03cbad3b-1e00-455d-9d44-3b0d9decee6e)
parent: fl-b9qtzty
---

Answers leras#le-yxhtwgw, whose body names both steps as owner actions on this repo. leras ships
its float sums lane against float-abel rev 75c25fe as a path dep on the sibling checkout, with the
git line beside it for the swap; main here is 147 commits ahead of origin/main, which holds the
initial commit alone, and jbrjake/float-abel is private, so the pin resolves for neither cargo
nor leras's Actions runner.

The two steps, in order:
1. `git push origin main` from this checkout, through the push gate (.githooks/pre-push runs
   scripts/regression.sh: strict lint, build, every test with --include-ignored, the release
   budget tests, cargo-deny on both closures; the gate needs the sandbox off and a warm target).
   leras then pins whatever origin/main holds; the surface it uses is Wide::{deposit, limbs,
   from_parts, specials, to_f64, div_f64, LIMBS}, Fixed256::{try_from(f64), limbs, from_limbs,
   to_f64, div_f64, quantize, LIMBS}, OutOfWindow and the std ops, so a rename there is a compile
   error at the swap, not a silent change.
2. Make the repo fetchable from leras's runner: flip it public (the plan's posture,
   PLAN-reproducible-floats.md §3.1), or add jbrjake/float-abel to the ORESEUR_TOKEN
   fine-grained PAT's repository list (leras docs/CI-CREDENTIALS.md).

Neither is a session's act: the push publishes 147 commits and the visibility change is a
GitHub setting on the owner's account. When both hold, this task's comment carries
"<date> owner pushed main to origin and made the repo readable by leras's CI", the verify goes
green, and leras's swap task waits only on leras closing its ask.

## log
- 2026-10-16T16:51Z created
- 2026-10-16T17:22Z open→blocked — owner, 2026-10-16, in session: "ignore the push to main for now"; held until the owner reopens it
- 2026-10-16T19:42Z blocked→open
- 2026-10-16T19:42Z open→doing — claimed by claude (03cbad3b-1e00-455d-9d44-3b0d9decee6e)
- 2026-10-16T19:44Z doing→blocked — step 2, the repo's readability for leras's runner (flip public or add to the ORESEUR_TOKEN PAT), is a GitHub setting on the owner's account; main is on origin at b14d830

## comments
- 2026-10-16T19:42Z [claude (03cbad3b-1e00-455d-9d44-3b0d9decee6e)] 2026-10-16 owner, in session: "push to origin". Step 1 is on: git push origin main through the push gate from this checkout. Step 2, the repo's readability for leras's runner, is a GitHub setting on the owner's account and is not asked for; the task stays open until the owner does it and writes the sentence the verify reads.
- 2026-10-16T19:44Z [claude (03cbad3b-1e00-455d-9d44-3b0d9decee6e)] 2026-10-16 step 1 done: main pushed to origin at b14d830 (ae9505a..b14d830, 88 commits) through the push gate, green end to end (MSRV 1.74 default features, the dbsp leg on 1.99.0, both deny closures). A plain push fails here with "Permission denied (publickey)" because the ssh-agent holds no identities; GIT_SSH_COMMAND naming ~/.ssh/github_jbrjake is what worked, no config changed. Step 2 is not done: jbrjake/float-abel is still private, so leras's runner cannot fetch rev b14d830 until the owner flips it public or adds the repo to the ORESEUR_TOKEN PAT, and the verify's sentence waits on that.
- 2026-10-20T01:02Z [claude (645d46e7-6f8d-45dc-9306-1e0d2847cda3)] 2026-10-20 owner word, offered the choice: "Scrub before visibility". Before step 2 makes the repo readable, the tree drops its names of sibling projects outside this repo (30 lines over 9 files at f128de6, task files included), and the plan ask fl-k7gec8v leaves this store.
- 2026-11-02T15:05Z [claude (c90f1eae-8c6b-41ec-aa59-9d0990f41bc2)] 2026-11-02 review of the repo's GitHub side ahead of the flip, read through `gh api`. The scrub is settled (fl-yka7wjf dropped on the owner's 2026-10-20 word), so what remains is settings, every one the owner's act, and none available to a private free-plan repo today (branch protection and rulesets answer 403 "make this repository public to enable this feature"):
  
  1. Settings → General: visibility public. Public standard runners are free, which also ends the fuzz workflow's "job was not started because recent account payments have failed or your spending limit needs to be increased" refusals of 2026-10-30 and -31 (runs 41037279134, 41189358250; they resumed on the 1st with the month's quota).
  2. Settings → Code security: enable Private vulnerability reporting (SECURITY.md names it as the only channel and the URL 404s until it is on), Dependabot alerts (disabled today), and confirm secret scanning with push protection, which GitHub turns on for public repos.
  3. Settings → Actions → General: "Require actions to be pinned to a full-length commit SHA" (sha_pinning_required is false; this is the standing decision as a server-side check), "Require approval for all external contributors" on fork pull requests (the pull_request triggers in ci, fuzz and golden would otherwise run a stranger's PR on first contribution), workflow permissions stay read-only (already read).
  4. Settings → Environments: create `release` with the owner as a required reviewer. release.yml names it and nothing exists; a dispatch would auto-create it with no gate, and crates.io's Trusted Publishing config will name the same environment.
  5. Rules → Rulesets on main: block force-push and deletion, require the ci gate/msrv/deny jobs and golden's seven targets to pass before merge.
  6. About: topics (rust, no-std, floating-point, reproducibility, differential-dataflow, dbsp) and the docs.rs link once published.
  7. Dependabot PRs #1–#3 are red on stale bases (PR #2's gate at b14d830 could not exec the meshwork shim, since skipped under CI; #1 and #3 at 4a83a00 hit the miri index-out-of-bounds c385a17 fixed) and #4 on the dbsp advisories (its own task); `@dependabot rebase` or the weekly run re-bases them, then they merge green.
- 2026-11-02T17:26Z [claude (de9bd423-d8b5-4cf3-8a2c-44d698f4cc3e)] 2026-11-02 the settings list, re-read after the release process landed (main = dev = 35d0a77, default branch dev, set by the session with gh api). Every item is the owner's act; the first is the one the release PR waits on today:
  
  1. Settings → Secrets → RELEASE_PLZ_TOKEN: a GitHub App installed on this repo with Contents and Pull requests read/write (recommended: the release PR is then the app's and runs ci), or a fine-grained PAT with the same two permissions on this repo. Without it the release-pr job ends at 403 "GitHub Actions is not permitted to create or approve pull requests" (run 41490353635). The alternative, Settings → Actions → General → "Allow GitHub Actions to create and approve pull requests", opens the PR with GITHUB_TOKEN, and that PR runs no ci. After either, re-run that job or push to dev: the release PR "chore: release v0.1.0" opens against dev.
  2. Settings → General: visibility public (attestations are public-repo-only on a Free plan, rulesets too; public runners are free, which ends the fuzz workflow's spending-limit refusals). Pull Requests: allow merge commits only, squash and rebase off (release-plz tags the merged PR's last commit and GitHub does not sign rebase-merged commits; a squash would make the next dev→main PR replay history).
  3. Rules → Rulesets, after the flip: main (pull request required, source branch dev, required checks: ci's gate, msrv, deny and commits jobs plus golden's targets; no force-push, no deletion); dev (no force-push, no deletion); tags v* (no update, no deletion), the ruling's replacement for a signed tag.
  4. Settings → Code security: Private vulnerability reporting on (SECURITY.md names it), Dependabot alerts on, secret scanning with push protection confirmed.
  5. Settings → Actions → General: "Require actions to be pinned to a full-length commit SHA" on (the standing decision as a server-side check); "Require approval for all external contributors" on fork PRs; workflow permissions stay read-only.
  6. Settings → Environments → release with the owner as required reviewer: release.yml's crates.io dispatch names it; nothing in the release process does, so this waits for the day crates.io joins.
  7. About: topics (rust, no-std, floating-point, reproducibility, differential-dataflow, dbsp).
  8. Dependabot PRs #1–#4 now base dev; #2 and #3 carry 'Bump' titles the commits job refuses (fl-hp7md5n).
- 2026-11-02T17:51Z [claude (de9bd423-d8b5-4cf3-8a2c-44d698f4cc3e)] 2026-11-02, correcting item 1 of the list above after the rebuild without release-plz (fl-05nqrwd): the secret is RELEASE_PR_TOKEN (a GitHub App installed on this repo with Contents and Pull requests read/write, or a fine-grained PAT with the same), read by .github/workflows/cut-release.yml's release-pr job; without it the job's PR step ends at the same 403 unless Settings → Actions → General allows Actions to create pull requests (then the PR opens with GITHUB_TOKEN and runs no ci). Everything else on the list stands.
- 2026-11-02T18:05Z [claude (de9bd423-d8b5-4cf3-8a2c-44d698f4cc3e)] 2026-11-02, end of session: ci runs on pull requests and main only, a push to dev runs nothing, and the release PR is opened by `gh workflow run cut-release.yml --ref dev` on the owner's word (owner: "A PUSH IS JUST A PUSH"; "DOING THAT WHEN I TELL YOU TO BUILD A PR"). Item 1 stands (RELEASE_PR_TOKEN or the Actions setting) and is what the dispatch needs. Two ci runs were still in progress when the session ended and the session could not cancel them: 41495939738 (dev) and 41491375969 (main); `gh run cancel <id>` stops each.
- 2026-11-02T18:34Z [claude (227b40e5-20bd-44b0-9b79-d44c1a3c3587)] 2026-11-02, fl-ezhk4cz at 4bec2d5: item 1's workflow is .github/workflows/release.yml (from cut-release.yml), release-pr job; the dispatch is `gh workflow run release.yml --ref dev`. Read this session: `gh api repos/jbrjake/float-abel/rulesets` and `.../branches/main/protection` both answer 403 'Upgrade to GitHub Pro or make this repository public to enable this feature', so until the flip no required check, no PR-only main and no v* tag ruleset can exist; the dev→main PR's checks are what the owner reads before merging by hand. ci.yml's check names changed with the merge: the golden jobs are 'golden <target>' and the fuzz jobs 'fuzz <target> (10 min)', plus 'code changed since the last nightly'; the required-checks list on the ruleset, when it can be set, names those.
- 2026-11-02T20:06Z [claude (987c488d-43f3-4192-b0e1-6b279735b89c)] 2026-11-02, fl-ebbxwd2 at c9aaa03: item 1 of the settings list (RELEASE_PR_TOKEN, or Settings → Actions → General → "Allow GitHub Actions to create and approve pull requests") is struck. release.yml has no release-pr job and no workflow_dispatch; `scripts/release-pr.sh` cuts the release PR from a clone under the developer's own gh login, and no secret or Actions toggle is wanted for the release (owner: "why do we need a workflow for this? why can't it be something done from a clone using gh?"; "OF COURSE FIX IT!"). Items 2 to 8 stand as written. The branch release/next on origin at ae1c010 is the runner's "chore(release): v0.30.1" from run 41495586562; the script's first real run force-pushes over it.
