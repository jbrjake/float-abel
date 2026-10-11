# bitrep conformance vectors

`vectors.json` is `conformance/vectors.json` from [bitrep](https://crates.io/crates/bitrep) 0.5.1
(repository: <https://github.com/KyleClouthier/bitrep>), copied unmodified from the published crate.
It is copyright (c) 2026 Kyle Clouthier and licensed under MIT OR Apache-2.0; the MIT text is
alongside as `LICENSE-MIT`. bitrep's `FORMAT.md` specifies the format the vectors exercise.

Each case's `state_hex` is bitrep's 281-byte state: 34 little-endian `u64` limbs in units of
2^-1074 (272 bytes), then one flag byte, then a `u64` count. (bitrep's FORMAT.md says 289 bytes;
its own `SumF64::BYTES` and every vector say 281.) `Wide` shares the limb layout, so
`tests/wide.rs` reproduces the first 272 bytes of every case and the `value_bits` of every read.
The flag byte and the count are bitrep's and are not compared: `Wide` counts specials where bitrep
flags them, and keeps no count.
