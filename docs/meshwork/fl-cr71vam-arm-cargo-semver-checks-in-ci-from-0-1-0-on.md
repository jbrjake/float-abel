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
---

DESIGN.md § Release once said cargo-semver-checks is armed from the first release on. The version is now a count over the commits (fl-05nqrwd), so a semver check cannot change a number; what it can do is fail a PR whose commits claim no break while the API broke. That is a new tool in the build (cargo-semver-checks, and an action or a hash-pinned tarball to install it), and the owner ruled the build carries no dependency it does not need ("THIS IS A MINIMALIST PROJECT THAT IS SUPPOSED TO HAVE NO DEPENDENCIES IT DOESN'T NEED"), so this is the owner's call before anything is built: a ci job on pull requests running cargo semver-checks check-release against the last release tag as the baseline, or nothing. Ask, do not build.

## log
- 2026-10-20T15:04Z created
