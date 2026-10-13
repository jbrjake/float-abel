---
id: fl-7czsn8x
title: "Rule on dependabot for the action hashes — a github-actions ecosystem that refreshes each pin and its version comment by PR, or pins refreshed by hand"
status: open
category: rulings
labels: [reproducible-floats]
discovered-from: fl-cvpya2q
docs:
  - scripts/check-actions-pinned.sh
seq: 137
created: 2026-10-13T14:51Z
verify: contains docs/meshwork/fl-7czsn8x-rule-on-dependabot-for-the-action-hashes-a-githu.md /2026-[0-9][0-9]-[0-9][0-9] owner ruled/
---

Owner's call, split out of "Verify the golden workflow's wasmtime and cross tarballs by checksum" (fl-cvpya2q) so that task could close on its mechanical half. Every `uses:` in ci.yml, golden.yml and fuzz.yml names a forty-hex commit with the tag in a trailing comment (owner: "use git hashes to reference dependencies not git labels that can be manipulated"), and scripts/check-actions-pinned.sh fails both gates on anything else. A hash does not move, so the pins stop tracking upstream the day they land: a fix in actions/checkout or actions/cache reaches this repo only when someone re-resolves the tag and rewrites the hash and the comment together.

Options:
(a) Add `.github/dependabot.yml`, `version: 2`, one `updates:` entry for the `github-actions` ecosystem, `directory: /`, weekly. Dependabot opens one PR per bump that rewrites the hash and keeps the `# vX.Y.Z` comment current, and check-actions-pinned.sh runs on that PR like any other. Nothing merges on its own. The tarball digests (`WASMTIME_SHA256`, `CROSS_SHA256` in golden.yml) are not actions and stay by hand either way.
(b) No dependabot. The pins are refreshed by hand, each one a task: resolve the tag through `gh api repos/<repo>/git/ref/tags/<tag>` (and `git/tags/<sha>` for an annotated tag), confirm the commit carries the version, rewrite hash and comment.

Recommendation: (a), because a pin nobody is prompted to refresh is a pin that rots, and the PR is the prompt; the review stays the owner's, and the gate already holds any PR to the hash-only form. A `cargo` ecosystem entry is a separate question and is not asked here.

The owner writes the line this task's verify looks for, dated.

## log
- 2026-10-13T14:51Z created
