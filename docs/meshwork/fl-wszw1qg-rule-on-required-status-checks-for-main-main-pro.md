---
id: fl-wszw1qg
title: "Rule on required status checks for main: main-protection has no required_status_checks rule, so a red suite on the dev→main PR does not hold the merge button; fl-k594v2g planned the gate, MSRV and deny jobs as required"
category: rulings
labels: [reproducible-floats]
parent: fl-b9qtzty
relates: [fl-enkdnvk]
seq: 20
verify: contains CLAUDE.md /required checks/
docs:
  - CLAUDE.md#standing-decisions
status: open
created: 2026-11-02T21:28Z
handoff: |
  State at 2026-11-25, dev at 99b943b plus store commits, pushed; the
  inbox
  is empty. Two calls of the 2026-11-24 session wait on the owner's word:
  the task to cache the cargo-fuzz binary was dropped on a measurement
  (the
  install is 16 s), and the dbsp cache task was closed with its verify
  waived, on its done line and without the restore/save split. Neither is
  ruled on; `search "cargo-fuzz binary"` and `search "done line,
  observed"`
  find them, and each says how to reopen.
  
  This task and the two behind it, fl-wh7r2df and fl-pg35w24, are the
  owner's: a ruling and two rulesets on the owner's repository. Put them
  to
  the owner; build nothing on them. The repository is public, so nothing a
  ruleset costs is billed.
  
  The first thing to read is the first suite run after 99b943b, the
  nightly
  (it fires between 11:00 and 14:00 UTC, whatever the cron says):
  
  - each fuzz leg should log `Cache hit for restore-key:
  corpus-<target>-45512975046-` and `seed corpus: files: N` with N over
  zero; the one restore watched so far was a re-run inside one run;
  - the miri job's install step prints `nproc`, which is step one of
  fl-wpgpd4q and has not been seen on a runner.
  
  The next task a session can build is fl-wpgpd4q, step two: its last
  comment carries the brief and the miri job's times.
  
  Open beside it, for the owner and not to be built on: fl-jjy5xt9 (the
  heap
  word's integer-to-pointer cast under miri; the bitrep test now reaches
  the
  heap form under miri and prints that warning too) and fl-ng7bs72 (the
  SIMD
  kill condition; retake `cargo bench --bench simd` on a quiet Mac before
  asking). `bench-baseline.json` is not re-recorded for the faster
  one-value
  deposit.
---
Found 2026-11-02 reading `gh api repos/jbrjake/float-abel/rules/branches/main`: deletion, non_fast_forward, pull_request, and nothing else. fl-k594v2g's body planned "a ruleset on main requiring a pull request, the ci gate, msrv and deny jobs"; fl-4fcbtkx's 18:34Z comment named the check names a required-checks list would carry ('smoke + regression (stable)', 'MSRV 1.74 on the default features', 'cargo-deny', 'golden <target>', 'fuzz <target> (10 min)', 'code changed since the last nightly'). Today the owner reads the checks by hand before merging, which fl-4fcbtkx recorded as the interim.

The owner rules which jobs, if any, hold the merge. Note the probes: after fl-fjac8jd, miri and the fuzz jobs are skipped on a PR whose code a green run covered, and a skipped job satisfies a required check in a ruleset only when it is reported as skipped (GitHub treats skipped as passing for required checks); the golden macOS leg bills whenever it runs and is kept by ruling. A required_status_checks rule with strict_required_status_checks_policy false names each check by its job name and lists the integration id of GitHub Actions (15368). Done is the owner's list, the rule added, and CLAUDE.md's release decision naming the required checks.

## log
- 2026-11-02T21:28Z created
- 2026-11-25T00:43Z handoff by claude (4aebe618-581f-48ef-9ee8-943721730ef9)
- 2026-11-25T00:43Z handoff by claude (4aebe618-581f-48ef-9ee8-943721730ef9)
