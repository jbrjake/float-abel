---
id: fl-jdsha55
title: Drop the dead zstd@0.13 and zstd-safe@7 skips from deny-dbsp.toml
status: done
category: crate/hardening
labels: [reproducible-floats]
parent: fl-b9qtzty
verify: "all(lacks deny-dbsp.toml /zstd@0.13/, lacks deny-dbsp.toml /zstd-safe@7/)"
docs:
  - CLAUDE.md#standing-decisions
seq: 210
created: 2026-11-02T19:34Z
---

The push gate's cargo-deny leg over deny-dbsp.toml prints two `warning[unmatched-skip]` rows on
every run at b2f3ef7: `zstd@0.13` (line 96) and `zstd-safe@7` (line 98) match nothing, because
Cargo.lock holds zstd 0.12.4 and 0.14.3 and zstd-safe 6.0.6 and 8.2.0 only, actix-http having
left the 0.13 line when fl-r0s86hg updated it. The licenses comment on line 42 also still lists
`zstd-safe 7`. A warning does not fail the gate, so the rows are dead config that misdescribes the
closure. Drop the two skip rows and the stale name in the comment, keep `zstd@0.12`,
`zstd-safe@6` and the RUSTSEC-2026-0403 ignore, then run `cargo deny --config deny-dbsp.toml
--log-level warn check -D advisory-not-detected` with the sandbox off and read a clean `bans ok`
with no unmatched-skip warning; `cargo deny check` over deny.toml must stay green too.

## log
- 2026-11-02T19:34Z created
- 2026-11-25T00:11Z open→doing — claimed by claude (4aebe618-581f-48ef-9ee8-943721730ef9)
- 2026-11-25T00:12Z doing→done — verify exit 0 @ 1254159+2

## comments
- 2026-11-25T00:12Z [claude (4aebe618-581f-48ef-9ee8-943721730ef9)] 2026-11-24 at 1254159: the zstd@0.13 and zstd-safe@7 skip rows are gone and the licences comment names zstd-safe 8 alone. Before, this session's push at 37e3431 printed the two unmatched-skip warnings from the deny-dbsp.toml leg; after, the gate's two lines (`cargo deny --log-level warn check -D advisory-not-detected` and the same with `--config deny-dbsp.toml`, cargo-deny 0.20.2, sandbox off) each exit 0 on `advisories ok, bans ok, licenses ok, sources ok` with no warning line. zstd@0.12, zstd-safe@6 and the RUSTSEC-2026-0403 ignore stand.
