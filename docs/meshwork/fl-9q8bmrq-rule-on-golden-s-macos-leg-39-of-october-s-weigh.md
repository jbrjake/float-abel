---
id: fl-9q8bmrq
title: "Rule on golden's macOS leg: 39% of October's weighted minutes for a 21-second job, kept, gated on Rust changes, or replaced by the aarch64 Linux witness"
category: rulings
labels: [reproducible-floats]
seq: 60
parent: fl-b9qtzty
docs:
  - docs/DESIGN.md#bit-identical-across-architectures
verify: contains docs/meshwork/fl-9q8bmrq-rule-on-golden-s-macos-leg-39-of-october-s-weigh.md /2026-[0-9][0-9]-[0-9][0-9] owner ruled on the macOS leg/
status: open
created: 2026-11-02T15:14Z
---
Owner, 2026-11-02, on the billing: "WHY ARE WE USING MACOS ... IF IT COSTS 10X MORE". Fuzzing is
not on macOS (fuzz.yml is ubuntu-latest throughout); the one macOS job is golden.yml's `aarch64
macOS` target. It runs 21 to 24 seconds, rounds to one minute, bills ten, and ran 17 times in
October: 1,170 of the month's 2,976 weighted minutes, 39%, for the leg that is a single hash
comparison. Why it exists: tests/golden/main.rs says the committed hash was "computed once on
aarch64 macOS" (the owner's machine), and DESIGN.md § Bit-identical across architectures names it
among the seven targets. What it proves beyond `aarch64 Linux` (ubuntu-24.04-arm, free multiplier):
the aarch64-apple-darwin triple's codegen on the same ISA, and the blessing machine's own target.

The three shapes, the owner's pick:
1. Keep it as is. Public repos pay nothing for standard runners, so the cost is zero after the flip
   and the leg stays the blessing platform's own check.
2. Keep it but only on main pushes whose diff reaches Rust (the `changes` gate task): PRs and docs
   commits stop paying for it; still free after the flip.
3. Drop it and let aarch64 Linux be the aarch64 witness; DESIGN.md's list goes to six targets and
   tests/golden/main.rs's "computed once on aarch64 macOS" stays as provenance.
The recommendation is 2: it is free either way once public, and the gate task is being built
regardless. The verify's sentence names the pick.

## log
- 2026-11-02T15:14Z created
