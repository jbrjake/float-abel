---
id: fl-8f337yq
title: Clear RUSTSEC-2026-0382 and -0383 from the dbsp closure by updating rustls-webpki and webpki-root-certs
status: doing
category: ci/dbsp
relates: [fl-v6f5d70, fl-ss8hygv]
verify: contains Cargo.lock /(?s)name = "webpki-root-certs"\s+version = "1\.0\.10"/
docs:
  - docs/DESIGN.md#hardening-sp-hardening
seq: 50
created: 2026-10-20T19:27Z
claimed-by: claude (30b8d01c-d5ec-456d-a066-16edcae1c161)
---

The ci dbsp job's cargo-deny step over deny-dbsp.toml went red at e43a638 (run 39482535508, job 118834500832, 19:25Z) after passing at 17:22Z and 18:12Z on the same config, and the local gate had passed at 19:15Z: two advisories published in between, both on transitive crates of the dbsp closure with fixed releases, RUSTSEC-2026-0382 (rustls-webpki 0.103.15, "Certificate verification may accept distrusted intermediates", fixed in 0.103.16) and RUSTSEC-2026-0383 (webpki-root-certs 1.0.9, "Bundled root certificates allow trust in distrusted emSign intermediates", fixed in 1.0.10). `cargo update -p rustls-webpki -p webpki-root-certs` moves exactly those two; the dbsp and miri cache keys follow the lockfile, so the next dbsp job builds cold. Then `cargo deny --config deny-dbsp.toml --all-features check advisories` must say ok, and the pushed commit's dbsp job must go green; record the run id.

## log
- 2026-10-20T19:27Z created
- 2026-10-20T19:27Z open→doing — claimed by claude (30b8d01c-d5ec-456d-a066-16edcae1c161)
