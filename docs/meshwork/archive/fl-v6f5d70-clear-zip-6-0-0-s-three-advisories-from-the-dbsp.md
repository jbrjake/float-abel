---
id: fl-v6f5d70
title: "Clear zip 6.0.0's three advisories from the dbsp closure, which fail the push gate"
status: done
category: crate/adapters
relates: [fl-f8brdgt]
verify: contains Cargo.lock /(?s)name = "zip"\nversion = "9\./
docs:
  - CLAUDE.md#standing-decisions
  - docs/DESIGN.md#hardening-sp-hardening
seq: 50
created: 2026-10-20T15:05Z
---

The push gate's `cargo deny --config deny-dbsp.toml check` fails closed at ef90777 on three
vulnerabilities in `zip 6.0.0`, which `dbsp 0.364.0` drags in: RUSTSEC-2026-0367 (SUID/SGID bits
kept on extraction), -0368 (symlink targets outside the destination) and -0369 (permissions
applied through symlinks), dated 2026-10-09 and -10, all patched in `zip >= 9.0.0`. `cargo update
-p zip` cannot cross the major; `dbsp 0.367.0` (2026-10-18, `rust-version` 1.96.1 as before) is
on `zip ^9.0.1`, and a 0.x minor does not unify, so reaching it moves the feature's floor from
`"0.364"` to `"0.367"` (CLAUDE.md § Standing decisions: the caret, chosen over an exact pin, and
"moving the floor means re-running [the probe] there first", Cargo.toml). In CI the gate job skips
cargo-deny (not installed there), so the `dbsp` job is the one that goes red, and the `deny` job's
graph never holds zip.

Two routes, the owner's call because the posture is theirs ("security is absolutely critical
here"):
(a) Bump: `dbsp = "0.367"`, `cargo update -p dbsp`, re-run the probe (`tests/dbsp*.rs`, the
    `dbsp_uda` example, `deny-dbsp.toml`'s skip rows re-derived) on rustc 1.99.0, and confirm
    1.97 and 1.98 still ICE or no longer do. Clears the advisories at the source.
(b) Ignore the three ids in `deny-dbsp.toml`, each row naming zip 6.0.0 as dbsp 0.364.0's and
    that this crate's adapter never extracts an archive. Keeps the floor; carries a known
    vulnerability in the published feature's closure.

Recommendation: (a). The verify is the lockfile's zip at 9.

## log
- 2026-10-20T15:05Z created
- 2026-10-20T15:07Z open→doing — claimed by claude (4a8ded2c-6eb4-4638-a7f1-ee6da67e86ba)
- 2026-10-20T15:31Z doing→done — verify exit 0 @ 79cf25b+1

## comments
- 2026-10-20T15:07Z [claude (4a8ded2c-6eb4-4638-a7f1-ee6da67e86ba)] 2026-10-20 owner ruled, in session, choosing route (a): "Bump dbsp to 0.367". The floor moves from "0.364" to "0.367" with the caret kept; the probe re-runs at the new floor on rustc 1.99.0 and deny-dbsp.toml's rows are re-derived there.
- 2026-10-20T15:13Z [claude (4a8ded2c-6eb4-4638-a7f1-ee6da67e86ba)] 2026-10-20 owner, in session, on the bump's one licence cost: allowed MPL-2.0 in deny-dbsp.toml for colored 3.1.1, feldera-observability 0.367.0's; deny.toml's posture unchanged.
- 2026-10-20T15:31Z [claude (4a8ded2c-6eb4-4638-a7f1-ee6da67e86ba)] 2026-10-20 cleared at db4e5d9, pushed in 79cf25b: the push gate's two cargo-deny checks read 'advisories ok, bans ok, licenses ok, sources ok' on the way out (regression: OK), with dbsp at 0.367.0 and zip at 9.0.5 in Cargo.lock. deny-dbsp.toml's rows were re-derived from a no-skip bans run over the new closure; MPL-2.0 allowed for colored on the owner's word. CI run 39449087008 carries the dbsp job's own deny check.
