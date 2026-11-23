---
id: fl-1vyq5fe
title: "Clear the push gate's dbsp deny leg of the seven advisories published since the last push: move actix-http and url in Cargo.lock to their fixed releases, and put cookie, chrono and humantime, which have no in-range fix, to the owner"
status: open
category: crate/hardening
verify: "all(lacks Cargo.lock /name = \"url\"\\nversion = \"2\\.5\\.8\"/, lacks Cargo.lock /name = \"actix-http\"\\nversion = \"3\\.20\\.3\"/)"
docs:
  - docs/DESIGN.md#hardening-sp-hardening
seq: -25
created: 2026-11-23T16:27Z
---

Found 2026-11-23 by the pre-push hook on dev at 1f72978: every leg of scripts/regression.sh passed
through the dbsp tests and deny.toml's check, and deny-dbsp.toml's failed on seven advisories
published since the last push, all in dbsp 0.367.0's closure, so the push was refused:

- RUSTSEC-2026-0552, 0553, 0554: actix-http 3.20.3 (through actix-web 4.18.2, feldera-types);
  fixed in 3.21.1, 3.21.3, 3.21.2.
- RUSTSEC-2026-0592: url 2.5.8 (actix-web, object_store, reqwest, tower-http); fixed in 2.5.9.
- RUSTSEC-2026-0594: cookie 0.16.2 (actix-web 4.18.2); fixed in 0.18.3.
- RUSTSEC-2026-0543: chrono 0.4.45 unmaintained (feldera-observability, feldera-types,
  object_store); no safe upgrade.
- RUSTSEC-2026-0542: humantime 2.4.0 unmaintained (object_store 0.14.2, feldera-storage); no safe
  upgrade.

Move every crate whose fix the closure's carets admit with `cargo update -p`, as fl-vct9c5r did.
An advisory with no in-range fix is an ignore by id in deny-dbsp.toml with the dragging crate
named, and each such ignore is the owner's to rule: asked in session, never added on precedent.

## log
- 2026-11-23T16:27Z created
