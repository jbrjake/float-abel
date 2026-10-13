---
id: fl-e4jjqf1
title: "Measure — criterion benches over same-scale and adversarial distributions, promotion and allocation rates, bulk deposit per tier scalar vs SIMD, and the differential-dataflow end-to-end SUM against isize"
category: crate/measure
labels: [reproducible-floats, rf-tail]
docs: [docs/DESIGN.md#targets-and-kill-conditions-sp-kills, docs/DESIGN.md#measured-on-a-prototype-sp-measured]
needs: [fl-v6mh9cv, fl-b41y9kt]
seq: 150
verify: "all(exists benches/ops.rs, exists BENCH.md)"
status: open
created: 2026-10-11T15:02Z
covers:
  - ref: docs/DESIGN.md#sp-kills
    sha: 266f6a0fde1bc6b41fb9520cfa5ae45a331824afea6a57dc86591c019f7edd5c
  - ref: docs/DESIGN.md#sp-measured
    sha: 89ea84a0312ebdcc5029dad6491cfdafdf63357bd90ce2e32b1f7ef460c515ec
handoff: |
  The tree at 4929c80 has the push gate green end to end
  (`./scripts/regression.sh`, sandbox off for
  cargo-deny) and nothing pushed: main is some eighty-five commits ahead
  of origin, so every CI
  claim is unverified until the owner pushes. This session landed the
  tarball digests in
  golden.yml, dependabot on the action hashes, the compile-fail wording
  pinned to stable 1.99 with
  rust-src (tests/ui.rs; any other compiler skips the compile-fail cases
  and the gate says so), and
  dbsp on a caret, `"0.364"`, by the owner's ruling. The owner's other two
  rulings this session:
  the SIMD product kernel waits on this task's `deposit_products` row (the
  follow-up that needs this
  task carries the question), and SECURITY.md points at GitHub's private
  reporting with no email.
  
  Re-read this task's body and both comments before starting: Exact is a
  32-byte Copy word with no
  heap arm, so there is no promotion rate and no allocation rate to
  measure; it is the width-cap
  rate through the checked API on the same-scale and adversarial fixtures,
  bytes and caps. The bench
  set needs a `deposit_products` row per tier, scalar check-then-deposit
  against the same tier's
  bulk `deposit` kernel, on the same corpus and machine, because the
  product-kernel decision is
  made on that ratio.
  
  What exists: benches/simd.rs is the only bench target (`[[bench]] name =
  "simd"`, `harness =
  false`, `required-features = ["simd"]`, criterion 0.8.2 with no default
  features), and the
  verify wants benches/ops.rs and BENCH.md. CLAUDE.md § Performance asks
  for a committed
  per-machine baseline, a median of seven or more runs after warm-up, and
  peak RSS; the NEON numbers
  come from this machine and the AVX2 numbers from jons-amd-mini (the
  rsync and ssh recipe is in
  the session memory, not the tree; the owner named the machine in
  session). Numbers are never
  gated: a missed target is a design question for the owner.
  
  The outward-docs task, the other build task in `ready`, carries its own
  handoff written this
  session and does not depend on this one.
---
Per DESIGN.md § Targets and kill conditions and § Measured, on a prototype. `BENCH.md` names the
machine and the commit for every number. It re-takes the prototype table from the crate and reports
the SIMD targets with NEON (Apple M5 Max) and AVX2 (the x86 runner) side by side, never blended. The
end-to-end run uses the differential integration harness on 10⁷ updates, at 1 and 8 workers.

Advisory, never a gate. The kill conditions read from here.

## log
- 2026-10-11T15:02Z created
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-kills @841c721487cc
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-measured @89ea84a0312e
- 2026-10-11T18:13Z cover docs/DESIGN.md#sp-kills @266f6a0fde1b
- 2026-10-11T18:13Z cover docs/DESIGN.md#sp-measured @89ea84a0312e
- 2026-10-13T15:16Z handoff by claude (ace5f87f-8122-4167-a02a-0a2238009e67)
- 2026-10-13T15:16Z handoff by claude (ace5f87f-8122-4167-a02a-0a2238009e67)

## comments
- 2026-10-11T18:13Z [claude (bd2b6c06-c485-4bc5-9de3-2093f3abcd32)] Exact is now a 32-byte Copy word with no heap arm (d33a0ee): there is no promotion. Re-read this body before starting: the promotion-rate measurement becomes the width-cap rate on the same-scale and adversarial fixtures through the checked API, and the allocation rate is zero by construction, so measure bytes and caps, not allocations.
- 2026-10-13T15:14Z [claude (ace5f87f-8122-4167-a02a-0a2238009e67)] Owner's ruling on fl-cb4pxq1 (2026-10-13, "Hold for Measure's row"): this task's bench set needs a deposit_products row per tier, scalar check-then-deposit against the same tier's bulk deposit kernel, because fl-mj3tcta decides the product kernel on that ratio.
