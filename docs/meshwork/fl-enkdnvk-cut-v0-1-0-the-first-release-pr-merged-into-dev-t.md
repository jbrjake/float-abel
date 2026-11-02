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
  2026-11-02, the session that cut the release pull request, for the
  session
  that sees it merged. Read the comments here (what was cut, why the count
  is
  0.30.2) and fl-fjac8jd's last comment (the probe gate) before anything
  else.
  
  The owner's words this session, in force: "at this point i don't want
  another release today. the one that's running will be the first release
  when its fuzz completes. don't update the release pr or make a new pr."
  
  State on origin: dev at 249fdad, `chore(release): v0.30.2`, the release
  commit scripts/release-pr.sh made; main at 3ac2723; pull request #5
  open,
  dev → main, title `chore(release): v0.30.2`, body the entry; ci run
  41524208877 on 249fdad, the whole suite (`gh pr checks 5`; `gh run view
  41524208877`). No tag past v0.0.0.
  
  This clone's dev is three commits past origin/dev, NOT pushed, on
  purpose:
  7009a5e (store), 9b7acc3 `ci(probes): run miri and the fuzz targets on
  the
  pull request only when no green run covers the code` (fl-fjac8jd, with
  scripts/probes-due.sh and its self-test in both gates), and the store
  commit closing fl-fjac8jd. A push to dev while #5 is open moves the pull
  request's head and reruns its suite, which is what the owner said not to
  do. All three are chore or ci commits: the count stays 0.30.2, so they
  would not break release.yml's version check, but they wait.
  
  What remains, in order:
  1. The suite on #5 is green: run 41524208877 concluded success at 21:23Z
  on all seventeen jobs. #5 is BLOCKED by the main-protection ruleset's
  pull_request rule alone: required_approving_review_count 1, no bypass
  actor, one account on the repo, so nobody can merge. fl-494d9cb holds
  the ruling (0 reviews recommended, or an admin bypass actor); the PUT
  body for 0 is one parameter off the live ruleset and sits unsent in
  the session's scratchpad (rebuild it: GET the ruleset, move
  required_approving_review_count to 0, PUT name, target, enforcement,
  bypass_actors, conditions, rules). On the owner's word, send it, then
  `gh pr view 5 --json mergeStateStatus` should read CLEAN. Three
  sibling findings under fl-b9qtzty: fl-wszw1qg (no required checks on
  main), fl-wh7r2df (branch-protection reaches no branch), fl-pg35w24
  (no v* tag ruleset; the release job's tag creation is free today).
  If a job had to be fixed on dev before the merge: a push moves #5, and
  a `fix` commit moves the count to 0.30.3, after which
  scripts/release-pr.sh must run again before the merge (release.yml's
  first step refuses a Cargo.toml that disagrees with the count). The
  owner's call, not yours.
  2. The owner merges #5 (merge commit; the repo allows no other). Do not
  merge it yourself. The push to main runs release.yml's one job:
  version check, entry read, `cargo package --locked`, sha256, the
  attestation (actions/attest-build-provenance), then `gh release create
  v0.30.2` with the .crate and .sha256 attached. Watch it: `gh run list
  --workflow release.yml`, `gh run view <id>` to conclusion=success. The
  attest step has never run on this repo; if it fails, the failure is
  the finding, and no tag is made (the tag comes last).
  3. From a clean directory: `gh release download v0.30.2 --repo
  jbrjake/float-abel --pattern '*.crate*'`, check the sha256 against the
  .sha256, then `gh attestation verify float-abel-0.30.2.crate --repo
  jbrjake/float-abel`. Comment the output here, with the release URL.
  4. `close fl-enkdnvk` (the verify is the 0.30.2 heading in CHANGELOG.md,
  already green). fl-cr71vam unlocks.
  5. Then, and only then, `git push origin dev` (the push gate runs, ~4
  min
  warm, sandbox off for cargo-deny). That lands the probe gate; its
  first live run is the next pull request into main, where the changes
  job's probes step says whether miri and fuzz run and why.
  6. Do not re-offer crates.io; fl-7en8z60 is blocked on the owner's word
  (owner: "I WILL TELL YOU WHEN I WANT IT"). Do not add a tool.
  
  Left open on purpose: fl-jdsha55 (the two dead zstd skip rows in
  deny-dbsp.toml; warnings, not a gate failure), and the CI-cost tasks
  fl-c8jpmpm, fl-5mywkqp, fl-wpgpd4q, fl-btst4df; none changes the
  released
  artifact. The store lint warns that fl-b9qtzty's verify reads
  fl-4fcbtkx's task file, which is archived; not touched.
---

The owner's merges, twice, and the session's watch. The release PR cut-release.yml keeps against dev carries the version the log counts to (owner: "YOU NEED TO BE COUNTING FROM 0.0.0 ON FROM _THE VERY FIRST COMMIT_"), Cargo.lock and the CHANGELOG entry; no version is typed anywhere by hand. The owner merges it (merge commit); the session opens dev→main (`gh pr create --base main --head dev --title "chore(release): cut the release the log yields"`) before anything else lands on dev; the owner merges it; the release job tags `v<that version>` on the merge commit, creates the GitHub Release with the entry as notes, builds the .crate with --locked, attests it and uploads .crate and .sha256. `gh attestation verify float-abel-<version>.crate --repo jbrjake/float-abel` passes from a clean machine. crates.io stays out until the owner says. Done is the verify (a released version heading in CHANGELOG.md) plus the release URL and the verify command's output in a comment here.

## log
- 2026-11-02T15:33Z created
- 2026-11-02T17:26Z handoff by claude (de9bd423-d8b5-4cf3-8a2c-44d698f4cc3e)
- 2026-11-02T17:51Z handoff by claude (de9bd423-d8b5-4cf3-8a2c-44d698f4cc3e)
- 2026-11-02T18:05Z handoff by claude (de9bd423-d8b5-4cf3-8a2c-44d698f4cc3e)
- 2026-11-02T20:06Z handoff by claude (987c488d-43f3-4192-b0e1-6b279735b89c)
- 2026-11-02T20:43Z handoff by claude (987c488d-43f3-4192-b0e1-6b279735b89c)
- 2026-11-02T21:02Z handoff by claude (aa073407-90a0-459f-92ea-439fb58eb49c)
- 2026-11-02T21:21Z handoff by claude (aa073407-90a0-459f-92ea-439fb58eb49c)
- 2026-11-02T21:30Z handoff by claude (aa073407-90a0-459f-92ea-439fb58eb49c)

## comments
- 2026-11-02T18:34Z [claude (227b40e5-20bd-44b0-9b79-d44c1a3c3587)] 2026-11-02, fl-ezhk4cz at 4bec2d5: the release workflow is .github/workflows/release.yml (from cut-release.yml); the owner's dispatch is `gh workflow run release.yml --ref dev`. The release job now packages and attests the .crate before creating the tag and the GitHub Release, and attaches the .crate and its sha256 in the `gh release create` call, so a failed package leaves no tag behind. The release PR into dev runs no ci (ci.yml triggers on PRs into main alone); the dev→main PR runs the whole suite, fuzz and golden included. The RELEASE_PR_TOKEN or the Actions setting is still what the dispatch needs: `gh secret list` is empty and `can_approve_pull_request_reviews` is false at this writing.
- 2026-11-02T21:02Z [claude (aa073407-90a0-459f-92ea-439fb58eb49c)] 2026-11-02, from this clone, on the owner's "cut the release pr" and "if there are things still to do i guess do them": fl-rhztsdq landed first, 21f46c1 `fix(release): anchor the include globs to the crate root`, because it changes what the first .crate carries. `cargo package --list --locked` went from 49 files to 47, the two tests/data/bitrep paths gone and the six root files kept; the fix moved the count from 0.30.1 to 0.30.2. The push gate passed the tree at 530c71d (regression: OK, the two unmatched-skip warnings of fl-jdsha55 printed and ignored, as warnings are). `./scripts/release-pr.sh` then committed `chore(release): v0.30.2` as 249fdad (CHANGELOG.md +138 lines, Cargo.toml and Cargo.lock one line each), pushed dev with the hooks skipped and said so, and opened https://github.com/jbrjake/float-abel/pull/5, dev → main, title `chore(release): v0.30.2`, body the 138-line entry opening with `## [0.30.2](…/compare/v0.0.0...v0.30.2) - 2026-11-02`. ci run 41524208877 started on 249fdad: Conventional Commits and the code-changed gate passed, fifteen jobs pending (smoke + regression, MSRV, miri, cargo-deny, dbsp and ui on 1.99, three fuzz, seven golden); the pull request's mergeStateStatus is BLOCKED until they pass. main stays at 3ac2723. This task's verify is green from here on (the 0.30.2 heading in CHANGELOG.md), but the task is done only once the owner merges, release.yml tags and attests, and `gh attestation verify` passes from a clean download, so it stays open.
- 2026-11-02T21:30Z [claude (aa073407-90a0-459f-92ea-439fb58eb49c)] 2026-11-02T21:23Z: ci run 41524208877 on 249fdad concluded success, all seventeen jobs (miri 20:59 -> 21:23, 24 min; the three fuzz targets, dbsp, the gate, MSRV, cargo-deny and the seven golden legs green). #5 is still BLOCKED: reviewDecision REVIEW_REQUIRED, from the main-protection ruleset's pull_request rule (required_approving_review_count 1, bypass_actors []), which no account on this one-account repo can satisfy, the author's approval never counting. Filed fl-494d9cb (the ruling: 0 reviews, or an admin bypass actor), fl-wszw1qg (no required_status_checks rule on main), fl-wh7r2df (branch-protection's include is the single string 'refs/heads/main dev' and reaches no branch) and fl-pg35w24 (no v* tag ruleset exists). The PUT body for 0 reviews is prepared, one parameter different from the live ruleset, and not sent: a ruleset is the owner's. The owner's word this session stands: no second release today, #5 untouched.
