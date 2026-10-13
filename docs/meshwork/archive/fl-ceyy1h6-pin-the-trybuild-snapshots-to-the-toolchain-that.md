---
id: fl-ceyy1h6
title: "Pin the trybuild snapshots to the toolchain that blessed them — tests/ui's .stderr files match rustc 1.99 and fail on 1.96.1, so the push gate is green only on one compiler"
status: done
category: crate/hardening
labels: [reproducible-floats]
relates: [fl-b41y9kt]
verify: contains tests/ui.rs /rustversion|rustc_version|skip/
docs:
  - CLAUDE.md#commands
seq: 165
created: 2026-10-12T21:09Z
---

Found 2026-10-12 running `./scripts/regression.sh` on jons-amd-mini (rustc 1.96.1): `cargo test -- --include-ignored` fails in the trybuild topic on tests/ui/fixed_to_bytes_wrong_length.rs, because the committed .stderr was blessed on rustc 1.99.0 and 1.96.1 words the const-assertion diagnostic differently. Every other test passes there. So the push gate is green only on the compiler that blessed the snapshots, and stable moving past 1.99 will break CI's gate job the same way.

Options: gate the ui topic on the rustc version the snapshots were blessed with (the `rustversion` crate, or `rustc --version` at test time) and skip loudly elsewhere; or pin the gate's toolchain in CI and say so in CLAUDE.md § Commands; or both. Whatever lands, the regression script on a machine with another stable must either pass or say why it skipped, never fail on wording.

## log
- 2026-10-12T21:09Z created
- 2026-10-13T14:52Z open→doing — claimed by claude (ace5f87f-8122-4167-a02a-0a2238009e67)
- 2026-10-13T15:09Z doing→done — verify exit 0 @ 9fc6dac+1

## comments
- 2026-10-12T22:43Z [claude (70076334-fceb-445b-b3a5-700374edbd88)] A fourth snapshot since 2026-10-12: tests/ui/binned_as_dbsp_weight.stderr, blessed on rustc 1.99.0 like the others. Its harness test, ui_dbsp_binned_is_no_dbsp_weight in tests/ui.rs, is under #[cfg(feature = "dbsp")], and that feature only builds on 1.96.1 and 1.99.0 anyway, so whatever pins the other three to their toolchain should name this one too, and a pin to 1.99.0 is already its reality.
- 2026-10-13T15:03Z [claude (ace5f87f-8122-4167-a02a-0a2238009e67)] Second axis found while fixing this: the .stderr files render core's panic.rs source inside the E0080 diagnostic, which rustc does only when rust-src is installed; without it the same release prints a bare '= note:' line. Local stable 1.99.0 has rust-src and the 1.99.0 toolchain does not, so four cases mismatch on 1.99.0 here, and CI's gate job (stable, --profile minimal, no rust-src) would mismatch too. So the blessed compiler is stable 1.99 with rust-src: the harness checks release, channel and the component through rustc -vV and --print sysroot, skips the compile-fail cases elsewhere with the reason on stderr, and ci.yml's 1.99.0 job installs rust-src and runs the ui_ topic.
