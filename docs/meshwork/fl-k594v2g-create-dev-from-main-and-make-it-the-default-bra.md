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
status: open
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
