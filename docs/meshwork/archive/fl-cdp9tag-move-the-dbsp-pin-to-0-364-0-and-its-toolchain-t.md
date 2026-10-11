---
id: fl-cdp9tag
title: Move the dbsp pin to 0.364.0 and its toolchain to rustc 1.99.0
status: done
category: crate/build
relates: [fl-06sggdc]
verify: "all(contains .github/workflows/ci.yml /rustup toolchain install 1\\.99\\.0/, contains CLAUDE.md /\\+1\\.99\\.0/, contains docs/DESIGN.md /=0\\.364\\.0/, lacks docs/DESIGN.md /builds only on rustc 1\\.96\\.1/, lacks CLAUDE.md /1\\.96\\.1/)"
docs:
  - docs/DESIGN.md#dbsp-sp-dbsp
  - docs/DESIGN.md#crate-posture-sp-posture
  - CLAUDE.md
seq: 15
created: 2026-10-11T16:05Z
---

Owner ruling, this session: "yes move to the new rust and dbsp vers that work together."

The probe on the dbsp task shows dbsp 0.356.0 and 0.364.0 both build on rustc 1.96.1 and 1.99.0
and ICE on 1.97.x and 1.98-beta. So: DESIGN.md § Crate posture and § DBSP name dbsp 0.364.0 and
rustc 1.99.0, the Features table says dbsp 0.364 with the companion crates it resolves, CLAUDE.md's
dbsp command and the CI dbsp job pin 1.99.0, README's command matches, and the dbsp task's title
and verify name the new versions. The design's probe record stays as what was measured; the dbsp
task re-runs it at the pin before the feature ships.

## log
- 2026-10-11T16:05Z created
- 2026-10-11T16:05Z open→doing — claimed by claude (85c2609c-6abc-4455-b7f3-6c1c79eda92c)
- 2026-10-11T16:06Z doing→done — verify exit 0 @ b863aed+3
