---
id: fl-vct9c5r
title: "Move actix-web and actix-http in Cargo.lock past RUSTSEC-2026-0446, 0447 and 0448: feldera-types' caret on 4.11 admits the fixed 4.18.2, so a cargo update clears the push gate's dbsp deny leg without an ignore"
status: open
category: crate/hardening
verify: lacks Cargo.lock /name = "actix-web"\nversion = "4\.16\.0"/
docs:
  - docs/DESIGN.md#hardening-sp-hardening
seq: 30
created: 2026-11-06T15:19Z
---

Found 2026-11-06 at 81b3dc1 by a hand run of scripts/regression.sh: every leg passed through the
dbsp tests, deny.toml's check passed, and deny-dbsp.toml's failed on three advisories cargo-deny
had not seen before, RUSTSEC-2026-0446, RUSTSEC-2026-0447 and RUSTSEC-2026-0448, against
actix-web 4.16.0 and its actix-http in the lockfile, reached through feldera-types 0.367.0 from
dbsp 0.367.0 (a compressed request body can drive a large allocation before the extractor
rejects it; this crate opens no HTTP connection).

feldera-types requires actix-web "4.11.0" on a caret, so the fixed 4.18.2 (actix-http 3.20.2) is
in range and no ignore is wanted: `cargo update -p actix-web -p actix-http`, then
`cargo deny --config deny-dbsp.toml --log-level warn check -D advisory-not-detected` green, and the
`cargo +1.99.0 test --features dbsp --test main dbsp_` leg once more on the moved lockfile. Until
then a `git push` fails closed at the gate's last leg (CLAUDE.md § Standing decisions: an ignore
by id is for an advisory dbsp itself pins with no in-range fix, which this is not).

## log
- 2026-11-06T15:19Z created
