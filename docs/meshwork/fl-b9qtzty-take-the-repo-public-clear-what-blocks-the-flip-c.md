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
handoff: |
  2026-11-02, dev at the commit this handoff was committed in: the
  workflows are three, ci.yml (the suite, nightly on dev only when dev
  moved outside docs/meshwork/, and on the PR into main), release.yml
  (one job, the tag and GitHub Release with the attested .crate on the
  push to main; no dispatch, no release-pr job) and publish.yml
  (crates.io, dormant). The release PR is cut from a clone by
  `scripts/release-pr.sh` under the developer's own gh login (fl-ebbxwd2,
  owner: "OF COURSE FIX IT!"), so the release needs no secret and no
  Actions toggle; `--dry-run` printed 0.30.1 and its entry this session.
  The three release scripts' self-tests run in both gates (fl-jkb64s3).
  Every action sits at its current upstream pin; the pins' first run is
  the next nightly. The release process now waits on the owner's word for
  `./scripts/release-pr.sh`, the flip and two merges (fl-enkdnvk carries
  the brief, fl-4fcbtkx the settings from item 2 on, fl-9hcnr6q the
  ruling on one PR or two). Open here besides: fl-c8jpmpm (fuzz corpus
  and cargo-fuzz cache), fl-5mywkqp (dbsp cache save; cache v6.1.0 logs a
  policy write denial as a warning, so read the nightly's dbsp post-cache
  step first), fl-wpgpd4q (miri hour), fl-9rczhm8, fl-821gnyr, fl-jdsha55,
  fl-cr71vam.
---

The one tree for getting the project in front of other people (owner, 2026-11-02: "what else do we
have to do before making this repo public?"; "WE NEED A TASK TREE TO PREP FOR THIS RELEASE"; "DON'T
BUILD A SECOND SET OF TASKS THAT HAVE TO BE TRACKED AND MANAGED SEPARATELY WHEN THIS ALL LEADS TO
THE SAME GOAL"). Every child is on that path: fl-4fcbtkx is the flip itself and carries the
GitHub-settings checklist as its 2026-11-02 comment; fl-r0s86hg the advisories that failed the push
gate; the CI cost and duration leaves; fl-m3cymxv the release process (branches, release-plz, the
attested .crate, the first cut), which absorbed the former release umbrella fl-fm78m8d and its
children; and the doc-truth, hygiene and miri-ceiling tasks that were loose in the store.

October's Actions bill, reconstructed from job durations (each job rounded up to the minute, Linux
×1, Windows ×2, macOS ×10; the account is on Free, 2,000 minutes a month, and ran out on the 30th):
2,976 weighted minutes over 73 runs and 307 jobs. golden's macOS leg 1,170 (17 jobs, 117 raw), ci
1,039 (85 jobs, all Linux), fuzz 576 (96 jobs), golden's six other legs 184. main did not move after
f24b051 on 2026-10-20; twelve nightly fuzz runs and four dependabot PR runs happened anyway.

## log
- 2026-11-02T15:14Z created
- 2026-11-02T15:37Z handoff by claude (c90f1eae-8c6b-41ec-aa59-9d0990f41bc2)
- 2026-11-02T17:26Z handoff by claude (de9bd423-d8b5-4cf3-8a2c-44d698f4cc3e)
- 2026-11-02T18:05Z handoff by claude (de9bd423-d8b5-4cf3-8a2c-44d698f4cc3e)
- 2026-11-02T18:36Z handoff by claude (227b40e5-20bd-44b0-9b79-d44c1a3c3587)
- 2026-11-02T19:29Z handoff by claude (55cb23a1-b965-4570-9fea-74159b1286bc)
- 2026-11-02T20:06Z handoff by claude (987c488d-43f3-4192-b0e1-6b279735b89c)
