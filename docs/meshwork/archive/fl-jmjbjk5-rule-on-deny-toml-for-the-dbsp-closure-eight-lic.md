---
id: fl-jmjbjk5
title: "Rule on deny.toml for the dbsp closure — eight licences, thirty duplicate majors and RUSTSEC-2026-0235 in rkyv 0.7 fail cargo-deny on all features"
status: done
category: crate/adapters
labels: [reproducible-floats]
relates: [fl-06sggdc, fl-yzy8bwc, fl-e4jjqf1, fl-ajqrs9p, fl-c3ahdwc, fl-ceyy1h6]
docs:
  - docs/DESIGN.md#dbsp-sp-dbsp
seq: 135
created: 2026-10-12T22:39Z
verify: contains docs/meshwork/fl-jmjbjk5-rule-on-deny-toml-for-the-dbsp-closure-eight-lic.md /2026-[0-9][0-9]-[0-9][0-9] owner ruled/
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
- 2026-10-13T00:44Z open→doing — claimed by claude (70076334-fceb-445b-b3a5-700374edbd88)
- 2026-10-13T00:50Z doing→done — verify exit 0 @ 53b1b5d+1

## comments
- 2026-10-13T00:44Z [claude (70076334-fceb-445b-b3a5-700374edbd88)] 2026-10-12 owner ruled, in session: "yes follow your rec on dbsp's CI job", which is option (b): dbsp leaves deny.toml's graph, and a second config, deny-dbsp.toml, states the dbsp closure's licences, duplicate majors and the rkyv advisory by id, run by the dbsp CI job and the push gate's dbsp leg.
- 2026-10-13T00:50Z [claude (70076334-fceb-445b-b3a5-700374edbd88)] Built under the ruling, 2026-10-12: deny-dbsp.toml (all features; eight licences with their carrying crates; thirty-seven skip rows, the older major of each duplicate with what sits on each side, the `syn@2` row repeated from deny.toml; five advisory ignores by id), deny.toml's graph narrowed to every feature but dbsp, the deny CI job naming its config (the action's default `--all-features` would have overridden the graph), a deny-dbsp.toml step in the dbsp CI job, both configs in the push gate, and the sentence in DESIGN.md § DBSP and CLAUDE.md's standing decision. Watched: `cargo deny check` → advisories ok, bans ok, licenses ok, sources ok; `cargo deny --config deny-dbsp.toml check` → the same; `./scripts/regression.sh` → OK end to end, MSRV and dbsp legs included.
  
  Derived from the ruling, read back for the owner: the closure carries two advisories beyond the rkyv one this task presented, which the first gate run's tail had cut. RUSTSEC-2026-0258, a vulnerability in h2 0.3.27 (unbounded empty DATA frames), reached through actix-http 3.18 → actix-web → feldera-types from dbsp; this crate opens no HTTP connection, and the fix is in the h2 0.4 line hyper is already on, so nothing in dbsp's closure can take it. RUSTSEC-2024-0370, proc-macro-error 1.0.4 unmaintained, through utoipa-gen → utoipa → feldera-ir/feldera-types, a build-time macro. Both are ignored by id in deny-dbsp.toml with the dragging crates named, the same shape as the rkyv row; if the owner would rather not carry an ignore for a vulnerability they did not see, those two rows are the ones to strike and the dbsp CI job goes red by design.
