---
id: fl-fm78m8d
title: "Prepare the 0.1.0 release — CI publish job with --locked, a signed tag and provenance, cargo-semver-checks armed, and the CHANGELOG entry"
category: crate/release
labels: [reproducible-floats, rf-tail]
docs: [docs/DESIGN.md#release-sp-release]
needs: [fl-z91b7pq, fl-6shtzpj, fl-v6mh9cv, fl-06sggdc, fl-9kp61j0, fl-ajqrs9p, fl-c3ahdwc, fl-v7t6xhz, fl-b41y9kt, fl-jmjbjk5, fl-k3g1511]
seq: 170
verify: "all(exists CHANGELOG.md, contains CHANGELOG.md /^## 0\\.1\\.0/)"
status: open
created: 2026-10-11T15:02Z
covers:
  - ref: docs/DESIGN.md#sp-release
    sha: 56381872cf6a1b1ceef187cdda6df4028cbdee7e10fddf87590a202cc3ee6f62
handoff: |
  Waits on the publish ruling in this task's `needs:` (who runs `cargo
  publish`: CI on the owner's
  signed tag, or the owner's machine with CI packaging and attesting);
  build nothing of the publish
  step before it lands, and `why` names it. The tree at 6b92804 has the
  outward docs in: README.md
  with its three Rust blocks run as doctests through `mod readme` in
  src/lib.rs (behind
  `cfg(doctest)` and `differential`), SECURITY.md pointing at GitHub's
  private vulnerability
  reporting alone (switching the form on in the repository settings is the
  owner's act and is not
  done), and CHANGELOG.md opening with an `## Unreleased` list of what
  0.1.0 carries. The commit gate
  ran 169 tests green at 1621409; the push gate was last run green end to
  end at 6be0b49.
  
  What this task builds once ruled: `.github/workflows/release.yml` on tag
  push `v*`, every `uses:`
  hash-pinned with the tag in a trailing comment
  (scripts/check-actions-pinned.sh fails both gates
  otherwise), `git verify-tag` against a committed public key (the owner's
  signing key is not in the
  tree: ask for it, or for the fingerprint to pin), `cargo package
  --locked`, the dry-run or the
  publish per the ruling, `actions/attest-build-provenance` over the
  `.crate`, and a GitHub release
  carrying it. A `semver` job in ci.yml running `cargo semver-checks`
  against the crates.io baseline
  errors while the crate is unpublished, so arm it in the same change as
  the `## 0.1.0` entry. The
  `## 0.1.0` CHANGELOG entry is this task's verify: it moves the
  Unreleased list under a `## 0.1.0`
  heading at release time, the owner's moment, so it is the last thing
  written. Cargo.toml is at
  version 0.0.0 with `include` naming README.md, FORMAT.md and the two
  licences; CHANGELOG.md and
  SECURITY.md stay out of the package, which is usual.
  
  Open beside this and unrelated: fl-jqbhm8j (bench-baseline.json from the
  criterion estimates at
  2a82388; the owner said to move on from benchmarking, so ask before
  spending a session on it) and
  fl-qv0vh39 (the AVX-512 kernel, which needs a machine with the flag).
  The spec audit's 23 re-open
  candidates are done tasks whose clauses have moved since they were
  pinned, standing noise from
  before this session, not drift this session caused.
---
Per DESIGN.md § Release. This task makes 0.1.0 ready to ship: the CI publish job, the signed-tag
flow, provenance, `cargo-semver-checks` armed from here, and the `## 0.1.0` CHANGELOG entry. Running
`cargo publish`, and any announcement, are the owner's acts.

## log
- 2026-10-11T15:02Z created
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-release @56381872cf6a
- 2026-10-14T21:28Z handoff by claude (4e0e29d7-b85d-4a99-93e3-a9a5c00bb168)
