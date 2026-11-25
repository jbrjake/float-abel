---
id: fl-v9rc7ds
title: "Bring deny-dbsp.toml's reasons up to the lockfile: seven rows name actix-http 3.20 where it is 3.22.0, and the header counts four dbsp ignores where seven stand"
status: done
category: crate/hardening
labels: [reproducible-floats]
parent: fl-b9qtzty
discovered-from: fl-jdsha55
verify: "all(lacks deny-dbsp.toml /actix-http 3\\.20/, lacks deny-dbsp.toml /the four dbsp/)"
docs:
  - CLAUDE.md#standing-decisions
created: 2026-11-25T00:13Z
---

Found 2026-11-24 while closing fl-jdsha55. e15b604 moved actix-http to 3.22.0 and 6ed5d19 added three ignores (cookie, chrono, humantime), and the config's prose stayed behind: six skip reasons (derive_more@1, h2@0.3, http@0.2, rand@0.8, rand@0.9, zstd@0.12) name actix-http 3.20.3, the RUSTSEC-2026-0258 reason names actix-http 3.20, and the header says 'the four dbsp's closure carries' over seven rows. Checked against Cargo.lock with cargo +1.99.0 tree --all-features -i <crate>@<version> --depth 1 for each duplicated crate: every other dependent and version the six reasons name is what the lockfile holds, and actix-http 3.22.0 stands where 3.20.3 did (derive_more 2.1.1, h2 0.3.27, http 0.2.12, rand 0.10.3, zstd 0.14.3). So the change is the version string, the count, and a header sentence for the three new rows; both cargo-deny lines of the push gate must stay at exit 0 with no warning.

## log
- 2026-11-25T00:13Z created
- 2026-11-25T00:13Z open→doing — claimed by claude (4aebe618-581f-48ef-9ee8-943721730ef9)
- 2026-11-25T00:13Z doing→done — verify exit 0 @ b61fa6f+4

## comments
- 2026-11-25T00:13Z [claude (4aebe618-581f-48ef-9ee8-943721730ef9)] 2026-11-24 at b61fa6f: the six skip reasons name actix-http 3.22.0, the RUSTSEC-2026-0258 reason names actix-http 3.22, the header counts seven and says in one sentence what cookie, chrono and humantime are. Checked before the edit with `cargo +1.99.0 tree --offline --all-features -i <crate>@<version> --depth 1` on derive_more 1.0.0 and 2.1.1, h2 0.3.27 and 0.4.20, http 0.2.12 and 1.5.0, rand 0.8.8, 0.9.5 and 0.10.3, zstd 0.12.4 and 0.14.3: each row's dependents are the lockfile's. After it, the gate's two cargo-deny lines (0.20.2, sandbox off) exit 0 with no warning. The other skip rows' reasons were not re-derived.
