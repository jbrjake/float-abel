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
  Next session, in this order; the owner's standing word (2026-11-02) is
  one or two leaves a session,
  then pause.
  
  1. ci run 41472015430 on main@682db04 (the advisories push). If green,
  `close fl-r0s86hg` (its
  verify is already green; the run was the last done-criterion). If the
  dbsp job is red, read its
  deny step first: the lock has ctutils 0.4.7 and actix-http 3.20.0,
  deny-dbsp.toml one new row
  (RUSTSEC-2026-0403), and both gates run `-D advisory-not-detected`.
  2. fl-cnkjdzp, the ruling on the release process: needs the owner in
  session. The readback is
  fl-m3cymxv's body (release-plz, release PR against dev, the dev→main
  merge cuts the release,
  attested .crate on `release: published`, crates.io on the owner's
  dispatch, no version ever
  typed). Three questions in the task. Write the owner's words, dated, as
  the comment the verify
  reads, and rewrite DESIGN.md § Release to the ruled shape. Everything
  under fl-m3cymxv waits on
  it.
  3. Leaves that need no ruling, pick one or two: fl-c8jpmpm (fuzz
  schedule; full brief in its
  handoff), fl-cqmcs7c (gate miri/dbsp/golden on a Rust-reaching diff),
  fl-5mywkqp (restore/save
  split so red jobs still cache), fl-fs9vhnd (top-level `permissions:
  contents: read`).
  4. The owner's acts, surfaced not done: the flip and its seven settings
  (fl-4fcbtkx's 2026-11-02
  comment), `@dependabot rebase` and the four merges (fl-hp7md5n, once ci
  is green on main).
  
  Facts to carry: the account is on GitHub Free, so rulesets, environments
  and attestation all wait
  on the flip; caches evict after seven idle days and save only from green
  jobs; the miri job is
  41–57 minutes of interpretation and the long pole of every run.
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
