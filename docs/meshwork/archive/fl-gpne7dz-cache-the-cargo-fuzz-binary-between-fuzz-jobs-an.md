---
id: fl-gpne7dz
title: "Cache the cargo-fuzz binary between fuzz jobs and skip `cargo install cargo-fuzz` on a hit"
category: crate/hardening
labels: [reproducible-floats]
parent: fl-b9qtzty
discovered-from: fl-c8jpmpm
docs:
  - docs/DESIGN.md#hardening-sp-hardening
verify: "contains .github/workflows/ci.yml cache-hit"
status: dropped
created: 2026-11-24T23:46Z
---
The third defect of fl-c8jpmpm, split out so its measurement has a home. That task priced the
install at "one to two minutes" of a two-minute build-only job; neither half holds on ci.yml as
it stands.

Measured on the nightly of 2026-11-24, run 45407465768 at c8093d3, ubuntu-latest, the
`fuzz sum_vs_oracle` leg (job 138492077833): `cargo install cargo-fuzz@0.13.2 --locked` logs
`Finished release profile [optimized] target(s) in 16.34s`, 56 crates, and the whole step
"Install nightly and cargo-fuzz" is 24 s (23 to 31 s in seven of the eight suite runs since
2026-11-02, 50 s in the other). The build-only job the body weighed it against went with the push
trigger in fl-ezhk4cz; the one job left is ten minutes of fuzzing.

What a cached binary would buy:

- 16 s of a leg that runs 651 to 654 s.
- No wall time on the suite: the three legs ended by 14:00:53 and the miri job beside them at
  14:29:29.
- Nothing billed either way: `gh repo view` reports the repository PUBLIC, and
  `actions/runs/45407465768/timing` reports 0 billable ms on all seventeen jobs.
- A restored executable in place of a build from crates.io under cargo-fuzz's own lockfile.

Build it only if the owner wants the 16 s.

## log
- 2026-11-24T23:46Z created
- 2026-11-24T23:46Z open→dropped — Measured on run 45407465768: the install is 16.34 s of a 653 s leg that ends 29 minutes before the miri job, on a public repository that bills nothing; a cached binary buys no suite time and puts a restored executable where a build from crates.io was. Reopen if the owner wants the 16 s.
