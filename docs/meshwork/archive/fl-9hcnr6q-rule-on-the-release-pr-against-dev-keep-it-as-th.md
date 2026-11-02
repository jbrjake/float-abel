---
id: fl-9hcnr6q
title: "Rule on the release PR against dev: keep it as the entry's review on GitHub, or let release-pr.sh commit the release prep straight to dev so the dev→main PR is the one review and one merge"
category: rulings
labels: [reproducible-floats]
parent: fl-m3cymxv
relates: [fl-ebbxwd2]
docs:
  - docs/DESIGN.md#release-sp-release
verify: contains docs/meshwork/fl-9hcnr6q-rule-on-the-release-pr-against-dev-keep-it-as-th.md /2026-[0-9][0-9]-[0-9][0-9] owner ruled on the release PR/
status: done
created: 2026-11-02T19:55Z
---
The release PR against dev was release-plz's mechanism, carried over when the scripts replaced it. The owner's words asked for one PR: "pr from dev to main to cut one, versions automatically created from conventional commit log tracking" (fl-m3cymxv). Today a release is two PRs and two merges by the owner: the release PR (release/next into dev, the three generated files) and then dev→main. With `scripts/release-pr.sh` cutting the PR from a clone, the entry is readable locally with `--dry-run` before anything is pushed, so the release PR's one job, a GitHub-side read of the number and the entry, is done on the developer's machine.

The alternative: release-pr.sh commits `chore(release): vX.Y.Z` onto dev itself and pushes dev (a push to dev runs no ci, like every push to dev), and the owner opens and merges dev→main alone. Concretely, for the first cut at 0.30.1: today, `./scripts/release-pr.sh`, merge the release PR on GitHub, `gh pr create --base main --head dev`, merge; with the change, `./scripts/release-pr.sh`, `gh pr create --base main --head dev`, merge. What the change gives up: the release prep commit never sits in a PR of its own, so a wrong entry is caught in `--dry-run` or in the dev→main PR's diff, not in a dedicated review. Nothing else moves: the release job's refusal of a Cargo.toml that disagrees with the count, the tag, the attestation and the Release are unchanged either way.

A ruling, not a decision: the session built the clone-side script in the current two-PR shape and changed nothing here. When the owner rules, this task's comment carries "<date> owner ruled on the release PR: <words>", and a `keep` closes it while a `one PR` files the change under fl-m3cymxv.

## log
- 2026-11-02T19:55Z created
- 2026-11-02T20:35Z open→done — verify exit 0 @ 7d78a7a+1

## comments
- 2026-11-02T20:35Z [claude (987c488d-43f3-4192-b0e1-6b279735b89c)] 2026-11-02 owner ruled on the release PR, in session: "i want it to be a single PR, dev to main." The release PR against dev goes; scripts/release-pr.sh commits the release prep onto dev and opens or refreshes the one pull request, dev into main. Filed under fl-m3cymxv as the change this ruling names.
