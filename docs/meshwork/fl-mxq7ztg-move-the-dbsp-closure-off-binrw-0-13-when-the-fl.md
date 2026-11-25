---
id: fl-mxq7ztg
title: "Move the dbsp closure off binrw 0.13 when the floor next moves: a future rustc rejects it (E0365, rust-lang/rust#127909)"
status: open
category: crate/hardening
labels: [reproducible-floats]
verify: lacks Cargo.lock /(?s)name = "binrw"\s+version = "0\.13/
docs:
  - CLAUDE.md#standing-decisions
seq: 220
created: 2026-11-25T00:45Z
---

Found 2026-11-25 in the push gate's output: 'warning: the following packages contain code that will be rejected by a future version of Rust: binrw v0.13.4', printed on every run of scripts/regression.sh's dbsp legs. `cargo +1.99.0 report future-incompatibilities` names the lint: E0365, 'extern crate `binrw` is private and cannot be re-exported' at binrw-0.13.4/src/lib.rs:336, 'previously accepted by the compiler but is being phased out; it will become a hard error in a future release', rust-lang/rust#127909. binrw 0.13.4 is dbsp 0.367.0's own direct dependency (`cargo tree --all-features -i binrw@0.13.4`), and binrw 0.14.0 to 0.15.2 exist. Nothing breaks today: the dbsp legs pin rustc 1.99.0. What to do: when the dbsp caret's floor next moves, check whether that dbsp is on a binrw past 0.13 and say so in the bump's commit; until then the note is the gate's one warning line. Not looked up: which rustc release makes it an error, and which dbsp release, if any, moved binrw. No issue goes upstream (CLAUDE.md § Boundaries).

## log
- 2026-11-25T00:45Z created
