---
id: fl-821gnyr
title: "Bring CLAUDE.md under the baseline's 4 KB: it is 11 KB, and its standing decisions carry history clauses the git log already holds"
status: open
category: crate/release
labels: [reproducible-floats]
verify: lacks CLAUDE.md /zip 6/
docs:
  - CLAUDE.md#standing-decisions
seq: 190
created: 2026-11-02T15:25Z
---

The baseline's § This file: standing decisions each with a one-clause why, current state only, no dates, history or incident stories, under 4 KB. CLAUDE.md is 11 KB on 2026-11-02. The dbsp bullet alone narrates three successive rulings ("chose Caret 0.364 over the exact pin, then Bump dbsp to 0.367 past zip 6's advisories"); the current state is the caret, the floor and the two deny configs. Keep every owner quote that is a live ruling; cut the sequence of how it got there, and the duplicated whys (zero-copy is stated in two bullets).

## log
- 2026-11-02T15:25Z created
