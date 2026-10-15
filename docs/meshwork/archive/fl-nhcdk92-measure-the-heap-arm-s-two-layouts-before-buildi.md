---
id: fl-nhcdk92
title: "Measure the ruled 32-byte niche form of the heap arm before building it — against the Copy word, the banded idiom and the 40-byte enum on the cost probe's three bands and under serialized exchange, settling the block shape and the budget the heap task must meet"
category: crate/types
labels: [reproducible-floats, rf-critical-path]
discovered-from: fl-t154dsp
seq: 10
docs:
  - docs/OVERFLOW.md#cost
  - docs/DESIGN.md#exact-sp-exact
status: done
created: 2026-10-15T15:33Z
verify: contains docs/meshwork/fl-nhcdk92-measure-the-heap-arm-s-two-layouts-before-buildi.md /^- 20[0-9][0-9]-[0-9][0-9]-[0-9][0-9]T[0-9:]+Z \[[^]]+\] Layout findings:/
attachments: [attachments/fl-nhcdk92/probe-Cargo.toml, attachments/fl-nhcdk92/sitting-1.log, attachments/fl-nhcdk92/sitting-2.log, attachments/fl-nhcdk92/probe-src-bin-cost-main.rs, attachments/fl-nhcdk92/probe-src-bin-cost-niche.rs, attachments/fl-nhcdk92/probe-src-bin-cost-big.rs, attachments/fl-nhcdk92/probe-src-bin-cost-proxies.rs]
---

Owner, 2026-10-15, in session: "yes let's do heap and allow product fols inside a dataflow. but i
like your option 4 so if there's a way to get the best of both worlds, i'm all for it." And on
the layout, same session: "yes i'm okay with one more unsafe to save that many bits on every
value". So the form is ruled, the 32-byte niche form; what this task settles is the budget and
the block.

The heap-arm proxy in docs/OVERFLOW.md § Cost ran 37% over the plain 32-byte `Exact` on the
same-scale band, where it never promotes, so that cost is the shape alone: an enum tag, a 40-byte
update through differential's sort against 32, and a non-`Copy` clone. Build the niche form as a
newtype in the scratch crate (`cost`, attached to fl-t154dsp; copy it into a scratchpad crate
with float-abel as a path dependency and `CARGO_TARGET_DIR` outside ~/Documents; raw pointers
are fine in a probe): today's word for the compact form, an even nonzero `lo` carrying the limb
count and `hi` the pointer for the heap form, `Clone` a copy on the compact path, `Drop` freeing
the block, `plus_equals` through `checked_add` first with promotion to a `Wide`-shaped block on
refusal and demotion after. Try two block shapes, a bare `Box<[u64]>` of limbs with `exp` and
the amounts inline, and one block holding a header and the limbs, and keep the faster.

Run it against plain `Exact`, the banded idiom and the 40-byte enum proxy on the three bands,
one worker, with a serialized-exchange column (`CommunicationConfig::ProcessBinary`, the probe's
shape), since a heap form serializes as its limbs. Ten runs each, the fastest kept, a quiet host
(load under 3, `uptime` and `ps aux | grep cargo` first). The same-scale column against plain
`Exact` is the budget fl-th7g69s must meet; note what the form does to `size_of` of a
differential update tuple (unchanged, 48 bytes with a `u64` key) and to the wire form.

Close with a comment opening "Layout findings:" carrying the table, the block shape chosen and
the budget. The heap task needs this one.

## log
- 2026-10-15T15:33Z created
- 2026-10-15T15:33Z handoff by claude (6cbb3399-1134-4aff-94e4-f8c1137cf60f)
- 2026-10-15T15:36Z handoff by claude (6cbb3399-1134-4aff-94e4-f8c1137cf60f)
- 2026-10-15T15:39Z open→doing — claimed by claude (3f239526-744d-4f69-8242-cd45409a169a)
- 2026-10-15T16:22Z doing→done — verify exit 0 @ 02adc7c+7

## comments
- 2026-10-15T15:38Z [claude (6cbb3399-1134-4aff-94e4-f8c1137cf60f)] Owner, 2026-10-15, in session: the whole readback on fl-t154dsp is ratified ("yes confirmed"), so next session starts building here with no ruling outstanding.
- 2026-10-15T16:21Z [claude (3f239526-744d-4f69-8242-cd45409a169a)] Layout findings: the 32-byte niche form costs 1.11× the Copy word on same-scale data typed and 1.17–1.20× serialized, where the 40-byte enum cost 1.32× and 1.36×; the two block shapes tie on that column and split the promoting bands by under 5% each way, so the heap task builds shape B, the header in the block; the budget is below.
  
  The probe (`cost`, attached, with its Cargo.toml and both sitting logs): the fl-t154dsp probe's rows plus `N<INLINE, DROP>`, the niche form as a newtype. The word is `repr(C)` `{lo, hi, exp, nan, pos, neg}`, the compact `Exact` bit for bit when `lo` is odd or zero (transmuted, so the compact path runs `Exact::checked_add` and `checked_mul_i64` unchanged), the heap form when `lo` is even and nonzero: `lo` the limb count, `hi` the pointer to one allocation. Shape A keeps the sign (`lo` bit 1), the exponent and the three counts in the word and the block is the limbs alone; shape B puts a three-limb header (sign, exponent, counts) in front of the limbs and zeroes the word's trailing sixteen bytes. The heap magnitude is sign-magnitude, normalised (odd, no leading zero limb), so a value fits the compact word iff its bit length is at most 127, and every heap result lands compact or allocates a fresh block (no capacity reuse; the working form is a stack buffer of 64 limbs). The compact fast paths take what the real arm will take: `ptr::write` over a compact self in `plus_equals` and `negate`, `mem::forget` of a compact by-value self in `multiply`, `clone_from` as a copy when both sides are compact, `is_zero` as the bitwise zero word (a heap word has a nonzero `lo`), `eq` by the word first. Its own tests pass on both shapes: `1e300 + 1e-300` promotes and `− 1e-300` demotes to `Exact::from(1e300)` structurally, `0.1 + 0.2` stays compact and equals the `Exact` sum, the five histories read the small value, a 2,000-row 2^±300 fold with retractions equals a `Wide` oracle at every step and cancels to the zero word, a heap value round-trips through bincode, `multiply` by −3 equals three negated adds, `Ord` holds across forms.
  
  The sitting: Apple M5 Max (18 cores), the probe built `--release` against float-abel at 02adc7c, differential-dataflow 0.25.1, timely 0.31.0; 10⁶ updates over 16 keys in one epoch, one worker; typed is `execute_directly` (timely's Thread allocator), serialized is `execute` with `CommunicationConfig::ProcessBinary(1)`; ten runs each, the fastest kept, run twice with the host's one-minute load under 3 at the start (2.69 and 2.78; the owner's desktop apps held it at 3.5–5.5 for the twenty minutes before each window). The second sitting is the table; the first (sitting-1.log, before `clone_from` and the bitwise `is_zero`) agrees within 2 ns on every cell. ns an update:
  
  | diff | same-scale typed | same-scale serialized | spread 2^±60 typed | spread 2^±60 serialized | mixed 2^±300 typed | mixed 2^±300 serialized |
  |---|--:|--:|--:|--:|--:|--:|
  | `isize` | 11.0 | 19.2 | | | | |
  | `Exact`, plain (the Copy word) | 22.9 | 37.9 | panics | panics | panics | panics |
  | `Exact`, banded, with the reduce | 23.4 | 41.5 | 48.9 | 67.2 | 42.0 | 59.7 |
  | 40-byte enum (`H`, the earlier proxy) | 30.3 | 51.5 | 99.5 | 120.4 | 117.9 | 138.8 |
  | niche A, header in the word | 25.6 | 44.4 | 72.8 | 93.4 | 82.6 | 104.6 |
  | niche B, header in the block | 25.5 | 45.4 | 69.8 | 91.6 | 85.7 | 106.9 |
  | niche A with an empty `Drop` (diagnostic, leaks) | 25.2 | 45.1 | | | | |
  | the Copy word plus the tag test alone (diagnostic) | 23.3 | 40.2 | | | | |
  | `Wide` as the diff | 171.8 | 236.2 | 178.9 | 236.9 | 179.4 | 236.9 |
  
  The run spreads are in the log (min, median, max of the ten): the niche rows sit within 0.5 ms of their medians on typed same-scale; the Copy word's serialized minimum, 37.9, is 2 ms under its median of 39.9, so the serialized ratios read 1.17× on medians (46.7 against 39.9) and 1.20× on minima.
  
  What the diagnostics say about the same-scale gap, typed: the tag test alone is 0.4 ns (2%); the drop glue is 0.3 ns (1–2%, niche A against the empty-`Drop` word); the remaining 1.9 ns (8%) is the non-`Copy` shape itself, a `Clone` that is a branch and a copy rather than a copy, `clone_from` and the moves through consolidation's sort and the batcher's merges on a type with drop glue. No cheap path in the word removes it: a pointer-owning word is not `Copy`. Serialized exchange adds the same shape cost on top of an unchanged 33-byte compact encoding.
  
  On the promoting bands the niche form runs 0.70–0.76× the enum proxy (a shorter limb add and a smaller block against `Box<Wide>`'s 34 limbs) and 1.4–2.0× the banded idiom, which never promotes; that is the probe's fresh-block-per-result heap path, an upper bound on the real arm's, which can add in place when a heap self's block already holds the result's length.
  
  Block shape: B. A against B is 0.1 ns on the budget column, B ahead by 3–4% on 2^±60 and A by 2–4% on 2^±300, inside a sitting's spread. B is the design's shape: the heap form's special amounts are dyadic and in a product chain as wide as the finite part, so they are variable-length limb runs that cannot live in the word's sixteen trailing bytes under any shape, and B keeps the whole heap value in one block with one layout: a header of four (sign, exponent, length) for the finite part and the three amounts, then four limb runs. The word for a heap value is then `lo = length << 1` (even, nonzero), `hi` the pointer, and the four trailing fields zero, which the decoder and `Eq` can demand, and the compact word is untouched.
  
  Sizes: a differential update `((u64, ()), u64, diff)` is 48 bytes with the Copy word and with either niche shape, 56 with the enum, 312 with `Wide` (the probe's `update_tuple_sizes` test). Wire: the compact encoding is unchanged at 33 bytes; a heap value crosses as a tagged variable-length body (22 bytes plus 8 a limb in the probe's encoding; fl-mrdqpay's is the design's). On the SUM idiom only single-double compact diffs are exchanged, since `count_total` carries the totals as data, so the serialized column prices the compact encoding plus the shape; a heap encoding is paid only where a heap diff crosses, a join on promoted weights or a multi-worker consolidation.
  
  The budget for fl-th7g69s: in one sitting against a control build of the Copy word (the A/B recipe in the build-environment memory: the control's `dd_sum` binary saved before src changes, `CRITERION_HOME` apart), the heap-armed `Exact`'s one-worker same-scale `dd_sum` rows must be within 1.12× the control typed and 1.20× serialized (the worse shape's minima, rounded up; 1.17× on medians). That is more than the "few percent" the heap task's body assumed, and the diagnostics say why: two points are the tag test, one or two the drop glue, eight the non-`Copy` shape, none of them removable by the word's layout. A row past the budget is a design question for the owner, not a build to ship.
