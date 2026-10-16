---
id: fl-qv0vh39
title: "Ship the AVX-512 kernel — eight doubles a step in src/simd/avx512.rs, dispatched above AVX2, landed only once a machine with AVX-512 runs the simd_ tests and the golden hash green"
status: open
category: crate/simd
labels: [reproducible-floats, rf-tail]
relates: [fl-b41y9kt]
verify: exists src/simd/avx512.rs
docs:
  - docs/DESIGN.md#speed-sp-simd
seq: 200
created: 2026-10-12T21:09Z
handoff: |
  Next session: nothing in `ready` is buildable on this machine. This task
  lands only on a machine
  with an avx512 flag, and none of the three has one (the body names
  them); fl-k3g1511, the publish
  ruling, is blocked on the owner's words of 2026-10-16, "i don't want to
  publish yet", with its
  three gating tasks done; fl-fm78m8d, the 0.1.0 release, waits behind
  that ruling; fl-4fcbtkx (push
  main to origin) and fl-mj3tcta (the SIMD product kernel) stay blocked on
  the owner's word. What the
  owner alone can move: the publish ruling and the release behind it; the
  push, which leras's ask
  le-yxhtwgw waits on; a machine with AVX-512 for this task; the ask out
  to portfolio, fl-k7gec8v
  (amend PLAN-reproducible-floats.md for the heap Exact), which the
  portfolio store answers; and
  the spec audit's 35 re-open candidates on docs/DESIGN.md, pins on done
  tasks whose clauses moved
  later, which nobody has ruled on repinning.
  
  What stands at 18e42b3: the laws kit runs at the limb cap's edge,
  tests/laws_cap.rs.
  `exact_at_the_cap()` draws at the closure check's own budget, a weight
  under 2^(CAP_BITS − 5)
  because a case folds WEIGHTS = 32 of them, with the multipliers inside
  those five bits; three wide
  shapes and three small, one draw in sixteen small and one in sixteen
  negative, so a case's fold
  climbs onto the cap's last bit and retracts across it. Closure holds by
  an integer and by an
  `Exact`, the group laws hold, and the `UnderCap` control (refuses a
  result spanning exactly
  CAP_BITS) fails there and holds on `exact_full()`, which proves the
  reach; 32 cases a sweep, 8.5 s
  wall for the five tests in the dev profile. Watched fail on `big::add`'s
  pre-fix refusal, 328 cap
  panics; `big::mul`'s pre-fix pre-check is unreachable by the check's
  independent draws (the bit
  counts of a weight and a multiplier at the budget sum to at most
  CAP_BITS) and stays with
  tests/exact_cap.rs's oracle sweep. One optional follow-up, unfiled:
  `check_ring` at the edge with
  a 4-bit multiplier, since `(a + b)·k` is one bit wider. Verified this
  session: the commit gate
  green on 18e42b3 with 236 tests, clippy `-D warnings` and rustdoc clean,
  and the push gate green
  end to end on 18e42b3 (MSRV 1.74 default features, the dbsp leg on
  1.99.0, both deny closures).
  main is unpushed by the owner's word.
  
  If a machine with AVX-512 turns up: src/simd/avx2.rs is the template op
  for op, the dispatch is
  src/simd/mod.rs's `fold_tile`, `span` and `top_limb`, and the AVX-512
  intrinsics need Rust 1.89, so
  the `simd` feature takes a stated floor or a version gate while the
  default features keep 1.74.
---

Per DESIGN.md § Speed: "AVX-512 where present". `src/simd/avx512.rs`, eight doubles a step, dispatched above AVX2 in `src/simd/mod.rs`'s `fold_tile`, `span` and `top_limb` behind `is_x86_feature_detected!("avx512f")`, the same algorithm as scalar.rs op for op. AVX-512 makes two of the AVX2 workarounds one instruction: `vpmaxuq`/`vpminuq` for the span pass and masked compares for the suspect test; the variable shifts keep AVX2's any-count-of-64-or-more-is-zero rule, so no byte chain.

Lands only once a machine with AVX-512 has run `cargo test -p float-abel simd_` and `cargo test -p float-abel-golden` green, because "done means you watched it work": this machine is aarch64, jons-amd-mini is a Ryzen 7 5700U and turing is an i7-5930K, and none has an avx512 flag. The x86 CI runner's CPU is not controllable.

MSRV: the AVX-512 intrinsics stabilised in Rust 1.89, above the crate's 1.74. The module needs either a version gate or a stated floor for the `simd` feature in Cargo.toml and CI; the default features keep 1.74 either way, because `simd` is not one.

## log
- 2026-10-12T21:09Z created
- 2026-10-16T19:34Z handoff by claude (03cbad3b-1e00-455d-9d44-3b0d9decee6e)
