---
id: fl-e4jjqf1
title: "Measure — criterion benches over same-scale and adversarial distributions, promotion and allocation rates, bulk deposit per tier scalar vs SIMD, and the differential-dataflow end-to-end SUM against isize"
category: crate/measure
labels: [reproducible-floats, rf-tail]
docs: [docs/DESIGN.md#targets-and-kill-conditions-sp-kills, docs/DESIGN.md#measured-on-a-prototype-sp-measured]
needs: [fl-v6mh9cv, fl-b41y9kt]
seq: 150
verify: "all(exists benches/ops.rs, exists BENCH.md)"
status: done
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
- 2026-10-13T15:16Z handoff by claude (ace5f87f-8122-4167-a02a-0a2238009e67)
- 2026-10-13T15:16Z handoff by claude (ace5f87f-8122-4167-a02a-0a2238009e67)
- 2026-10-13T15:20Z open→doing — claimed by claude (10cb7eaf-2ccb-44b5-b4e4-5396d637aa48)
- 2026-10-13T16:09Z doing→done — verify exit 0 @ 76ee02a+1

## comments
- 2026-10-11T18:13Z [claude (bd2b6c06-c485-4bc5-9de3-2093f3abcd32)] Exact is now a 32-byte Copy word with no heap arm (d33a0ee): there is no promotion. Re-read this body before starting: the promotion-rate measurement becomes the width-cap rate on the same-scale and adversarial fixtures through the checked API, and the allocation rate is zero by construction, so measure bytes and caps, not allocations.
- 2026-10-13T15:14Z [claude (ace5f87f-8122-4167-a02a-0a2238009e67)] Owner's ruling on fl-cb4pxq1 (2026-10-13, "Hold for Measure's row"): this task's bench set needs a deposit_products row per tier, scalar check-then-deposit against the same tier's bulk deposit kernel, because fl-mj3tcta decides the product kernel on that ratio.
- 2026-10-13T16:09Z [claude (10cb7eaf-2ccb-44b5-b4e4-5396d637aa48)] Closed at 76ee02a after three commits: 7792a3f added benches/ops.rs (the prototype table re-taken and the width-cap rates), the products group in benches/simd.rs and benches/dd_sum.rs, with the fixtures shared from benches/support/; fdf0575 added typed exchange beside serialized to dd_sum; 76ee02a recorded everything in BENCH.md. Numbers were taken on this machine (Apple M5 Max, rustc 1.99.0) and on jons-amd-mini (AMD Ryzen 7 5700U, rustc 1.96.1) by the rsync-and-ssh recipe; the NEON ops and simd rows were re-taken in a quiet sitting after the owner flagged another project's benchmarks on the host, and moved under 2%. Verdicts: width kill condition clear (0 caps on same-scale; the same-scale dot-product shape caps 41.8%, a new reading); every SIMD kernel clears 3× over scalar; Binned<3> bulk is 1.05× (NEON) and 1.5× (AVX2) faster than Wide bulk, under the 2× line, so it stays a convenience; every absolute target (compact add, to_f64, Wide add) is missed on both machines (fl-cbtfmqz); the end-to-end SUM is past the 2× kill line on every serialized row (fl-bbevtyv). Peak RSS was measured with /usr/bin/time on both machines; the macOS figure is an allocator high-water mark and BENCH.md says to read the Linux one. The owner's three mid-session instructions are quoted in the session memory: frame the numbers against plain floats, explain the terms, wrap up once recorded.
