---
id: fl-ajqrs9p
title: "Fuzz and UB-check — cargo-fuzz targets for decode, sum-vs-oracle and sub-round-trip, and cargo miri test in CI"
category: crate/hardening
labels: [reproducible-floats, rf-tail]
docs: [docs/DESIGN.md#hardening-sp-hardening]
needs: [fl-a2wrx7w]
seq: 140
verify: "all(exists fuzz/fuzz_targets/decode_robustness.rs, exists fuzz/fuzz_targets/sum_vs_oracle.rs, exists fuzz/fuzz_targets/sub_roundtrip.rs)"
status: done
created: 2026-10-11T15:02Z
covers:
  - ref: docs/DESIGN.md#sp-hardening
    sha: a3b209f092634765b7ec3afcf91073d89ff4365d1a6e4dd5514ad25a89abde8f
relates: [fl-yzy8bwc, fl-cb4pxq1]
---
Per DESIGN.md § Hardening. The oracle is a `num-bigint` rational sum, as a dev-dependency only. A
nightly job runs 10 minutes per target, and anything it finds is committed as a regression test.

## log
- 2026-10-11T15:02Z created
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-hardening @071b3fa1d56a
- 2026-10-11T18:13Z cover docs/DESIGN.md#sp-hardening @a3b209f09263
- 2026-10-13T00:50Z handoff by claude (70076334-fceb-445b-b3a5-700374edbd88)
- 2026-10-13T13:22Z open→doing — claimed by claude (2dfe7fb0-aa27-4caa-abdc-1dd0125d6691)
- 2026-10-13T14:44Z doing→done — verify exit 0 @ 1b076e9+3

## comments
- 2026-10-13T14:44Z [claude (2dfe7fb0-aa27-4caa-abdc-1dd0125d6691)] Built as DESIGN.md § Hardening names it, with what the tree gave and what it lacked.
  
  The three targets live in fuzz/, a cargo-fuzz package excluded from the workspace, with a lib
  that is the oracle: every finite double as an exact count of 2^-1074 units, the multiset's sum a
  num-bigint integer beside three signed special counts, rounded to a double or a single by the
  IEEE rule in integer arithmetic that shares nothing with the crate's, and laid into Wide,
  Fixed256 and Binned words bit for bit. The oracle's own rounding is held to all 173 committed
  read vectors in both formats. decode_robustness feeds every decoder the bytes as they come and
  forced to each type's length and version byte, re-derives FORMAT.md's verdict on an Exact body,
  and holds postcard to from_bytes; sum_vs_oracle runs a multiset of inserts and retracts through
  every tier against the oracle, Binned at K = 1 and 3 against the truncated sum, and the bulk
  entries against the loop; sub_roundtrip is a + b − b == a on every tier, Exact over any two
  canonical words through the checked operators. Locally each ran ninety seconds under ASan with
  no finding; .github/workflows/fuzz.yml builds them on every push and runs each ten minutes
  nightly, with a crash kept as an artifact.
  
  The arbitrary feature had no impls: it was a dependency and nothing else. It now implements
  Arbitrary for Exact (any canonical word, mantissa width and exponent size drawn first so a pair
  reaches the operators roughly one time in eleven), Wide and Fixed (their limbs), with a test
  topic, tests/arbitrary.rs, that holds the draws canonical and the distribution to what the fuzzer
  needs.
  
  miri found two things before it could run. Sixty-seven test sites built powers of two with
  `powi`, which miri perturbs by a few ulps on purpose; they now use the bit-built pow2, in tests/
  and in the lib's unit tests. And the suite's sweeps (up to 10⁶ rounds) run some thousand times
  slower under the interpreter, so a `rounds(n)` helper in tests/support.rs and
  src/exact/testing.rs cuts a sweep to a hundredth under cfg(miri), with the assertions that scale
  rewritten against the draw count and the tests whose count or size is the assertion ignored
  under miri by name; trybuild and the timely topics are ignored too, the laws kit is out (one
  proptest case ran four minutes under the interpreter), and every SIMD dispatch arm carries
  not(miri) so the portable kernel runs. The ci.yml miri job runs the lib, the noalloc member and
  the suite.
