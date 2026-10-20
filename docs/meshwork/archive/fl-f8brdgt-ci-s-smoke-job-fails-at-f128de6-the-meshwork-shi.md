---
id: fl-f8brdgt
title: "CI's smoke job fails at f128de6: the meshwork shim execs a v0.5.3 binary the runner never installs (exit 127)"
status: done
category: ci/smoke
created: 2026-10-20T00:33Z
verify: contains scripts/smoke.sh CI:-
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
- 2026-10-20T15:44Z doing→done — verify exit 0 @ 2ac0f3e+1

## comments
- 2026-10-20T15:44Z [claude (4a8ded2c-6eb4-4638-a7f1-ee6da67e86ba)] 2026-10-20 fixed at ef90777 (owner: "not running meshwork calls in ci to avoid the dependency altogether"): smoke.sh skips the store lint under CI, loudly. Green on the runner: ci run 39449087008 at 79cf25b, gate job 118718750192 success in 12m18s, its log reading 'smoke: SKIPPED meshwork lint (CI never installs meshwork; the store is linted at commit time)', 'smoke: OK (tree diff; 239 tests passed)' and 'regression: OK'. Before the fix the same step died in 15 s with exit 127 (run 38957258898). Reproduced locally first: HOME=/nonexistent CI=true ./scripts/smoke.sh exited 126 on the shim's exec, then printed the SKIPPED line after the change. The run's miri job is fl-jphvxrs's and the dbsp job's state on this run is noted there.
