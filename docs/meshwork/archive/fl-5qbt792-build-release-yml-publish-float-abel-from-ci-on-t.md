---
id: fl-5qbt792
title: "Build release.yml: publish float-abel from CI on the owner's workflow_dispatch"
category: crate/release
labels: [reproducible-floats]
seq: 160
discovered-from: fl-k3g1511
relates: [fl-fm78m8d]
docs:
  - docs/DESIGN.md#release-sp-release
  - CLAUDE.md#standing-decisions
verify: "all(exists .github/workflows/release.yml, contains .github/workflows/release.yml workflow_dispatch, contains .github/workflows/release.yml crates-io-auth-action)"
status: done
created: 2026-10-20T15:04Z
---
Owner, 2026-10-20, on fl-k3g1511: "make it a ci publish on my manual trigger". A `release.yml` on
`workflow_dispatch` alone, never on a tag push, with two inputs: the tag to publish (`v*`) and
`auth`, `token` or `trusted`. The job checks out that tag, asserts it names `Cargo.toml`'s version,
runs `cargo package --locked`, publishes with `cargo publish --locked` under the chosen
authentication, attests the `.crate` with `actions/attest-build-provenance` (DESIGN.md § Release:
"with provenance") and attaches it to a GitHub release of the tag. `token` reads the repository
secret `CARGO_REGISTRY_TOKEN` and fails closed when it is unset; `trusted` runs
`rust-lang/crates-io-auth-action` under `id-token: write`. Every `uses:` is hash-pinned
(`scripts/check-actions-pinned.sh`), permissions are the least each step needs, and the job runs
in the `release` environment so the owner can put reviewers or a branch rule on it.

Why two paths: crates.io's Trusted Publishing page says "Your crate must already be published to
crates.io (initial publish requires an API token)", so 0.1.0 goes by token and every later
release by OIDC once the owner configures Trusted Publishing on the crate.

Done: the file passes `scripts/check-actions-pinned.sh` and `actionlint` if present, the smoke
gate, and `cargo publish --dry-run --locked` runs green here; CLAUDE.md's standing decision and
DESIGN.md § Release say the ruling. Nothing publishes until the owner dispatches.

## log
- 2026-10-20T15:04Z created
- 2026-10-20T15:19Z open→doing — claimed by claude (4a8ded2c-6eb4-4638-a7f1-ee6da67e86ba)
- 2026-10-20T15:22Z doing→done — verify exit 0 @ 9f651e2+4

## comments
- 2026-10-20T15:22Z [claude (4a8ded2c-6eb4-4638-a7f1-ee6da67e86ba)] 2026-10-20 built at 9f651e2. Validated here: scripts/check-actions-pinned.sh OK on release.yml, YAML parses (8 steps, inputs tag and auth), cargo publish --dry-run --locked exit 0 (49 files, 148.4KiB compressed), and cargo package --locked leaves target/package/float-abel-<v>.crate, the attestation's subject. Not validated: a dispatch, which is the owner's act and needs the release environment (auto-created on first run), the CARGO_REGISTRY_TOKEN secret for the first publish, and a public repository for actions/attest-build-provenance. The tag's signature is fl-x2k7fbb's.
