---
id: fl-enkdnvk
title: "Cut the first release: the release PR merged into dev, the dev→main PR merged, the tag and GitHub Release created at the version the conventional-commit log yields, the attested .crate attached and verified with gh attestation verify"
category: crate/release
labels: [reproducible-floats]
seq: 19
parent: fl-m3cymxv
needs: [fl-kpk3gd6, fl-bct35bq, fl-4fcbtkx, fl-d609gg2]
relates: [fl-7en8z60, fl-fm78m8d]
docs:
  - README.md#getting-it
verify: contains CHANGELOG.md /^## \[?[0-9]+\.[0-9]+\.[0-9]+/
status: open
created: 2026-11-02T15:33Z
handoff: |
  2026-11-02, session de9bd423. The release process is built, on origin
  (main = dev = 35d0a77, default branch dev) and proven up to the owner's
  acts; this task is the first cut, and every step left is the owner's
  merge or setting.
  
  Built: release-plz.toml (git_only, features_always_increment_minor, the
  test crates off the tool), .github/workflows/release-plz.yml (release-pr
  job on dev behind a default-branch guard; release job on main that tags,
  releases, packages the .crate from the tag, attests it and uploads
  .crate + .sha256; release-plz 0.4.3 by its tarball's sha256,
  cargo-semver-checks 0.51.0 by install-action), the annotated tag v0.0.0
  on d2f29de as the base, scripts/check-commit-message.sh in the
  commit-msg hook, both gates and ci's commits job, ci on dev, CLAUDE.md
  § Commands, DESIGN.md § Release, README § getting it, CHANGELOG.md's
  header with the [Unreleased] marker release-plz writes under.
  
  Proven: run 41490352546 (main) "Already published - Tag v0.0.0 already
  exists", releases []; run 41490353635 (dev) derives 0.1.0 with
  cargo-semver-checks ✓ and builds the changelog, then 403 at PR
  creation ("GitHub Actions is not permitted to create or approve pull
  requests"), the token's. Locally and on jons-amd-mini release-plz update
  writes 0.1.0 and the [0.1.0] entry.
  
  The sequence to cut 0.1.0, once the owner has done fl-4fcbtkx's items 1
  and 2 (token or Actions-PR setting; flip public): (a) re-run the dev
  release-plz job or push to dev: the release PR "chore: release v0.1.0"
  opens against dev; read it, it carries Cargo.toml 0.1.0, Cargo.lock and
  the CHANGELOG entry; the owner merges it with the merge button (merge
  commit). (b) Right away, before anything else lands on dev: gh pr create
  --base main --head dev --title "chore(release): cut the release the log
  yields"; the owner merges it with the merge button. (c) The release job
  on main tags v0.1.0 on the merged release PR's last commit, publishes
  the GitHub Release with the entry as notes, builds
  float-abel-0.1.0.crate with --locked from that commit, attests it and
  uploads .crate and .sha256. (d) From a clean machine: gh attestation
  verify float-abel-0.1.0.crate --repo jbrjake/float-abel; paste the
  output and the release URL as a comment here; the verify (a released
  heading in CHANGELOG.md) is green on dev after the release PR merge.
  Then fl-cr71vam (the ci semver-checks job against the tag baseline)
  lands.
  
  Watch for: the attest step fails on a private repo (public-only on
  Free), so the flip comes before (b); the main ruleset's required checks
  must name jobs that run on a PR from dev; dependabot PRs #2 and #3 fail
  the commits job on their 'Bump' titles (fl-hp7md5n); crates.io stays out
  of all of this until the owner says.
---

The owner's merges, twice, and the session's watch: release-plz's PR into `dev` bumps Cargo.toml
from 0.0.0 to whatever the conventional-commit log since the start yields under its rules for a 0.x
crate (owner, 2026-11-02: "IT'S NOT V0.1.0 IT'S WHAT THE CONVENTIONAL COMMITS SAY IT IS"); no
version is typed anywhere by hand. The owner merges it; the session opens `dev`→`main`, the owner
merges it; `release-plz release` tags `v<that version>` and creates the release; the attest job
uploads the `.crate`; `gh attestation verify float-abel-<version>.crate --repo jbrjake/float-abel`
passes from a clean machine. crates.io (fl-7en8z60) stays the owner's separate dispatch unless the
ruling moved it in. Done is the verify (a released version heading in CHANGELOG.md) plus the
release URL and the verify command's output in a comment here.

## log
- 2026-11-02T15:33Z created
- 2026-11-02T17:26Z handoff by claude (de9bd423-d8b5-4cf3-8a2c-44d698f4cc3e)
