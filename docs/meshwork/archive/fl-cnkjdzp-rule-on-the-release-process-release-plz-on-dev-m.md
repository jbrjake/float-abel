---
id: fl-cnkjdzp
title: "Rule on the release process: release-plz on dev→main as read back, crates.io publish on the owner's dispatch or inside the release, and the signed-tag requirement against Sigstore attestation"
category: rulings
labels: [reproducible-floats]
seq: 12
parent: fl-m3cymxv
docs:
  - docs/DESIGN.md#release-sp-release
  - CLAUDE.md#standing-decisions
verify: contains docs/meshwork/fl-cnkjdzp-rule-on-the-release-process-release-plz-on-dev-m.md /2026-[0-9][0-9]-[0-9][0-9] owner ruled on the release process/
status: done
created: 2026-11-02T15:33Z
---
Three questions for the owner before anything is built; the parent's body is the readback.

1. The tool and flow: release-plz with the release PR against `dev` and the release cut on the
   `dev`→`main` merge, as read back. The alternative is release-please (Google's, language-agnostic,
   `rust` release type) with the same two-step shape; release-plz is chosen because it is the Rust
   ecosystem's own, understands Cargo workspaces and Cargo.lock, and runs cargo-semver-checks to
   decide the bump (fl-cr71vam folds in).
2. crates.io: stays on the owner's dispatch of release.yml (`publish = false`), or moves into the
   release so a merge to `main` publishes too (release-plz supports Trusted Publishing through
   rust-lang/crates-io-auth-action; the first publish still needs a token). CLAUDE.md § Standing
   decisions holds "make it a ci publish on my manual trigger"; the owner says whether that stands.
3. The signed tag: DESIGN.md § Release says "CI builds it with --locked from a signed tag", and
   fl-x2k7fbb waits on the owner naming a key. In this flow the tag is created by the release
   workflow itself, so a tag signature by the owner's key is not available; the artifact's
   provenance is the Sigstore attestation bound to the workflow run and the merge commit, which
   GitHub signs (web-flow) because the merge is made through the PR. The owner rules whether the
   attestation replaces the signed-tag clause or a bot key signs the tag.

Done is the owner's words on the three in this task, dated, and DESIGN.md § Release rewritten to
the ruled shape (the verify reads the tool's name there).

## log
- 2026-11-02T15:33Z created
- 2026-11-02T16:39Z open→done — verify exit 0 @ 9efa6ee+8

## comments
- 2026-11-02T16:22Z [claude (de9bd423-d8b5-4cf3-8a2c-44d698f4cc3e)] 2026-11-02 sources read raw ahead of the walkthrough (release-plz/release-plz main: website/docs, crates/next_version/src/lib.rs, crates/release_plz_core/src/{version.rs,diff.rs,updater.rs}; googleapis/release-please main: docs/manifest-releaser.md, src/strategies/{base,rust}.ts; actions/attest-build-provenance README; github/docs reusables/pull_requests/rebase_and_merge_verification*). What they settle, and a fourth question they raise:
  
  1. release-plz does not derive a never-published crate's version from the log. `Diff::should_update_version` is `registry_package_exists && has_commits() && is_version_published`, and version.rs's test `next_version_of_new_package_is_unchanged` pins it: a package absent from the registry (or, under `git_only`, with no matching tag) is released at Cargo.toml's version as it stands. Here that is 0.0.0. So the first version is typed once, by someone; only every later one is derived. The fourth question: which first version.
  2. next_version's rules (lib.rs doc): in 0.0.x every commit kind bumps the patch, breaking included ("the bump from 0.0.x to 0.1.x should be intentional (not automated)"); in 0.x a feat bumps the patch and a breaking change the minor unless `features_always_increment_minor = true`, which release-plz's config.md warns "violates the Cargo SemVer rules". From 0.0.0 the log's 28 feat, 7 fix and 0 BREAKING footers would yield 0.0.1 if the tool did derive it.
  3. release-please's rust strategy returns 0.1.0 as `initialReleaseVersion` (rust.ts), base.ts 1.0.0; `bump-patch-for-minor-pre-major` is off by default, so pre-1.0 feats bump the minor there. Its first version is the tool's constant, not the log's either.
  4. release-plz creates tags through the GitHub API, which "doesn't support signed tags" (github/persist-credentials.md); with `tag.gpgSign = true` it pushes signed tags over the git CLI, needing `persist-credentials: true` and the PAT in checkout. The release PR's commit is made through the GraphQL API and so carries GitHub's web-flow signature (usage/release-pr.md § Github). GitHub signs merge-button merge and squash commits and does not sign rebase-merged commits (reusable rebase_and_merge_verification).
  5. In the dev→main shape `release_always` must stay `true` (default): the `false` mode releases only when main's latest commit is associated with a PR whose branch starts with `release-plz-`, and main's latest commit is the dev→main merge. Under `publish = false` release-plz "will still use the cargo registry to check what's the latest release"; `git_only = true` reads the tag instead and is the setting for a crate kept off crates.io by the dispatch route. The release PR's base "is the default branch of the repository", so dev must be the default branch.
  6. The action is now `release-plz/action` (quickstart; the MarcoIeni name in fl-ps58fd0 is the old one); latest v0.5.148 at tag object 1fb72a9583c3601c197b3bd6fc6ce50d58e28ef3 (2026-11-02), release-plz core release-plz-v0.4.3. release-plz implements the crates.io Trusted Publishing exchange itself under `id-token: write`, and the quickstart repeats crates.io's limit: "new crates can't be published with trusted publishing — you need to publish them manually the first time."
  7. attest-build-provenance's README: "If you are on a GitHub Free, GitHub Pro, or GitHub Team plan, artifact attestations are only available for public repositories." The repo is private today, so fl-kpk3gd6 is proven only after the flip.
- 2026-11-02T16:38Z [claude (de9bd423-d8b5-4cf3-8a2c-44d698f4cc3e)] 2026-11-02 owner ruled on the release process, in this session, answering the walkthrough's four questions verbatim:
  
  - On the first version (asked which version to type once, since release-plz releases a never-published crate at Cargo.toml's version as it stands): "this question makes no sense. the base initial value WE NEVER USE is 0.0.0 that IMMEDIATELY GETS THE CONVENTIONAL COMMITS ADDED TO IT EVEN FOR THE FIRST BUILD!"
  - On the tool and flow (release-plz against release-please, dev→main, merge commits only): "well we don't want to publish to crates.io yet, and of course feats bump the minor wtf that is just standard conventional commits! and you're not mentioning the thing I ACTUALLY ASKED FOR which is github attestation!"
  - On where the crates.io publish lives: "WE AREN'T PUBLISHING TO CRATES YET!!!!!!!!!!!! I KEEP TELLING YOU THIS AND YOU KEEP IGNORING ME! I WILL TELL YOU WHEN I WANT IT AND THEN YOU WILL ADD IT TO THE RELEASE PROCESS, BUT NOT UNTIL WE HAVE A NORMAL GITHUB RELEASE WORKING LIKE YOU ALREADY SHOULD HAVE DONE!"
  - On what replaces DESIGN.md's "from a signed tag": the option "Attestation + tag ruleset (Recommended)": the Sigstore attestation of the .crate bound to the workflow and commit, the GitHub-signed commit under the tag, a ruleset blocking tag update and deletion, no key in CI, fl-x2k7fbb dropped.
  
  What the session derives from those words, read back to the owner in the same session and built unless the owner objects:
  
  1. release-plz stands as read back (the owner named no other tool), with `git_only = true` because crates.io is outside the release, and `features_always_increment_minor = true` for "feats bump the minor". A breaking change below 1.0 bumps the minor (release-plz's default, Cargo's rule); `breaking_always_increment_major = true` is the one-line switch if the owner wants 1.0.0 on the first `!`.
  2. The base: an annotated tag `v0.0.0` on d2f29de, the first commit carrying Cargo.toml at 0.0.0 (the root commit ae9505a holds only .gitignore and LICENSE, and release-plz's git-only path runs cargo metadata at the tag's commit, so the tag must sit on a manifest). It is never built or released. release-plz counts every commit since it, so the first release PR carries 0.0.0 plus the log's bump: 0.1.0 on today's log (28 feat, 7 fix, no BREAKING footer), and no one types it.
  3. crates.io: not part of the release process. release.yml's dispatch publish stays dormant and untouched, fl-7en8z60 stays blocked, and nothing in the release-plz workflow references a registry or a registry token. When the owner says, publishing is added to the release.
  4. Attestation is the deliverable: on the release cut, the workflow builds the .crate from the tag with `cargo package --locked`, attests it with actions/attest-build-provenance and attaches .crate, sha256 and attestation to the GitHub Release; README's install section says `gh attestation verify`. Proven after the flip, since attestations are public-repo-only on a Free plan.
  5. DESIGN.md § Release rewritten to this shape (the "0.1.0 ships once …" list and the signed-tag clause are gone; the owner's merge is the release gate); CLAUDE.md § Standing decisions carries the ruling in the owner's words.
  6. Owner acts the build will hand back: a GitHub App or fine-grained PAT as the `RELEASE_PLZ_TOKEN` secret so the release PR runs ci (the workflow falls back to GITHUB_TOKEN until then), `dev` as default branch, the `main` ruleset (PR from dev only, merge commits only, required checks), the `v*` tag ruleset, and the flip public.
