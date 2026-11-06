---
id: fl-vct9c5r
title: "Move actix-web and actix-http in Cargo.lock past RUSTSEC-2026-0446, 0447 and 0448: feldera-types' caret on 4.11 admits the fixed 4.18.2, so a cargo update clears the push gate's dbsp deny leg without an ignore"
status: done
category: crate/hardening
verify: lacks Cargo.lock /name = "actix-web"\nversion = "4\.16\.0"/
docs:
  - docs/DESIGN.md#hardening-sp-hardening
seq: 25
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
- 2026-11-06T17:15Z open→doing — claimed by claude (9113b2fa-3013-4b52-b301-b794a9f4a5fa)
- 2026-11-06T17:22Z doing→done — verify exit 0 @ e26f1f1+2

## comments
- 2026-11-06T17:22Z [claude (9113b2fa-3013-4b52-b301-b794a9f4a5fa)] 2026-11-06, e26f1f1: the registry resolved actix-http to 3.20.3, past the 3.20.2 named here, and actix-web to 4.18.2 with actix-web-codegen 4.5.0; base64 0.22 left the graph, so its skip row in deny-dbsp.toml went with it, and the resolver unified seven crates (colored, errno, mio, rustix, rustls-platform-verifier, tempfile, winapi-util) onto windows-sys 0.61.2, memory-stats alone keeping 0.52. deny-dbsp.toml's check: advisories ok, bans ok, licenses ok, sources ok, exit 0, with only fl-jdsha55's two zstd unmatched-skip warnings left; the dbsp_ leg passed 22 and the dbsp_uda example built on 1.99.0.
