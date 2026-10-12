---
id: fl-yzy8bwc
title: "Rule on the dbsp dependency line — the exact pin =0.364.0 the design names, or a caret like differential's"
status: open
category: crate/adapters
labels: [reproducible-floats]
relates: [fl-06sggdc]
docs:
  - docs/DESIGN.md#dbsp-sp-dbsp
seq: 145
created: 2026-10-12T22:39Z
verify: contains docs/meshwork/fl-yzy8bwc-rule-on-the-dbsp-dependency-line-the-exact-pin-0.md /2026-[0-9][0-9]-[0-9][0-9] owner ruled/
---

Owner's call, surfaced from "Ship the dbsp feature" (fl-06sggdc). The feature ships with `dbsp = { version = "=0.364.0", optional = true }`, which is what DESIGN.md § DBSP says: the probe the adapter is written against ran at that version on rustc 1.99.0, and moving the pin means re-running the probe there first. The differential feature is on a caret instead (owner: "0.25.1"), so a consumer's own 0.25.x unifies with it: the impls are a diff type's traits, which that consumer's dataflow has to see as one crate. The same argument applies to dbsp, and it cuts the other way too.

For the caret: a Feldera pipeline is built against one dbsp version, Feldera's own; with an exact pin here, a consumer whose dbsp is 0.365 gets two dbsps in one graph, and `Exact: DBWeight` holds only for the one this crate names, so the UDA fails to compile until this crate re-pins. With `dbsp = "0.364"` cargo unifies to the consumer's 0.364.x; across minors (Feldera releases a minor roughly weekly, and 0.x minors are breaking) it would not unify either way.

For the pin: a caret on a 0.x crate is a caret on a patch range only, and Feldera almost never publishes one: the crates.io index (read 2026-10-12) lists 302 dbsp releases, six of them patches (0.1.1, 0.1.2, 0.31.1, 0.34.1, 0.334.1, 0.334.2), and 0.364.0 has none. Each pin also has a toolchain story of its own (rustc 1.97 and 1.98 ICE inside this one), which the exact pin names and a range would blur.

Options:
(a) Keep `=0.364.0`, as the design says. Re-pin by task, with the probe re-run each time.
(b) `dbsp = "0.364"`, the differential shape, with the design's sentence changed to say the probe runs at the floor of the range.

Recommendation: (a), because on a 0.x crate the two spellings admit the same versions in practice and the exact pin is the one that says which version the probe ran at. The owner writes the line this task's verify looks for, dated.

## log
- 2026-10-12T22:39Z created
