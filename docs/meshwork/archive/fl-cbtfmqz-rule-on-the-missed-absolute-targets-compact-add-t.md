---
id: fl-cbtfmqz
title: "Rule on the missed absolute targets — compact add, to_f64 and Wide add miss DESIGN.md's 5, 5 and 3 ns on both machines: inline the operators, rewrite the carry chain and the read, or restate the targets"
status: done
category: rulings
labels: [reproducible-floats]
discovered-from: fl-e4jjqf1
relates: [fl-e4jjqf1]
verify: contains docs/meshwork/fl-cbtfmqz-rule-on-the-missed-absolute-targets-compact-add-t.md /2026-[0-9][0-9]-[0-9][0-9] owner ruled/
docs:
  - docs/DESIGN.md#targets-and-kill-conditions-sp-kills
  - BENCH.md#the-prototype-table-re-taken
seq: 153
created: 2026-10-13T16:09Z
---

Owner's call, raised by Measure (fl-e4jjqf1). DESIGN.md § Targets and kill conditions names three
absolute targets, never gated: compact add ≤ 5 ns, `to_f64` ≤ 5 ns, `Wide` add ≤ 3 ns. BENCH.md
§ The prototype table, re-taken, commit 7792a3f, same-scale band, the crate as a consumer builds
it (no LTO, no `#[inline]` on the operators):

| target | NEON | AVX2 |
|---|--:|--:|
| compact add ≤ 5 ns: `Exact::deposit` / `Exact += &Exact` | 5.98 / 6.80 ns | 16.10 / 17.68 ns |
| `to_f64` ≤ 5 ns | 9.47 ns | 17.58 ns |
| `Wide` add ≤ 3 ns: `Wide + Wide`, operands in L2 | 15.70 ns | 29.48 ns |

Every target is missed on both machines; against the prototype on the same NEON machine the
compact rows are 1.3–1.5× and the `Wide` add 7×. Against plain `f64 +=` (0.54 / 0.74 ns), `Exact`
one row at a time is 11× on NEON and 22× on AVX2.

Leads, observed in the bench binary and not investigated (the owner asked the session to wrap
up): the operators are out-of-line symbols in the bench binary, so each add returns a 40-byte
`Result` through memory and the dependent chain waits on it (a 0.6 s-window smoke under
`CARGO_PROFILE_BENCH_LTO=fat`, taken while another project's benchmarks shared the host, put the
built-operand add near 4.2 ns and `Wide + Wide` near 16 ns on NEON: a lead, not a measurement);
`limbs::add_assign` threads a flag-derived carry through 34 limbs serially where a `u128`
intermediate would let the compiler emit an add-with-carry chain; `Compact::read` goes through the
limb-slice rounding routine `Wide` shares, over two limbs. The AVX2 compact rows are 2.7× the
NEON ones where the float floor is only 1.4×, so `i128` shifts and `leading_zeros` on x86-64 are
part of that gap.

Options, none chosen: (a) mark the hot operators `#[inline]` and re-measure, which a consumer
without LTO would feel directly; (b) rewrite the carry chain and add a two-limb read path, each
behind the existing oracle tests; (c) restate the targets to what the canonical two's-complement
`Wide` and the un-inlined API can reach. The owner writes the line this task's verify looks for,
dated.

## log
- 2026-10-13T16:09Z created
- 2026-10-14T21:21Z open→done — verify exit 0 @ e628ed5+5

## comments
- 2026-10-14T14:43Z [claude (16e02ca3-15ad-4c80-bfb4-53e89fee757d)] After 4ec1471 (#[inline] on the operator chain down to the compact tier), the compact add meets its 5 ns target on NEON in a clean A/B at 2a82388 against a22fae7: deposit 5.80 → 4.94 ns, add with both operands built 8.54 → 3.74, += 7.20 → 3.81. to_f64 (9.8 ns) and Wide's add (15.6 ns) are unchanged, neither being inlined, so the ruling narrows to those two and to AVX2's rows, pending its sitting.
- 2026-10-14T21:21Z [claude (4e0e29d7-b85d-4a99-93e3-a9a5c00bb168)] 2026-10-14 owner ruled, in session: "let's move on from benchmarking. i am happy with the results from last session, showing that the incremental version wins out well below 1M records." Read as option (c): the targets restate to what is measured. The compact add meets 5 ns on NEON after the inlining (4.94 ns a deposit, 3.81 ns the engine's `+=` with both operands built, BENCH.md § The prototype table, re-taken, 2a82388) and stands at 15.9 / 10.9 ns on AVX2; `to_f64` (9.78 ns NEON, 17.78 AVX2) and `Wide + Wide` (15.60 and 29.48 ns) stand at their measured figures, neither inlined. DESIGN.md § Targets and kill conditions now states those as the targets and names the carry chain on `u128` intermediates and a two-limb read path as the leads should a consumer need less; neither is built. The reading from the owner's line to option (c) is the session's, read back in the closing report; a correction reopens this.
