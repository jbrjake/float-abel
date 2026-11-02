---
id: fl-48w2rt7
title: Complete the README features table's pulls-in column from Cargo.toml
status: done
category: crate/docs
verify: "all(contains README.md /timely 0\\.31/ , contains README.md /proptest. `std`/ , contains README.md /arbitrary 1. `std`/ , contains README.md /feldera-size-of. `std`/)"
docs:
  - README.md#features
created: 2026-10-23T21:49Z
---

Owner, in session: "add the other pulls in column entries or do the turns on / adds split." README § features omits what Cargo.toml [features] enables: `std` for `dbsp`, `laws` and `arbitrary`, and timely 0.31 for `differential`. Add them to the "pulls in" column. Leave README.md uncommitted, because the owner is mid-pass on it.

## log
- 2026-10-23T21:49Z created
- 2026-10-23T21:49Z open→doing — claimed by claude (101dc294-a5c4-4a2c-9db5-a90033e6f8c5)
- 2026-10-23T21:49Z doing→done — verify exit 0 @ 1b50150+4
