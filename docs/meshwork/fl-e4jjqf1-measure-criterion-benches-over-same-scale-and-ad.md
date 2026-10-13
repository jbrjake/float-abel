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

## comments
- 2026-10-11T18:13Z [claude (bd2b6c06-c485-4bc5-9de3-2093f3abcd32)] Exact is now a 32-byte Copy word with no heap arm (d33a0ee): there is no promotion. Re-read this body before starting: the promotion-rate measurement becomes the width-cap rate on the same-scale and adversarial fixtures through the checked API, and the allocation rate is zero by construction, so measure bytes and caps, not allocations.
- 2026-10-13T15:14Z [claude (ace5f87f-8122-4167-a02a-0a2238009e67)] Owner's ruling on fl-cb4pxq1 (2026-10-13, "Hold for Measure's row"): this task's bench set needs a deposit_products row per tier, scalar check-then-deposit against the same tier's bulk deposit kernel, because fl-mj3tcta decides the product kernel on that ratio.
