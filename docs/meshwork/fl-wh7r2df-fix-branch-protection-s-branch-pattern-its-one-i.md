---
id: fl-wh7r2df
title: "Fix branch-protection's branch pattern: its one include is the string \"refs/heads/main dev\", which matches no branch, so dev has no deletion or force-push protection and the ruleset protects nothing"
category: crate/hardening
labels: [reproducible-floats]
parent: fl-b9qtzty
seq: 30
verify: contains docs/DESIGN.md /refs\/heads\/dev/
docs:
  - docs/DESIGN.md#release-sp-release
status: open
created: 2026-11-02T21:28Z
---
Found 2026-11-02: `gh api repos/jbrjake/float-abel/rulesets/26389594` shows conditions.ref_name.include ["refs/heads/main dev"], a single pattern with a space, last changed 2026-11-02T20:45Z (the version that added dev, by the look of it). `gh api repos/jbrjake/float-abel/rules/branches/dev` returns an empty list: no ruleset reaches dev. main is covered by main-protection's own deletion and non_fast_forward rules, so the gap is dev alone, the default branch, where every push lands. The fix is two patterns, "refs/heads/main" and "refs/heads/dev" (or the GitHub shorthand ~DEFAULT_BRANCH for dev), by PUT on the ruleset. A setting is the owner's; on their word the session makes the change and reads back `rules/branches/dev` listing deletion and non_fast_forward. DESIGN.md § Release names the refs the rulesets cover.

## log
- 2026-11-02T21:28Z created
