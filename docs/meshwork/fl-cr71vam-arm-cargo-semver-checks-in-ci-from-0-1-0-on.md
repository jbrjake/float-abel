---
id: fl-cr71vam
title: Arm cargo-semver-checks in CI from the first release on
category: crate/release
labels: [reproducible-floats]
seq: 190
discovered-from: fl-k3g1511
docs:
  - docs/DESIGN.md#release-sp-release
verify: contains .github/workflows/ci.yml semver-checks
status: open
created: 2026-10-20T15:04Z
parent: fl-m3cymxv
needs: [fl-enkdnvk]
handoff: |
  2026-11-02, end of the session that cut and saw through the first
  release
  (fl-enkdnvk closed; read its last two comments for the release's
  record),
  for a fresh session. The owner's last word: "wrap up once you've got
  this
  stuff done, not start new work", so nothing here was started.
  
  State on origin: main at 74ed9ee, the merge of #5, tagged v0.30.2 with
  the
  GitHub Release, the attested .crate and its sha256; dev at b99611e, four
  past the release commit (the probe gate 9b7acc3 and store commits) plus
  whatever this session's last commit is (see git log). main-protection
  requires a pull request, merge commits, no approving review (owner set
  0). The probe gate (fl-fjac8jd, scripts/probes-due.sh) has not yet run
  on
  GitHub: its first live run is the next pull request into main, where the
  changes job's probes step says whether miri and the fuzz targets run and
  why. The nightly has never fired yet (no schedule event on ci.yml so
  far);
  the first will be 03:17 UTC on dev if code moved.
  
  This task: cargo-semver-checks is a tool the build does not have, and
  the
  crate's posture is no dependency the build does not need (owner: "THIS
  IS
  A MINIMALIST PROJECT THAT IS SUPPOSED TO HAVE NO DEPENDENCIES IT DOESN'T
  NEED"; the release runs on git, cargo, gh and two GitHub-owned actions).
  Arming it means an installer or an action in ci.yml; read that back to
  the owner and get their word before adding anything. If they say yes:
  the
  baseline to compare against is the v0.30.2 .crate on the Release; the
  job
  belongs in ci.yml beside cargo-deny, pinned by hash or tarball digest
  like
  the golden jobs' tools, and gated on the suite's `run` output.
  
  Open beside it, none started: fl-wszw1qg (required checks on main, a
  ruling), fl-wh7r2df (branch-protection's include is the string
  "refs/heads/main dev" and reaches no branch; two patterns fix it, on the
  owner's word), fl-pg35w24 (no v* tag ruleset exists; CLAUDE.md says one
  pins v* tags), fl-7en8z60 (crates.io, blocked on the owner's word; do
  not
  re-offer), fl-jdsha55 (dead zstd skip rows), the CI-cost tasks
  fl-c8jpmpm, fl-5mywkqp, fl-wpgpd4q, fl-btst4df, and leras's inbound ask
  le-y8jn8md (TryFrom<&Exact> for Fixed), six days old. The store lint
  warns that fl-b9qtzty's verify reads fl-4fcbtkx's task file, which is
  archived; re-point it or ask.
---

DESIGN.md § Release once said cargo-semver-checks is armed from the first release on. The version is now a count over the commits (fl-05nqrwd), so a semver check cannot change a number; what it can do is fail a PR whose commits claim no break while the API broke. That is a new tool in the build (cargo-semver-checks, and an action or a hash-pinned tarball to install it), and the owner ruled the build carries no dependency it does not need ("THIS IS A MINIMALIST PROJECT THAT IS SUPPOSED TO HAVE NO DEPENDENCIES IT DOESN'T NEED"), so this is the owner's call before anything is built: a ci job on pull requests running cargo semver-checks check-release against the last release tag as the baseline, or nothing. Ask, do not build.

## log
- 2026-10-20T15:04Z created
- 2026-11-02T22:05Z handoff by claude (aa073407-90a0-459f-92ea-439fb58eb49c)
