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

## comments
- 2026-11-02T17:21Z [claude (de9bd423-d8b5-4cf3-8a2c-44d698f4cc3e)] 2026-11-02 dependabot re-targeted its four PRs at dev when dev became the default branch (#1–#4 now base dev). ci's new commits job checks each PR's commit subjects and title against scripts/check-commit-message.sh, which wants a lowercase subject: #1 and #4 say 'bump' and pass, #2 and #3 carry dependabot's older 'Bump' and will fail that job until dependabot recreates them (a '@dependabot recreate' comment, or the weekly run after closing them) or the owner merges them by hand. Merging needs the owner's token: the PRs are dependabot's, and the merge is a write to dev.
- 2026-11-02T18:34Z [claude (227b40e5-20bd-44b0-9b79-d44c1a3c3587)] 2026-11-02, fl-ezhk4cz at 4bec2d5: ci.yml now triggers on `pull_request` into main alone, so dependabot's PRs into dev run no checks; their merge is a push to dev, tested by the next nightly. The `commits` job no longer runs on them either, but it runs on the dev→main PR over every non-merge commit in the range, so a 'Bump' subject (#2, #3) merged into dev as a merge commit fails there. Merge each with a conforming subject: squash with the PR title (dependabot's titles are lowercase 'bump' on #1 and #4; retitle #2 and #3 by hand or `@dependabot recreate`), or rebase-merge after a recreate. The verify stands: the old checkout hash gone from ci.yml.
