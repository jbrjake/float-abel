---
id: fl-pg35w24
title: "Create the v* tag ruleset the standing decision names: none exists, so a release tag can be moved or deleted; it must forbid deletion and update on refs/tags/v* without blocking the release job's creation of the tag"
category: crate/hardening
labels: [reproducible-floats]
parent: fl-b9qtzty
relates: [fl-enkdnvk]
seq: 40
verify: contains docs/DESIGN.md /refs\/tags\/v\*/
docs:
  - docs/DESIGN.md#release-sp-release
status: open
created: 2026-11-02T21:28Z
---
Found 2026-11-02: `gh api repos/jbrjake/float-abel/rulesets` lists two rulesets, both target branch; CLAUDE.md § Standing decisions says "a ruleset pins v* tags (owner chose "Attestation + tag ruleset")" and DESIGN.md § Release says "A ruleset keeps v* tags from being moved or deleted". Until the flip no ruleset could exist (fl-4fcbtkx's 18:34Z comment); the flip is done and the first tag, v0.30.2, is about to be created by release.yml with the workflow's GITHUB_TOKEN. A tag ruleset with deletion and non_fast_forward (update) rules on refs/tags/v* and no creation rule leaves that creation free and pins every tag once made; a creation rule would need the GitHub Actions integration as a bypass actor. Create it after the owner's word, read it back with `gh api repos/jbrjake/float-abel/rules/branches` (tags: `.../rulesets/<id>`), and have DESIGN.md § Release name refs/tags/v* as the ref it covers.

## log
- 2026-11-02T21:28Z created
