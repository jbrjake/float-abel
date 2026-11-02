---
id: fl-k594v2g
title: "Create dev from main and make it the default branch; rule main PR-only from dev with the required checks; point the hook flow and CLAUDE.md at dev"
category: crate/release
labels: [reproducible-floats]
seq: 14
parent: fl-m3cymxv
needs: [fl-cnkjdzp]
docs:
  - CLAUDE.md#commands
verify: contains CLAUDE.md /git push origin dev/
status: done
created: 2026-11-02T15:33Z
---
`git branch dev main && git push origin dev` from the checkout, then the owner's settings: default
branch `dev`; a ruleset on `main` requiring a pull request, the ci gate, msrv and deny jobs and the
golden targets as required checks, no force-push, no deletion, and (the owner's pick) restricting
the source branch to `dev`; a ruleset on `dev` blocking force-push and deletion. The pre-push hook
keeps running the regression gate on every push to `dev`. CLAUDE.md § Commands names `git push
origin dev` and the release cut as a PR; the sessions' memory of the push recipe moves with it.
Rulesets need the repo public or Pro, so this sits behind the flip (fl-4fcbtkx) in practice.

## log
- 2026-11-02T15:33Z created
- 2026-11-02T16:43Z open→doing — claimed by claude (de9bd423-d8b5-4cf3-8a2c-44d698f4cc3e)
- 2026-11-02T17:05Z doing→done — verify exit 0 @ a2cf3df+1

## comments
- 2026-11-02T17:05Z [claude (de9bd423-d8b5-4cf3-8a2c-44d698f4cc3e)] 2026-11-02 done from the checkout: dev created from main and pushed (both at a2cf3df) through the push gate, the default branch set to dev with gh api (a release-plz release-pr targets the default branch and has no flag for it), ci runs on pushes to dev and main, release-plz.yml refuses the release PR while the default branch is not dev, CLAUDE.md § Commands names git push origin dev and the dev→main PR. Left for the owner, since rulesets answer 403 on a private Free repo until the flip: a ruleset on main (pull request required, source branch dev, merge commits only, required checks ci gate/msrv/deny and golden's targets, no force-push, no deletion), one on dev (no force-push, no deletion), and one on v* tags (no update, no deletion); listed with the rest of the flip settings on fl-4fcbtkx.
