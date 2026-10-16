---
id: fl-4fcbtkx
title: "Push main to origin and make the repo readable by leras's CI — git push origin main through the push gate, then flip jbrjake/float-abel public or add it to leras's ORESEUR_TOKEN PAT, both the owner's acts"
status: open
category: rulings
labels: [reproducible-floats]
answers: leras#le-yxhtwgw
docs:
  - CLAUDE.md#standing-decisions
  - docs/DESIGN.md#release-sp-release
seq: 56
created: 2026-10-16T16:51Z
verify: contains docs/meshwork/fl-4fcbtkx-push-main-to-origin-and-make-the-repo-readable-b.md /2026-[0-9][0-9]-[0-9][0-9] owner pushed main to origin/
---

Answers leras#le-yxhtwgw, whose body names both steps as owner actions on this repo. leras ships
its float sums lane against float-abel rev 75c25fe as a path dep on the sibling checkout, with the
git line beside it for the swap; main here is 147 commits ahead of origin/main, which holds the
initial commit alone, and jbrjake/float-abel is private, so the pin resolves for neither cargo
nor leras's Actions runner.

The two steps, in order:
1. `git push origin main` from this checkout, through the push gate (.githooks/pre-push runs
   scripts/regression.sh: strict lint, build, every test with --include-ignored, the release
   budget tests, cargo-deny on both closures; the gate needs the sandbox off and a warm target).
   leras then pins whatever origin/main holds; the surface it uses is Wide::{deposit, limbs,
   from_parts, specials, to_f64, div_f64, LIMBS}, Fixed256::{try_from(f64), limbs, from_limbs,
   to_f64, div_f64, quantize, LIMBS}, OutOfWindow and the std ops, so a rename there is a compile
   error at the swap, not a silent change.
2. Make the repo fetchable from leras's runner: flip it public (the plan's posture,
   PLAN-reproducible-floats.md §3.1), or add jbrjake/float-abel to the ORESEUR_TOKEN
   fine-grained PAT's repository list (leras docs/CI-CREDENTIALS.md).

Neither is a session's act: the push publishes 147 commits and the visibility change is a
GitHub setting on the owner's account. When both hold, this task's comment carries
"<date> owner pushed main to origin and made the repo readable by leras's CI", the verify goes
green, and leras's swap task waits only on leras closing its ask.

## log
- 2026-10-16T16:51Z created
