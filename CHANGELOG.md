# Changelog

One section per release, newest first, each naming what a consumer sees: the public API, the wire
format, the feature set, the MSRV and the toolchains. Versions follow semantic versioning, and
`cargo-semver-checks` is armed from the first published release on.

## Unreleased

Nothing is published yet. The first release, 0.1.0, carries:

- `Exact`, `Wide` and `Fixed` as exact abelian groups over `f64` with a ring on top, and `Binned`
  as the batch-only tier that is not a group;
- the canonical wire format, `FORMAT.md`, on `to_bytes`/`from_bytes` and behind `serde`;
- the `differential` and `dbsp` adapters, `Exact` as a diff type for either engine;
- the `laws` kit, proptest checks of what an incremental engine asks of a weight;
- the `simd` feature's NEON and AVX2 bulk kernels, bit-identical to the scalar reference;
- MSRV 1.74 on the default features.
