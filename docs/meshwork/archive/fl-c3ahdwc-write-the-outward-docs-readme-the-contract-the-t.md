---
id: fl-c3ahdwc
title: "Write the outward docs — README (the contract, the trilemma, the tiers table, the not-a-group argument), rustdoc on every public item, SECURITY.md and CHANGELOG.md"
category: crate/docs
labels: [reproducible-floats, rf-tail]
docs: [docs/DESIGN.md#documentation-sp-docs, docs/DESIGN.md#why-the-adaptive-binned-accumulator-is-not-a-group-sp-not-a-group]
needs: [fl-6shtzpj]
seq: 160
verify: "all(exists README.md, contains README.md /not a group/, exists SECURITY.md, exists CHANGELOG.md, exists src/lib.rs, contains src/lib.rs missing_docs)"
status: done
created: 2026-10-11T15:02Z
covers:
  - ref: docs/DESIGN.md#sp-docs
    sha: 904a3906a40e5620196bf7a894da6ee75359c0c3f54782893bcd7c020353c95d
  - ref: docs/DESIGN.md#sp-not-a-group
    sha: 373785c4859c6da80c731d8709b693004ccff63b692a79548c0e60e956976708
relates: [fl-pmqp4rh]
---
Per DESIGN.md § Documentation. The not-a-group argument is the README's centrepiece, because nobody
else on crates.io makes it. `#![deny(missing_docs)]` in `src/lib.rs`.

## log
- 2026-10-11T15:02Z created
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-docs @7e33ab21417f
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-not-a-group @373785c4859c
- 2026-10-13T15:11Z handoff by claude (ace5f87f-8122-4167-a02a-0a2238009e67)
- 2026-10-13T15:11Z handoff by claude (ace5f87f-8122-4167-a02a-0a2238009e67)
- 2026-10-13T15:14Z handoff by claude (ace5f87f-8122-4167-a02a-0a2238009e67)
- 2026-10-13T15:15Z handoff by claude (ace5f87f-8122-4167-a02a-0a2238009e67)
- 2026-10-13T16:09Z handoff by claude (10cb7eaf-2ccb-44b5-b4e4-5396d637aa48)
- 2026-10-13T16:10Z handoff by claude (10cb7eaf-2ccb-44b5-b4e4-5396d637aa48)
- 2026-10-14T16:43Z handoff by claude (16e02ca3-15ad-4c80-bfb4-53e89fee757d)
- 2026-10-14T16:43Z handoff by claude (16e02ca3-15ad-4c80-bfb4-53e89fee757d)
- 2026-10-14T21:21Z open→doing — claimed by claude (4e0e29d7-b85d-4a99-93e3-a9a5c00bb168)
- 2026-10-14T21:21Z cover docs/DESIGN.md#sp-docs @904a3906a40e
- 2026-10-14T21:26Z doing→done — verify exit 0 @ 1621409+1
