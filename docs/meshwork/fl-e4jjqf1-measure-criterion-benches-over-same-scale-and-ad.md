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
    sha: 841c721487cc9edb6fb1a507996729b4d649fbe6f26f64245e7e810785d153b5
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
