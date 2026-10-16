---
id: fl-k3g1511
title: "Rule on who runs cargo publish for 0.1.0 and after — CI on the owner's signed tag through crates.io Trusted Publishing, or the owner's machine with CI packaging and attesting the .crate"
status: open
category: rulings
labels: [reproducible-floats]
discovered-from: fl-fm78m8d
relates: [fl-fm78m8d]
docs:
  - docs/DESIGN.md#release-sp-release
  - CLAUDE.md#standing-decisions
seq: 165
created: 2026-10-14T21:28Z
verify: contains docs/meshwork/fl-k3g1511-rule-on-who-runs-cargo-publish-for-0-1-0-and-aft.md /2026-[0-9][0-9]-[0-9][0-9] owner ruled/
needs: [fl-dchdzd4, fl-p4dc5w5, fl-jqbhm8j]
---

Owner's call, surfaced from "Prepare the 0.1.0 release" (fl-fm78m8d). Two standing texts read two
ways. DESIGN.md § Release: "CI builds it with `--locked` from a signed tag, with provenance, and
`cargo-semver-checks` is armed from then on. Publishing is the owner's act". CLAUDE.md § Standing
decisions: "`cargo publish` runs only with `--dry-run`. A publish, the crates.io name reservation
included, is the owner's act." Whether the owner's act is the signed tag push that a workflow then
publishes from, or the `cargo publish` command itself on the owner's machine, decides the
workflow's shape: its secrets, its permissions, and what the first release can do at all.

Fixed either way: the name `float-abel` is unreserved on crates.io and reserving it is the owner's
act; crates.io's Trusted Publishing (OIDC from GitHub Actions, no long-lived token) is configured
per crate on crates.io after the crate exists, so it cannot carry the first publish; every `uses:`
is hash-pinned and `scripts/check-actions-pinned.sh` fails both gates otherwise.

Options:
(a) CI publishes. A `release.yml` on tag push `v*`: verify the tag is signed by the owner's key
    (`git verify-tag` against a committed public key), `cargo package --locked`, `cargo publish
    --locked` through Trusted Publishing (`id-token: write`, no secret), attest the `.crate` with
    `actions/attest-build-provenance` and attach it to a GitHub release. The owner's act is the
    signed tag. 0.1.0 is the exception: it runs on the owner's machine with a token, because
    Trusted Publishing needs the crate to exist first, and the workflow carries every release after.
(b) The owner publishes, always. The workflow on tag push verifies the signature, packages with
    `--locked`, dry-runs `cargo publish`, attests the `.crate` and attaches it to a GitHub release,
    and the owner runs `cargo publish --locked` from a clean checkout of the tag. No secret, no
    OIDC, and CLAUDE.md's line holds for CI as it does for sessions.
(c) (a), triggered by `workflow_dispatch` on the tag rather than by the tag push, so a signed tag
    alone never publishes.

Recommendation: (b) for 0.1.0, because the first publish cannot come from CI under any option and
the crate's posture keeps `cargo publish` out of every automated path; (a) from 0.1.1 on, filed
when the owner wants it, because Trusted Publishing then takes the token off the owner's machine.
The owner writes the line this task's verify looks for, dated, with the option.

## log
- 2026-10-14T21:28Z created
- 2026-10-16T18:11Z handoff by claude (cd1130c2-56bf-432b-bcb1-8ed1c474e8fa)

## comments
- 2026-10-16T18:12Z [claude (cd1130c2-56bf-432b-bcb1-8ed1c474e8fa)] 2026-10-16 owner, in session, on this task leading the queue: "look no there is no publish decision until it's working! you have other tasks don't you???". So the ruling waits behind the open build work: this task now needs fl-dchdzd4 (the limb cap's edge), fl-p4dc5w5 (the heap arm's budget) and fl-jqbhm8j (the bench baseline), and leaves ready until they are done.
