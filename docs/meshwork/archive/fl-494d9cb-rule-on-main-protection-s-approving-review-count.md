---
id: fl-494d9cb
title: "Rule on main-protection's approving-review count, which blocks the dev→main merge: at 1 with no bypass actor nobody can merge #5, since an author's own approval never counts; 0 keeps PR-only main, merge commits and thread resolution, a bypass actor keeps the 1 and logs every merge as a bypass"
category: rulings
labels: [reproducible-floats]
parent: fl-b9qtzty
relates: [fl-enkdnvk]
seq: 10
verify: contains CLAUDE.md /approving review/
docs:
  - CLAUDE.md#standing-decisions
status: done
created: 2026-11-02T21:28Z
---
Found 2026-11-02 at 21:25Z with #5 green on all seventeen jobs and `gh pr view 5` saying mergeStateStatus BLOCKED, reviewDecision REVIEW_REQUIRED. The ruleset `main-protection` (id 26389711, active, on refs/heads/main, created 2026-11-02T18:45Z by the owner's account, which is also what a session's gh shows) carries deletion, non_fast_forward and a pull_request rule with required_approving_review_count 1, required_review_thread_resolution true, allowed_merge_methods [merge], dismiss_stale_reviews_on_push true, require_extra_approval_for_unattributed_changes true, and bypass_actors []. jbrjake is the repository's one account and #5's author, and GitHub never counts an author's approval, so no account can merge any PR into main as set. 1 is the GitHub UI's default when "Require a pull request before merging" is ticked.

Two ways out, the owner's call: (a) required_approving_review_count 0, keeping every other parameter, so main stays PR-only with merge commits and resolved threads and nothing is bypassed; (b) bypass_actors gets the Repository admin role (actor_type RepositoryRole, actor_id 5, bypass_mode pull_request), so the owner merges with "bypass rules" and each merge is an audited bypass event. (a) is recommended: a rule nobody can satisfy without a bypass is the same rule with noise. The change is a PUT of the ruleset with the one parameter moved; the session has the JSON ready. Done is the owner's word, the ruleset updated, `gh pr view 5 --json mergeStateStatus` no longer BLOCKED, and CLAUDE.md's release decision saying what main's ruleset requires.

## log
- 2026-11-02T21:28Z created
- 2026-11-02T22:04Z open→done — verify exit 0 @ b99611e+5

## comments
- 2026-11-02T22:04Z [claude (aa073407-90a0-459f-92ea-439fb58eb49c)] 2026-11-02: ruled by the owner's act, read back from the ruleset: required_approving_review_count 0 at 21:38:09Z, bypass_actors still [], every other parameter as it was (merge commits only, thread resolution, dismiss stale reviews). #5 merged at 21:38:56Z. CLAUDE.md's release decision now states what main's ruleset requires.
