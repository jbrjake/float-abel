---
id: fl-ps58fd0
title: "Add release-plz: release-plz.toml with publish off, the release-PR job on dev and the release job on main, hash-pinned, with CHANGELOG.md's hand-written Unreleased folded into the generated form"
category: crate/release
labels: [reproducible-floats]
seq: 16
parent: fl-m3cymxv
needs: [fl-cnkjdzp, fl-k594v2g]
docs:
  - docs/DESIGN.md#release-sp-release
verify: "all(exists release-plz.toml, exists .github/workflows/release-plz.yml)"
status: doing
created: 2026-11-02T15:33Z
claimed-by: claude (de9bd423-d8b5-4cf3-8a2c-44d698f4cc3e)
---

The ruled shape (fl-cnkjdzp, 2026-11-02). `release-plz.toml`: `git_only = true` (crates.io is outside the release, so the tag, never a registry, says what is released), `features_always_increment_minor = true` ("of course feats bump the minor"), `semver_check = true`, `git_release_enable = true`, `git_tag_enable = true`, `release_always = true` (the default; the `false` mode looks for a `release-plz-*` branch on main's latest commit, and in the dev→main shape that commit is the dev merge), `pr_branch_prefix = "release-plz-"`, and git-cliff's conventional preset tuned to `type(scope): subject`, `BREAKING CHANGE` footers kept. No `publish` key and no registry token anywhere in the workflow.

The base: an annotated tag `v0.0.0` on d2f29de, the first commit carrying Cargo.toml at 0.0.0, pushed before the first release-pr run. release-plz's git-only path checks the tag's commit out and runs cargo metadata there, so the tag must sit on a manifest; the root commit has none. Everything since it counts, so the first release PR carries 0.1.0 on today's log and nobody types it.

`.github/workflows/release-plz.yml`: on push to `dev`, `release-plz release-pr` (the PR's base is the default branch, so dev must be it: fl-k594v2g); on push to `main`, `release-plz release`, then in the same job, when `releases_created` is true, check out the tag, `cargo package --locked`, `sha256sum`, `actions/attest-build-provenance` and `gh release upload` of the .crate, the .sha256 and nothing else (fl-kpk3gd6 owns that half; same job so it runs whether or not the owner has set a token, since a Release created with GITHUB_TOKEN fires no `release: published`). The action is `release-plz/action` (the MarcoIeni name is retired), pinned by commit hash with the tag in the comment, v0.5.148 at tag object 1fb72a9583c3601c197b3bd6fc6ce50d58e28ef3 on 2026-11-02 (resolve the tag to its commit before pinning, since that sha is the annotated tag object). The release-pr job runs with `GITHUB_TOKEN: ${{ secrets.RELEASE_PLZ_TOKEN || github.token }}`: with the owner's GitHub App or fine-grained PAT in the secret the release PR runs ci, and without it the PR still opens. Checkout uses `fetch-depth: 0` and `persist-credentials: false`. Permissions: `contents: write`, `pull-requests: write` on the PR job; `contents: write`, `id-token: write`, `attestations: write` on the release job.

CHANGELOG.md's hand-written Unreleased bullets become the body of the first generated release entry rather than being lost, and fl-cr71vam's release-plz half closes on `semver_check = true`. Done is the two files passing `scripts/check-actions-pinned.sh` and the smoke gate, plus a local `release-plz update --dry-run`-style check that the first PR would carry 0.1.0 (install release-plz for it; record the output here).

## log
- 2026-11-02T15:33Z created
- 2026-11-02T16:50Z open→doing — claimed by claude (de9bd423-d8b5-4cf3-8a2c-44d698f4cc3e)
