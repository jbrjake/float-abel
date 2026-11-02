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
status: open
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
