---
id: fl-d609gg2
title: "Make the release-pr job pass on the runner: release-plz 0.4.2 fails parsing CHANGELOG.md's header as a template (stack limit reached) where 0.4.3 passes locally"
status: done
category: crate/release
labels: [reproducible-floats]
parent: fl-m3cymxv
verify: lacks .github/workflows/release-plz.yml /release-plz@0\.4\.2/
docs:
  - .github/workflows/release-plz.yml#release-pr
seq: 15
created: 2026-11-02T17:14Z
---

Run 41488323689 on dev at 5a1bdd0: the guard passed, release-plz 0.4.2 found tag v0.0.0, cargo-semver-checks 0.51.0 ran (3 minutes, API compatible) and the next version came out 0.1.0, then `error while building changelog: Template parse error --> 1:1 '# Changelog' ^--- stack limit reached`. The same CHANGELOG.md header and release-plz.toml passed under release-plz 0.4.3 locally (fl-ps58fd0's dry run). 0.4.3's only note is 'update Cargo.lock dependencies', so the parser (tera over pest) moved between the two. Reproduce with 0.4.2 locally (cargo install release-plz --version 0.4.2 --locked --root <scratch>), then either pin 0.4.3 once taiki-e/install-action's manifest carries it (a sha256 per platform, the hash-pinning rule), or install it from crates.io by exact version with cargo install --locked in the job behind an actions/cache keyed on the version, or shorten the header if the parse depth is what it trips on. Done is the release-pr job green on a dev push, with the release PR it opens (or its 403 on PR creation, which is the token's, not this task's).

## log
- 2026-11-02T17:14Z created
- 2026-11-02T17:20Z open→doing — claimed by claude (de9bd423-d8b5-4cf3-8a2c-44d698f4cc3e)
- 2026-11-02T17:26Z doing→done — verify exit 0 @ 35d0a77+2

## comments
- 2026-11-02T17:20Z [claude (de9bd423-d8b5-4cf3-8a2c-44d698f4cc3e)] 2026-11-02 reproduced on jons-amd-mini (x86_64 Ubuntu, the repo rsynced with its .git) with the four release assets, release-plz update --allow-dirty each: 0.4.2 x86_64 musl (sha256 b1341080…, the binary taiki-e/install-action's manifest serves) fails with 'error while building changelog' after deriving 0.1.0; 0.4.2 gnu (bd7c3adc…) passes; 0.4.3 musl (4a4cfa92…) passes; 0.4.3 gnu (f4b0cc40…) passes, each writing version 0.1.0 and the [0.1.0] entry. Locally on macOS a cargo-installed 0.4.2 passes too, so the failure is the 0.4.2 musl build's (its lockfile predates pest_stack_guard, the one crate 0.4.3's Cargo.lock adds). Fix: both jobs install release-plz 0.4.3 from its x86_64 gnu release tarball and check sha256 f4b0cc404cd9a58cd1b33f49e439bba65186773f63e323ea47f3ea57e557a3d3, measured on the mini and again here (shasum -a 256) before anything runs; cargo-semver-checks stays on install-action. The tarball holds release-plz at its top level.
- 2026-11-02T17:26Z [claude (de9bd423-d8b5-4cf3-8a2c-44d698f4cc3e)] 2026-11-02 run 41490353635 on dev at 35d0a77: release-plz 0.4.3 installed by hash (sha256 checked), tag v0.0.0 found, cargo-semver-checks 0.51.0 ran (✓ API compatible), next version 0.1.0, changelog built, and the job failed only at 'Failed to open PR: GitHub Actions is not permitted to create or approve pull requests' (403 on POST /pulls), which is the token's: the owner's RELEASE_PLZ_TOKEN (GitHub App or fine-grained PAT) or the Actions setting that lets GITHUB_TOKEN open PRs. The run on main at the same commit, 41490352546, is green: 'Already published - Tag v0.0.0 already exists', releases [].
