---
id: fl-7en8z60
title: "Run the first publish, 0.1.0, by dispatching release.yml with a crates.io token, then move the crate to Trusted Publishing"
category: crate/release
labels: [reproducible-floats]
seq: 180
discovered-from: fl-k3g1511
needs: [fl-5qbt792]
relates: [fl-x2k7fbb, fl-fm78m8d]
docs:
  - docs/DESIGN.md#release-sp-release
verify: contains docs/meshwork/fl-7en8z60-run-the-first-publish-0-1-0-by-dispatching-relea.md /2026-[0-9][0-9]-[0-9][0-9] owner published 0.1.0/
status: blocked
created: 2026-10-20T15:04Z
blocked-reason: "owner, 2026-10-16, in session: \"i don't want to publish yet\"; every step is the owner's act, and DESIGN.md § Release's ship list still lacks the AVX-512 kernel (fl-qv0vh39)"
---
The owner's act, every step. Held on the owner's word of 2026-10-16, "i don't want to publish
yet", and DESIGN.md § Release's list of what 0.1.0 ships with (the SIMD kernels are still short
the AVX-512 one, fl-qv0vh39).

1. On crates.io, a new API token scoped to publish-new and publish-update, crate `float-abel`;
   set it as the repository secret `CARGO_REGISTRY_TOKEN`. The first publish is the name
   reservation.
2. `version = "0.1.0"` in `Cargo.toml` and `Cargo.lock`, committed; tag `v0.1.0`, signed if
   release-sig has landed; push the tag.
3. Dispatch `release.yml` with `tag=v0.1.0`, `auth=token`; watch the run green and the crate on
   crates.io with its attestation on the GitHub release.
4. On crates.io, Settings → Trusted Publishing → GitHub: owner `jbrjake`, repository
   `float-abel`, workflow `release.yml`, environment `release`. Then delete the token on
   crates.io and the secret in the repository, so no long-lived credential survives.
5. Write the line this task's verify looks for, dated.

## log
- 2026-10-20T15:04Z created
- 2026-10-20T15:05Z open→blocked — owner, 2026-10-16, in session: "i don't want to publish yet"; every step is the owner's act, and DESIGN.md § Release's ship list still lacks the AVX-512 kernel (fl-qv0vh39)
