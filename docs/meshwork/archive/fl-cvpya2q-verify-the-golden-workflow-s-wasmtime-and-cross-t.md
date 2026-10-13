---
id: fl-cvpya2q
title: "Verify the golden workflow's wasmtime and cross tarballs by checksum, and rule on dependabot for the action hashes"
status: done
category: crate/hardening
labels: [reproducible-floats]
verify: "all(contains .github/workflows/golden.yml sha256sum, lacks .github/workflows/golden.yml /tar xJ$/)"
docs:
  - .github/workflows/golden.yml
seq: 136
created: 2026-10-13T13:43Z
discovered-from: fl-k2383pg
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
- 2026-10-13T14:47Z open→doing — claimed by claude (ace5f87f-8122-4167-a02a-0a2238009e67)
- 2026-10-13T14:51Z doing→done — verify exit 0 @ 8f9b1ea+2

## comments
- 2026-10-13T14:51Z [claude (ace5f87f-8122-4167-a02a-0a2238009e67)] Checksum half landed in 8f9b1ea: WASMTIME_SHA256 a4d6e9e3…9322 from the release API's asset digest, confirmed against a local download; CROSS_SHA256 642375d1…8bad computed from the download because the 2023-02-04 release reports no API digest (the ELF inside embeds 'cross 0.2.5'). Both steps ran locally from the workflow's own text: real digests pass and extract, a flipped digit fails with nothing extracted. Unverified: the workflow on a GitHub runner, since main is not pushed. The dependabot half is its own owner-gated task, fl-7czsn8x.
