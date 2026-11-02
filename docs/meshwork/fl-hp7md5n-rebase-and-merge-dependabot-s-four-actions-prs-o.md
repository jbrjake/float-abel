---
id: fl-hp7md5n
title: "Rebase and merge dependabot's four actions PRs once main is green: checkout v7.0.1, cache v6.1.0, upload-artifact v7.0.2, crates-io-auth-action v1.0.6"
category: crate/hardening
labels: [reproducible-floats]
seq: 75
parent: fl-b9qtzty
needs: [fl-r0s86hg]
docs:
  - CLAUDE.md#standing-decisions
verify: lacks .github/workflows/ci.yml 11d5960a326750d5838078e36cf38b85af677262
status: open
created: 2026-11-02T15:14Z
---
PRs #1 to #4 are red on stale bases, not on their changes: #2's base b14d830 still exec'd the
meshwork shim from smoke.sh (`exit 127`, since skipped under CI); #1 and #3 at 4a83a00 hit the miri
index-out-of-bounds at tests/binned_product.rs:426 that c385a17 fixed; #4 at f24b051 hit the dbsp
advisories (fl-r0s86hg). Each PR keeps the hash-plus-version-comment form and the gate's pinning
check passed on #1 and #3, so the standing decision holds through them.

Once fl-r0s86hg lands on main: `@dependabot rebase` on each (a comment on GitHub, so the owner's
word first, or the weekly schedule does it), watch the ci, golden and fuzz checks green, and merge
in the order #1 (checkout, touches all four workflows), #3 (cache), #2 (upload-artifact), #4
(crates-io-auth-action). checkout v7 and cache v6 run on node24; ubuntu-latest has it. Merging is
the owner's act on the repo; the verify reads the old checkout hash gone from ci.yml.

## log
- 2026-11-02T15:14Z created
