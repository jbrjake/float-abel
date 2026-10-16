---
id: fl-jqbhm8j
title: "Commit a per-machine bench-baseline.json — the medians and peak RSS criterion measured, written by a script from its estimates and compared at 1.5×, per CLAUDE.md § Performance"
status: open
category: crate/measure
labels: [reproducible-floats]
discovered-from: fl-e4jjqf1
relates: [fl-e4jjqf1]
verify: "all(exists bench-baseline.json, exists scripts/bench-baseline.sh)"
docs:
  - CLAUDE.md#performance
seq: 175
created: 2026-10-13T16:09Z
handoff: |
  Next session: the bench baseline is at the head of `ready` by date, with
  fl-p4dc5w5 (the heap
  arm's budget A/B) and fl-dchdzd4 (the limb cap's edge, the one open
  correctness defect: a decoded
  at-cap value cannot be subtracted from itself, because `big::add` and
  `big::mul` in
  src/exact/big.rs refuse one bit past what a result needs) beside it; the
  publish ruling
  fl-k3g1511 now needs all three and stays out of `ready` by the owner's
  word of 2026-10-16, "there
  is no publish decision until it's working". None of the three has a seq;
  pick by the owner's
  word in session, not by the date order.
  
  Two rulings from the 2026-10-16 session, recorded on fl-k3g1511 and
  fl-p4dc5w5: no publish
  decision until the crate works, and a bench sitting never waits for a
  load-average threshold on
  the owner's dev box ("dude there is nothing else running WHY DO YOU
  INSIST ON SO LITTLE LOAD TO
  RUN YOUR TESTS THIS IS MY DEV BOX????"): check `ps` once for competing
  work, run, and judge by the
  `isize` canaries afterwards. For this task that means the sitting that
  writes the baseline runs
  at once; for fl-p4dc5w5, the control binary is
  `~/.cache/cargo-target/float-abel-control/release/deps/dd_sum-ab4b21425246350c`
  (the 8-row harness
  at 03d96e9) and the variant must be the same harness shape, built from a
  worktree with the
  current lib and 03d96e9's benches/dd_sum.rs, both run back to back with
  `CRITERION_HOME` apart.
  
  What stands at b706675: `Wide` and `Fixed<LSB, LIMBS>` carry
  differential-dataflow's weight traits
  (src/wide/differential.rs, src/fixed/differential.rs: `IsZero`,
  `Semigroup`, `Monoid`, `Abelian`,
  `Multiply<i64>` and `Multiply<isize>` on either side, `From<i8>`,
  `Fixed`'s pinned by the
  `HoldsIntegers` const assertion in src/fixed/ops.rs with
  tests/ui/fixed_from_i8_above_one.rs
  blessed on 1.99.0) and DBSP's (src/wide/dbsp.rs, src/fixed/dbsp.rs:
  `HasZero`, `MulByRef<i64>`
  and `<isize>`, `SizeOf`, `IsNone`, `SupportsRoaring`; the archived form
  is the word itself through
  rkyv's derive with `#[archive(as = "...")]`, `Fixed`'s `limbs` field
  carrying `#[omit_bounds]`;
  `Deserialize` and `OrdRepr` hand-written as a copy and `Ord`'s `cmp`).
  Neither word has
  `Multiply<Self>`, `Mul<Self>` or `HasOne`: no carrier for the product of
  two words, so a `join` of
  two collections weighted by one word and dbsp's
  `neg`/`minus`/`differentiate` on a stream they
  weight do not compile, and the docs say so (DESIGN.md § Wide, § Fixed,
  the trilemma,
  § differential-dataflow, § DBSP; README § The trilemma and § In an
  engine; the crate docs). The
  operators live in src/wide/ops.rs and src/fixed/ops.rs with by-reference
  forms for dbsp's
  blankets. Tests: tests/differential_words.rs (13,
  `differential_wide_*`/`differential_fixed_*`),
  tests/dbsp_words.rs (12; all 22 `dbsp_` green on `cargo +1.99.0 test
  --features dbsp --test main
  dbsp_`), by-reference tests in tests/wide_group.rs and tests/fixed.rs.
  BENCH.md § The end-to-end
  SUM has the `Wide` row at 440ac8c: 4.4–5.1× the `Exact` diff,
  9.5–13.7× `isize`, 179/244 ns an
  update typed/serialized at one worker, 13.64 GB peak RSS on the
  one-worker serialized row; this
  task's baseline should carry that row's medians too. clippy is installed
  on the 1.99.0 toolchain
  now, so `cargo +1.99.0 clippy --features dbsp --all-targets -- -D
  warnings` lints the feature,
  clean. The push gate was not run in that session; the commit gate ran on
  every commit. Main is
  unpushed by the owner's word (fl-4fcbtkx).
---

CLAUDE.md § Performance asks for criterion benches against a committed per-machine
`bench-baseline.json` (median of 7+ runs after warm-up, plus peak RSS) so a regression past 1.5×
is caught and taken to the owner. BENCH.md carries the numbers as prose tables with the machine
and the commit, and nothing in the tree holds them in a form a script can diff. Found by Measure
(fl-e4jjqf1) and left undone there on the owner's instruction to wrap up.

What to build: a `bench-baseline.json` keyed by machine (NEON: Apple M5 Max; AVX2: AMD Ryzen 7
5700U) holding each criterion id's median per element and the end-to-end peak RSS, written from
`target/criterion/**/new/estimates.json` by a script under scripts/ (criterion writes a group
named `a/b` to a directory `a_b`), and a compare mode that prints every row past 1.5× of the
baseline. Advisory, never a gate: the standing decision keeps speed out of the hooks. The
session's scratchpad script that read the estimates is the shape to start from; it is not in the
tree.

## log
- 2026-10-13T16:09Z created
- 2026-10-16T18:13Z handoff by claude (cd1130c2-56bf-432b-bcb1-8ed1c474e8fa)
