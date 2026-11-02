---
id: fl-fs9vhnd
title: "Set every workflow's token to read-only at the top: permissions in ci.yml, release.yml and publish.yml"
category: crate/hardening
labels: [reproducible-floats]
seq: 50
parent: fl-b9qtzty
docs:
  - CLAUDE.md#standing-decisions
verify: "all(contains .github/workflows/ci.yml /^permissions:/, contains .github/workflows/release.yml /^permissions:/, contains .github/workflows/publish.yml /^permissions:/)"
status: done
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
- 2026-11-02T18:34Z open→done — verify exit 0 @ 4bec2d5+2

## comments
- 2026-11-02T18:34Z [claude (227b40e5-20bd-44b0-9b79-d44c1a3c3587)] 2026-11-02 done inside fl-ezhk4cz at 4bec2d5: ci.yml carries `permissions: contents: read` at the top, and its changes job alone adds `actions: read` for `gh run list`; release.yml (from cut-release.yml) and publish.yml (from release.yml) keep `permissions: {}` at the top with per-job grants. fuzz.yml and golden.yml are gone: their jobs live in ci.yml under its top-level grant, and the fuzz artifact upload runs under that read-only token as the body said it would. Verify repointed at the three files that exist.
