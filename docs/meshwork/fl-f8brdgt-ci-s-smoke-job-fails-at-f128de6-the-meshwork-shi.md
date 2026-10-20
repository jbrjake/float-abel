---
id: fl-f8brdgt
title: "CI's smoke job fails at f128de6: the meshwork shim execs a v0.5.3 binary the runner never installs (exit 127)"
status: doing
category: ci/smoke
created: 2026-10-20T00:33Z
verify: contains scripts/smoke.sh CI:-
claimed-by: claude (4a8ded2c-6eb4-4638-a7f1-ee6da67e86ba)
---

The `ci` workflow's `smoke + regression (stable)` job fails at `f128de6` (run 38957258898,
2026-10-16T19:47Z) with exit 127:

    docs/meshwork/meshwork: 10: exec: /home/runner/.meshwork/versions/v0.5.3/meshwork: not found

`scripts/smoke.sh` runs `docs/meshwork/meshwork lint` whenever the diff touches the store. Its guard,
`[ -x docs/meshwork/meshwork ]`, tests the shim, which is executable on the runner, and not the binary
the shim execs, which the runner never installs. So the SKIPPED branch never fires and the job dies
before the cargo legs. `golden` is green on the same commit (run 38957258897), and the dependabot
PRs' `ci` runs fail as well (their cause not read).

Two shapes fix it: the guard tests the binary the shim resolves (and skips loudly when it is absent),
or the job installs the pinned meshwork before smoke runs. The close is a green `ci` run on the fixing
commit.

## log
- 2026-10-20T00:33Z created
- 2026-10-20T14:58Z open→doing — claimed by claude (4a8ded2c-6eb4-4638-a7f1-ee6da67e86ba)
