---
id: fl-cvpya2q
title: "Verify the golden workflow's wasmtime and cross tarballs by checksum, and rule on dependabot for the action hashes"
status: open
category: crate/hardening
labels: [reproducible-floats]
verify: "all(contains .github/workflows/golden.yml sha256sum, lacks .github/workflows/golden.yml /tar xJ$/)"
docs:
  - .github/workflows/golden.yml
seq: 136
created: 2026-10-13T13:43Z
discovered-from: fl-k2383pg
handoff: |
  The tree at 1b076e9 carries the three fuzz targets with their num-bigint
  oracle, the miri job, the
  Arbitrary impls and every action pinned to its commit hash; the push
  gate is green end to end
  (`./scripts/regression.sh`, sandbox off for cargo-deny's advisory
  fetch), the suite ran green
  under miri locally, and each fuzz target ran ninety seconds under ASan
  with no finding. This task
  is the owner's "security is absolutely critical here" carried one step
  further, and it has two
  halves.
  
  The checksum half is mechanical. golden.yml's `wasm32` job pipes
  `wasmtime-v49.0.2-x86_64-linux.tar.xz` into `tar xJ`, and its `emulated`
  job pipes
  `cross-x86_64-unknown-linux-gnu.tar.gz` (cross v0.2.5) into `tar xz -C
  ~/.cargo/bin`, each by a
  version-pinned URL and nothing else. Record each asset's sha256 as an
  `env:` value beside its
  version, download to a file, `echo "<sha> <file>" | sha256sum -c -`,
  then extract. Take the
  digests from the GitHub release API (`gh api
  repos/<owner>/<repo>/releases/tags/<tag>` lists
  assets with a `digest` field) or from the release's published checksum
  file, and say in the
  commit where each came from. The verify on this task holds the workflow
  to `sha256sum` present
  and the bare `| tar xJ` gone.
  
  The dependabot half is the owner's call and is written into the body:
  whether a
  `.github/dependabot.yml` with the `github-actions` ecosystem keeps the
  hashes current by PR (it
  refreshes the hash and the version comment together), or the pins are
  refreshed by hand. Read the
  question back and build nothing until the owner answers; if the answer
  is yes, the file is
  `version: 2` with one `updates:` entry for `github-actions` on a weekly
  schedule, and
  scripts/check-actions-pinned.sh already keeps any PR it opens honest.
  
  Mechanics that carry over: every `uses:` must stay a forty-hex-digit
  commit, with the tag in a
  trailing comment, or scripts/check-actions-pinned.sh fails both gates;
  resolve a tag through
  `gh api repos/<repo>/git/ref/tags/<tag>` and, when the object is an
  annotated tag, through
  `git/tags/<sha>` to the commit, then confirm the commit carries the
  version tag you name. The
  sandbox blocks github.com, so `gh api` and the gate need it off.
---

Follows the owner's ruling on fl-k2383pg ("security is absolutely critical here"). golden.yml
downloads two release tarballs by version-pinned URL, wasmtime v49.0.2 and cross v0.2.5, and
pipes each straight into tar: a replaced release asset would run on the runner unnoticed. Record
each asset's sha256 beside its version and check it with sha256sum before extracting; GitHub's
release API now reports an asset's digest, which is where to take it from.

Question for the owner, with the same task: the action hashes in ci.yml, golden.yml and fuzz.yml
are now exact commits, so they stop moving. A `.github/dependabot.yml` with a `github-actions`
ecosystem would open a PR per bump with the hash and the version comment kept together; without
it, the pins are refreshed by hand. Which does the owner want?

## log
- 2026-10-13T13:43Z created
- 2026-10-13T14:44Z handoff by claude (2dfe7fb0-aa27-4caa-abdc-1dd0125d6691)
- 2026-10-13T14:44Z handoff by claude (2dfe7fb0-aa27-4caa-abdc-1dd0125d6691)
