---
id: fl-jkb64s3
title: "Run the release scripts' self-tests in both gates: version-from-log.sh, changelog-entry.sh and release-pr.sh beside check-commit-message.sh's"
category: crate/release
labels: [reproducible-floats]
seq: 14
parent: fl-m3cymxv
relates: [fl-ebbxwd2]
docs:
  - CLAUDE.md#commands
verify: "all(contains scripts/smoke.sh version-from-log.sh, contains scripts/smoke.sh changelog-entry.sh, contains scripts/smoke.sh release-pr.sh, contains scripts/regression.sh version-from-log.sh, contains scripts/regression.sh changelog-entry.sh, contains scripts/regression.sh release-pr.sh)"
status: open
created: 2026-11-02T19:55Z
---
Found filing @clone: `scripts/version-from-log.sh --self-test` and `scripts/changelog-entry.sh --self-test` run in neither scripts/smoke.sh nor scripts/regression.sh; only `check-commit-message.sh --self-test` does (smoke.sh and regression.sh, unconditionally). The version the release carries is counted by the first and its notes rendered by the second, so a regression in either reaches the release PR before anything runs the fixtures. ci.yml does not run them either.

The fix: in smoke.sh, the three release scripts' `--self-test` under the leg that runs when `scripts/*` is in the diff (`rust=1`), since release-pr.sh's fixture builds a git repository and runs `cargo update` and the commit gate is scoped to the diff; in regression.sh, all three beside check-commit-message's, unconditionally. Each self-test prints `<script>: self-test OK` and exits non-zero on a failed fixture, so the gate fails closed.

## log
- 2026-11-02T19:55Z created
