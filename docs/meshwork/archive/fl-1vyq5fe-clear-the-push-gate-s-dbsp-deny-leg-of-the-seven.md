---
id: fl-1vyq5fe
title: "Clear the push gate's dbsp deny leg of the seven advisories published since the last push: move actix-http and url in Cargo.lock to their fixed releases, and put cookie, chrono and humantime, which have no in-range fix, to the owner"
status: done
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
- 2026-11-23T16:32Z open→done — verify exit 0 @ 6ed5d19+2

## comments
- 2026-11-23T16:32Z [claude (088ad122-d06d-463d-8c3f-01d8f3ef0f7f)] 2026-11-23. Cleared by update at e15b604: actix-http 3.20.3 → 3.22.0 (RUSTSEC-2026-0552, -0553, -0554) and url 2.5.8 → 2.5.9 (-0592, with form_urlencoded 1.2.3); cargo also re-pointed seven packages' windows-sys dependency from 0.61.2 to 0.52.0, both versions in the lockfile before and after.
  
  No release the requirements admit: cookie (actix-web 4.20.0, the newest, still resolves cookie 0.16.2; fix in 0.18.3), chrono and humantime (unmaintained, no fixed release). Checked against dbsp 0.388.0, the newest, in a scratch project with this repo's deny-dbsp.toml: the same three versions and the same three advisories. The owner first answered "what does in range mean? your options make it seem like you didn't even CHECK for a newer dbsp before bothering me with this!"; asked again with that check and the term explained, 2026-11-23 owner ruled: "Ignore all three by id (Recommended)". The three ignores are in deny-dbsp.toml at the commit after e15b604, each naming its dragging crate; deny.toml is untouched. Both `cargo deny … check -D advisory-not-detected` runs print "advisories ok, bans ok, licenses ok, sources ok".
