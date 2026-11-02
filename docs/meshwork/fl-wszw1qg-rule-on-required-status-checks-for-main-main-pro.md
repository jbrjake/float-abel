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
---
Found 2026-11-02 reading `gh api repos/jbrjake/float-abel/rules/branches/main`: deletion, non_fast_forward, pull_request, and nothing else. fl-k594v2g's body planned "a ruleset on main requiring a pull request, the ci gate, msrv and deny jobs"; fl-4fcbtkx's 18:34Z comment named the check names a required-checks list would carry ('smoke + regression (stable)', 'MSRV 1.74 on the default features', 'cargo-deny', 'golden <target>', 'fuzz <target> (10 min)', 'code changed since the last nightly'). Today the owner reads the checks by hand before merging, which fl-4fcbtkx recorded as the interim.

The owner rules which jobs, if any, hold the merge. Note the probes: after fl-fjac8jd, miri and the fuzz jobs are skipped on a PR whose code a green run covered, and a skipped job satisfies a required check in a ruleset only when it is reported as skipped (GitHub treats skipped as passing for required checks); the golden macOS leg bills whenever it runs and is kept by ruling. A required_status_checks rule with strict_required_status_checks_policy false names each check by its job name and lists the integration id of GitHub Actions (15368). Done is the owner's list, the rule added, and CLAUDE.md's release decision naming the required checks.

## log
- 2026-11-02T21:28Z created
