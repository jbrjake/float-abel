---
id: fl-jmjbjk5
title: "Rule on deny.toml for the dbsp closure — eight licences, thirty duplicate majors and RUSTSEC-2026-0235 in rkyv 0.7 fail cargo-deny on all features"
status: open
category: crate/adapters
labels: [reproducible-floats]
relates: [fl-06sggdc, fl-yzy8bwc, fl-e4jjqf1, fl-ajqrs9p, fl-c3ahdwc, fl-ceyy1h6]
docs:
  - docs/DESIGN.md#dbsp-sp-dbsp
seq: 135
created: 2026-10-12T22:39Z
verify: contains docs/meshwork/fl-jmjbjk5-rule-on-deny-toml-for-the-dbsp-closure-eight-lic.md /2026-[0-9][0-9]-[0-9][0-9] owner ruled/
handoff: |
  State of the tree at afccf59 plus the store commit after it: the dbsp
  feature is shipped and
  closed ("Ship the dbsp feature", verify green), and this task is the
  owner's ruling, not a build. Nothing in
  it is for a session to write except, once the owner has ruled, the
  deny.toml edit the ruling
  names: (a) is eight `allow` entries, thirty `skip` rows each naming
  dbsp, and one advisory ignore
  by id; (b) is `[graph] all-features = false` plus `features = [...]`
  naming every feature but
  dbsp, a second deny-dbsp.toml for the dbsp CI job (`cargo deny --config
  deny-dbsp.toml check`
  as a step after its clippy), and a sentence in DESIGN.md § DBSP's
  constraints paragraph, which
  today records the counts. Either way the ruling is a dated "owner ruled"
  line in this task's
  file, which its verify looks for; then `cargo deny check` (sandbox off:
  the advisory fetch
  needs github.com) and `./scripts/regression.sh` must both be watched
  green before the task
  closes, and fl-fm78m8d's gate is this.
  
  Until then the push gate is red at its last leg, cargo-deny, and green
  on every leg before it
  (clippy on three feature sets, rustdoc, the full suite with ignored,
  release every_build,
  noalloc, golden, MSRV 1.74 on the 488-package lockfile, and the new dbsp
  leg on 1.99.0). Do
  not push; the owner pushes. Work that is unblocked meanwhile: fl-e4jjqf1
  Measure and fl-ajqrs9p
  Fuzz, both ready; fl-c3ahdwc the outward docs; fl-ceyy1h6 the trybuild
  pin, which now has a
  fourth snapshot to cover. fl-yzy8bwc, the dependency-line ruling,
  changes one line of
  Cargo.toml and one sentence of DESIGN.md if the owner picks the caret,
  and nothing otherwise.
  
  Facts a session needs and the tree does not say: the `1.99.0` toolchain
  has no clippy, so the
  feature's local lint is `cargo +stable clippy --features dbsp
  --all-targets -- -D warnings`
  while stable is 1.99.0; the probe crate's source is attached to the
  closed "Ship the dbsp feature" task; and
  `cargo +1.99.0 run --features dbsp --example dbsp_uda` is the example's
  run line.
---

Owner's call, surfaced from "Ship the dbsp feature" (fl-06sggdc). deny.toml runs with `[graph] all-features = true`, so cargo-deny sees the dbsp closure, 440 crates against the crate's 114, every extra one reached only through dbsp. On 2026-10-12 with cargo-deny 0.20.2 (`cargo deny check` reproduces each list; DESIGN.md § DBSP records the counts): `licenses FAILED` on 18 crates carrying eight licences the allow-list lacks (BSD-3-Clause, ISC, Zlib, MIT-0, CC0-1.0, BSL-1.0, CDLA-Permissive-2.0, bzip2-1.0.6); `bans FAILED` on 30 crates at two or more majors, hashbrown at five; `advisories FAILED` on RUSTSEC-2026-0235, a vulnerability in rkyv 0.7.46 (checked access of archives holding `Rc`/`Arc` to unsized data; fixed in 0.8.17, 0.7 unsupported upstream). rkyv 0.7 is dbsp 0.364's own storage format, so this crate's rkyv line must stay 0.7; `Exact`'s archive is six integers with no pointer, so the flaw is not reachable through it. The differential precedent is an ignore by id naming the dragging crate (owner: "Ignore both by id"); this one is a vulnerability, so it is asked, not assumed.

So the push gate and CI's deny job fail closed on the tree that ships the feature until deny.toml says what the dbsp closure may carry. Licences are owner-scoped, and eight licences plus thirty skips is a posture, not a mechanical edit.

Options:
(a) Extend deny.toml under its rules: the licences into `allow`, each duplicate into `skip` naming dbsp, the advisory ignored by id. The published posture then includes CC0-1.0, BSL-1.0, CDLA-Permissive-2.0 and bzip2 through an optional feature.
(b) Take dbsp out of the default graph (`all-features = false`, `features` naming every other feature) and give the dbsp CI job a second config, deny-dbsp.toml, with its own allow-list.

Recommendation: (b): the crate's posture is zero required dependencies, and a second config keeps the default closure as strict as it is while the dbsp closure's is stated once. Approving (a) or (b) is the ruling; the owner writes the line this task's verify looks for, dated. "Prepare the 0.1.0 release" needs this, because its CI must be green.

## log
- 2026-10-12T22:39Z created
- 2026-10-12T22:43Z handoff by claude (70076334-fceb-445b-b3a5-700374edbd88)
- 2026-10-12T22:44Z handoff by claude (70076334-fceb-445b-b3a5-700374edbd88)
