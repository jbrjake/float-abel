---
id: fl-fs9vhnd
title: "Set every workflow's token to read-only at the top: permissions: contents: read in ci.yml, fuzz.yml and golden.yml"
category: crate/hardening
labels: [reproducible-floats]
seq: 50
parent: fl-b9qtzty
docs:
  - CLAUDE.md#standing-decisions
verify: "all(contains .github/workflows/ci.yml /^permissions:/, contains .github/workflows/fuzz.yml /^permissions:/, contains .github/workflows/golden.yml /^permissions:/)"
status: open
created: 2026-11-02T15:14Z
---
release.yml sets `permissions: {}` and grants per job; ci.yml, fuzz.yml and golden.yml set nothing
and inherit the repository default, read-only today (`default_workflow_permissions: read`). Once
public, fork PRs run these workflows, and the repository default is a setting a click can widen. A
top-level `permissions: contents: read` in each file pins what the setting leaves loose. fuzz.yml's
upload-artifact needs nothing more: it uploads today under the read-only default token, and a
top-level `contents: read` is that same token spelled out.

## log
- 2026-11-02T15:14Z created
