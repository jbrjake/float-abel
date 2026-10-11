---
id: fl-c3ahdwc
title: "Write the outward docs — README (the contract, the trilemma, the tiers table, the not-a-group argument), rustdoc on every public item, SECURITY.md and CHANGELOG.md"
category: crate/docs
labels: [reproducible-floats, rf-tail]
docs: [docs/DESIGN.md#documentation-sp-docs, docs/DESIGN.md#why-the-adaptive-binned-accumulator-is-not-a-group-sp-not-a-group]
needs: [fl-6shtzpj]
seq: 160
verify: "all(exists README.md, contains README.md /not a group/, exists SECURITY.md, exists CHANGELOG.md, exists src/lib.rs, contains src/lib.rs missing_docs)"
status: open
created: 2026-10-11T15:02Z
covers:
  - ref: docs/DESIGN.md#sp-docs
    sha: 7e33ab21417fb13cc9dd4b1853f2e1acb70c1b497b75a9174f779ac844aabb40
  - ref: docs/DESIGN.md#sp-not-a-group
    sha: 373785c4859c6da80c731d8709b693004ccff63b692a79548c0e60e956976708
---
Per DESIGN.md § Documentation. The not-a-group argument is the README's centrepiece, because nobody
else on crates.io makes it. `#![deny(missing_docs)]` in `src/lib.rs`.

## log
- 2026-10-11T15:02Z created
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-docs @7e33ab21417f
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-not-a-group @373785c4859c
