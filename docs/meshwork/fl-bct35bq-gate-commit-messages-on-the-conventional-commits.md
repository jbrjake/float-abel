---
id: fl-bct35bq
title: "Gate commit messages on the Conventional Commits form in smoke.sh and the PR title in ci, because the version is derived from them"
category: crate/hardening
labels: [reproducible-floats]
seq: 18
parent: fl-m3cymxv
needs: [fl-cnkjdzp]
docs:
  - CLAUDE.md#hygiene
verify: exists scripts/check-commit-message.sh
status: open
created: 2026-11-02T15:33Z
---
A derived version is only as good as the log it reads. `scripts/check-commit-message.sh` takes a
message file or a range and refuses anything not of the form `type(scope)?: subject` with the
baseline's types (feat, fix, docs, test, ci, chore, build, refactor, perf) and an imperative
subject under 72 characters; the `.githooks/commit-msg` hook runs it, smoke.sh runs it over the
staged commit's message where it can, and a ci step runs it over the PR's commits and title. Red
first: a fixture of bad subjects the script must reject.

## log
- 2026-11-02T15:33Z created
