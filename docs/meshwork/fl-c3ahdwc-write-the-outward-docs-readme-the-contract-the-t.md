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
handoff: |
  The tree at 76ee02a: Measure (fl-e4jjqf1) is closed, and BENCH.md now
  carries every number the
  kill conditions read, NEON (this machine) and AVX2 (jons-amd-mini) side
  by side, led by a "Terms"
  glossary and a "What reproducibility costs" table of multiples against
  plain f64 and the engine's
  isize diff, which is how the owner reads it ("how much performance do we
  lose for
  reproducibility?"). The README this task writes can cite that section
  for the price of the
  guarantee: Exact one row at a time is 11× a hardware add on NEON and
  22× on AVX2; the SIMD
  kernels sum a column at about 2 ns a value, 5–7× (AVX2) to 19–25×
  (NEON) the vectorised float sum;
  inside differential-dataflow Exact is 2–4× the native integer diff.
  Three rulings for the owner
  sit in `ready` above this task: fl-bbevtyv (the end-to-end ratio is past
  the design's 2× kill
  line on every serialized row), fl-cbtfmqz (every absolute target is
  missed on both machines, with
  the un-inlined operators, the 34-limb carry chain and the shared read
  routine as leads), and
  fl-mj3tcta (the product kernel, now with its row: the scalar product
  path is 7–23× the bulk
  deposit kernel). fl-jqbhm8j files the per-machine bench-baseline.json
  CLAUDE.md § Performance
  asks for. The push gate was not re-run after the bench commits (7792a3f,
  fdf0575, 76ee02a); the
  commit gate passed on each, and `cargo clippy --workspace --all-targets
  -- -D warnings` was green
  on the bench targets before 7792a3f. main is unpushed, so every CI claim
  stays unverified until
  the owner pushes. The owner's instruction at the end of that session was
  to wrap up once the
  numbers were recorded and explained, not to start new work.
  
  The brief below is the one written before Measure ran and still holds.
  
  The tree at 31dfbe8 has the push gate green end to end
  (`./scripts/regression.sh`, sandbox off
  for cargo-deny), with the tarball digests in golden.yml, dependabot on
  the action hashes (owner:
  "yes have dependabot open prs that update the pinned hashes"), and the
  compile-fail wording
  pinned to stable 1.99 with rust-src (tests/ui.rs, BLESSED_RUSTC; any
  other compiler skips the
  compile-fail cases and the gate says so). main is some eighty commits
  ahead of origin and
  unpushed, so every CI claim is unverified until the owner pushes.
  
  This task per DESIGN.md § Documentation (line 594) and § Why the
  adaptive binned accumulator is
  not a group (line 81), both covered by hash on this task; re-read them
  first, and `spec audit
  docs/DESIGN.md` says whether either drifted.
  
  What is there: README.md is 1.9 KB, a two-paragraph pitch, the build
  commands and the licence,
  and it points at DESIGN.md for the not-a-group argument instead of
  making it. SECURITY.md and
  CHANGELOG.md do not exist. src/lib.rs has `#![no_std]`,
  `#![deny(unsafe_code)]` and a doc_cfg
  line, no `missing_docs`. On 2026-10-13 `RUSTDOCFLAGS="-W missing_docs"
  cargo doc --no-deps`
  printed no warning, so every public item may already carry rustdoc;
  prove it by adding
  `#![deny(missing_docs)]` under `#![deny(unsafe_code)]` and running
  `cargo clippy --workspace
  --all-targets -- -D warnings` plus `RUSTDOCFLAGS="-D warnings" cargo doc
  --no-deps` on every
  feature set the gate builds (the lib alone with `--no-default-features`,
  the default, and the
  dbsp feature on `+1.99.0`), since a feature-gated item is only linted
  when its feature is on.
  
  What to write: the README's contract (same bits under any order, worker
  count or architecture:
  DESIGN.md's opening and § Bit-identical across architectures), the
  trilemma (DESIGN.md names
  it; quote its three corners, not a paraphrase), the tiers table (Fixed,
  Wide, Exact, and Binned
  as batch-only; widths, what each refuses, which engine traits each
  implements), and the
  not-a-group argument as the centrepiece, in the README's own words with
  the mechanism from
  § sp-not-a-group: retraction after promotion lands different bits, so
  the adaptive accumulator
  has no inverse. Keep the build block (it is what CLAUDE.md § Commands
  says, plus the ui_ line
  now) and the CARGO_TARGET_DIR note. CHANGELOG.md: Keep a Changelog
  shape, one `Unreleased`
  section listing what 0.1.0 will ship (DESIGN.md § Release lists it), no
  dates on unreleased
  work. SECURITY.md: the owner ruled "GitHub reporting only" (the address
  ruling, closed, is in
  this task's relates): the file points at
  GitHub's private vulnerability reporting on the repository (Security →
  Advisories → "Report a
  vulnerability") and carries no email; switching the form on in the
  repository settings is the
  owner's act, so say in the file that reports go there and note in the
  handoff whether the
  switch has been confirmed.
  
  The verify holds the README to the literal "not a group", lib.rs to
  `missing_docs`, and the two
  files to existing. Commit the docs apart from the store, and run the
  push gate before closing:
  rustdoc with `-D warnings` is in it and a broken intra-doc link fails
  there. The rest of the
  queue is rulings and the Measure task, which `prime` lists; Measure's
  body says to re-read it
  before starting, and the AVX-512 task needs a machine with the flag,
  which neither x86 box has.
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
