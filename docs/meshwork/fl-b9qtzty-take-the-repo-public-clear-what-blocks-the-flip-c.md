---
id: fl-b9qtzty
title: "Take the repo public: clear what blocks the flip, cut what the flip exposes, and hand the owner the settings to set at it"
category: crate/release
labels: [reproducible-floats]
seq: 15
relates: [fl-fm78m8d]
docs:
  - docs/DESIGN.md#release-sp-release
  - CLAUDE.md#standing-decisions
verify: contains docs/meshwork/fl-4fcbtkx-push-main-to-origin-and-make-the-repo-readable-b.md /2026-[0-9][0-9]-[0-9][0-9] owner pushed main to origin/
status: open
created: 2026-11-02T15:14Z
---
The umbrella for the 2026-11-02 review (owner: "what else do we have to do before making this repo
public?"; "WE NEED A TASK TREE TO PREP FOR THIS RELEASE"). The children are what the review found;
fl-4fcbtkx is the flip itself and carries the GitHub-settings checklist as its 2026-11-02 comment,
fl-r0s86hg the advisories that fail the push gate today. The 0.1.0 release umbrella fl-fm78m8d is
the next tree after this one: publishing is not visibility.

October's Actions bill, reconstructed from job durations (each job rounded up to the minute, Linux
×1, Windows ×2, macOS ×10; the account is on Free, 2,000 minutes a month, and ran out on the 30th):
2,976 weighted minutes over 73 runs and 307 jobs. golden's macOS leg 1,170 (17 jobs, 117 raw), ci
1,039 (85 jobs, all Linux), fuzz 576 (96 jobs), golden's six other legs 184. main did not move after
f24b051 on 2026-10-20; twelve nightly fuzz runs and four dependabot PR runs happened anyway.

## log
- 2026-11-02T15:14Z created
