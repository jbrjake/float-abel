---
id: fl-z91b7pq
title: "Prove correct rounding — to_f64, to_f32 and div_f64 against CPython-generated vectors, and cross-tier agreement"
category: crate/proof
labels: [reproducible-floats, rf-critical-path]
docs: [docs/DESIGN.md#reads-and-rounding-sp-rounding]
needs: [fl-n7beqqc, fl-at6nh41, fl-atkbp6q]
seq: 60
verify: "all(exists tests/data/rounding_vectors.json, run cargo test package=float-abel rounding_)"
status: open
created: 2026-10-11T15:02Z
covers:
  - ref: docs/DESIGN.md#sp-rounding
    sha: d2071b1880ed28f2421e76a0f2c0b6fc50d0d2d1e18b9b1e9ab8859e623d103e
handoff: |
  The tree at a849bad is green through `./scripts/regression.sh` (exit 0:
  93 tests in the main
  package, 31 lib and 62 in `tests/main.rs`, plus the release leg,
  `tests/noalloc`, workspace clippy,
  rustdoc, the MSRV 1.74 build, cargo-deny). The gate's cargo-deny leg
  fetches the advisory database
  from github.com, which the Bash sandbox blocks, so run the gate with the
  sandbox off. Since the
  last handoff fl-te1748m closed: `Wide: From<Fixed<LSB, LIMBS>>` in
  `src/wide.rs` (sign-extend and
  shift through `limbs::shl_extended`, pinned by the `Embeds::<LSB,
  LIMBS>::OK` const assertion),
  and the three `TryFrom`s in `src/exact/convert.rs` over
  `Compact::from_magnitude`,
  `Compact::dyadic` and `Wide::try_deposit_dyadic`, the one window check
  `deposit_product` now
  shares. trybuild `=1.0.122` is a dev-dependency; its cases live in
  `tests/ui/` with a pass case
  beside the compile-fail ones, because the assertion is a
  post-monomorphisation error that
  `cargo check` never evaluates. The committed `.stderr` files are rustc
  1.99.0's wording; when
  stable's E0080 text moves, `TRYBUILD=overwrite cargo test convert_`
  rewrites them.
  
  This task is next, and none of it exists yet: no
  `scripts/gen_rounding_vectors.py`, no
  `tests/data/rounding_vectors.json`, no `to_f32` on any tier, and
  `round::Format` has only `F64`
  (`precision: 53, exp_bits: 11`). `round::round(negative, limbs, exp,
  sticky, fmt)` is already
  generic over the format and returns the bit pattern as a `u64`, so `F32`
  is
  `Format { precision: 24, exp_bits: 8 }` and each tier's `to_f32` is its
  `read` path with `F32` and
  `f32::from_bits(bits as u32)`: `Wide` and `Fixed` go through
  `limbs::abs_into` then `round`, and
  `Exact` through `Compact::magnitude()`, which is private to
  `src/exact/compact.rs`, so add
  `Compact::read_f32` beside `Compact::read`. `div_f64` is
  `round::div_window(mag, exp, n)`, which
  returns a three-limb window, its exponent and a sticky bit, then `round`
  with `F64`.
  
  The generator must not round through `f64` for the `f32` vectors:
  `struct.pack('<f', x)` takes a
  Python float, so it double-rounds. Round the `Fraction` to 24 bits in
  integer arithmetic (nearest,
  ties to even, with the `f32` subnormal and overflow thresholds) and emit
  the `u32` bits; `float(Fraction)`
  is correctly rounded for `f64`, and `Fraction(sum, n)` then `float()` is
  the `div_f64` oracle.
  Every vector is `(negative, mant, exp)` with the expected `f64` bits,
  `f32` bits and, for the
  division cases, `n` and the quotient's bits. The tests inject a vector
  exactly through the
  conversions: shift `mant` onto `Wide`'s grid, `Wide::from_parts(limbs,
  [0; 3])`, then
  `Exact::try_from(&wide)` for the ones that fit 127 bits and
  `Fixed256::from_limbs` for the ones on
  its grid, and assert each tier's `to_f64`, `to_f32` and `div_f64`
  against the vector bit for bit.
  Cover the ties at every tier boundary (halfway between adjacent doubles
  and floats, at
  `2^-1022` and `2^-126`, at `f64::MAX` and `f32::MAX` plus half an ulp,
  which rounds to ∞ by
  ties-to-even), subnormal results including `2^-1075` (a tie, to zero)
  and `2^-1075 + ε` (to
  `2^-1074`), and the `f32` cases where rounding via `f64` first lands on
  an `f32` tie and then goes
  the other way. python3 here is 3.12.13.
  
  Tests `rounding_*` in `tests/rounding.rs` (`mod rounding;` in
  `tests/main.rs`) read the JSON with
  serde_json, a dev-dependency already. The cross-tier check on 10⁶
  random finite multisets inside
  `Fixed256`'s window (`Rng::on_grid(-44, 95)` in `tests/support.rs`) must
  keep the smoke gate under
  60 s: run a slice of it unignored and the full million `#[ignore]`,
  which the push gate runs with
  `--include-ignored`. The verify is
  `all(exists tests/data/rounding_vectors.json, run cargo test
  package=float-abel rounding_)`.
  
  After it: fl-a2wrx7w (the wire format and serde; § Wire says each body
  is the `repr(C)` bytes, so
  `to_bytes` is `to_le_bytes` of the limbs then the counters, and
  `from_bytes` on `Wide` and `Fixed`
  can be total while `Exact`'s validates), fl-6shtzpj (the laws kit),
  fl-v7t6xhz (Binned). The
  portfolio ask fl-cfx001b is still open. `CARGO_TARGET_DIR` is set
  outside ~/Documents by the
  repo's `.claude/settings.local.json`.
relates: [fl-a2wrx7w, fl-6shtzpj, fl-v7t6xhz, fl-cfx001b]
---
Per DESIGN.md § Reads and rounding. `scripts/gen_rounding_vectors.py` (committed; it uses
`fractions.Fraction`, `float()`, and `struct` for `f32`) writes `tests/data/rounding_vectors.json`.
The vectors cover ties at every tier boundary, subnormal results, overflow to ±∞, and the `to_f32`
cases where rounding via `f64` first would differ.

Tests `rounding_*` read the JSON. They also check that `Fixed256`, `Wide` and `Exact` agree on 10⁶
random finite multisets inside `Fixed256`'s window.

## log
- 2026-10-11T15:02Z created
- 2026-10-11T15:03Z cover docs/DESIGN.md#sp-rounding @d2071b1880ed
- 2026-10-11T19:01Z handoff by claude (9595457b-430f-4785-af6d-c23dc1a77cde)
