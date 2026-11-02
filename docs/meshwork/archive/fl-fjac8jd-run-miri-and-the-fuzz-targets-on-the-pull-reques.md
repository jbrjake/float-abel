---
id: fl-fjac8jd
title: "Run miri and the fuzz targets on the pull request only when no green run covers the code since it last changed: a probes output on ci.yml's changes job, decided by scripts/probes-covered.sh with a self-test in both gates"
status: done
category: crate/hardening
labels: [reproducible-floats]
relates: [fl-enkdnvk, fl-wpgpd4q]
verify: "all(contains .github/workflows/ci.yml /outputs.probes == 'true'/, exists scripts/probes-due.sh, contains CLAUDE.md /since the code last changed/)"
docs:
  - CLAUDE.md#standing-decisions
seq: 15
created: 2026-11-02T21:12Z
---

Owner, 2026-11-02, watching the first release pull request's suite: "i cannot stress ENOUGH just HOW MUCH I DO NOT WANT CUTTING A PR TO TAKE AN HOUR OF MY TIME just to re-run fuzz tests that are SUPPOSED to run every night! this is crazy!" and, on the rule: "FUZZ NEEDS TO RUN IF IT HASN'T YET SINCE THE CODE LAST CHANGED!"; earlier, "if it's just because it doesn't have a cache of it running in the past day, that's ok".

Today ci.yml's `changes` job says run=true on every pull_request event, and miri and the three fuzz targets hang off that alone, so the dev→main PR re-runs them whatever the nightly or a dispatch already proved. On #5 at 249fdad: eleven jobs done in two minutes, miri the long pole at 35 minutes, the fuzz targets ten each.

The rule: on the pull request, miri and the fuzz targets run only when no green run of them covers the code the PR carries. A green run is a completed ci.yml run on the head branch whose miri and fuzz jobs all concluded success (a nightly that skipped them does not count). The code is what the probes build: src/, tests/, fuzz/, Cargo.toml and Cargo.lock, with lines matching `^version = "` ignored so the release commit itself never trips it. The nightly and a dispatch keep their own gate. The decision is a script, scripts/probes-covered.sh, with a self-test in both gates like the release scripts, and the `changes` job exposes it as a second output the two jobs read. CLAUDE.md's ci.yml ruling and ci.yml's header say so.

Under this rule #5 still runs them today: Cargo.toml changed since the one green run (4bec2d5, the include globs), and no run has covered that.

## log
- 2026-11-02T21:12Z created
- 2026-11-02T21:15Z open→doing — claimed by claude (aa073407-90a0-459f-92ea-439fb58eb49c)
- 2026-11-02T21:21Z doing→done — verify exit 0 @ 9b7acc3+1

## comments
- 2026-11-02T21:21Z [claude (aa073407-90a0-459f-92ea-439fb58eb49c)] 2026-11-02, 9b7acc3 on local dev, held off origin until #5 merges on the owner's word ("at this point i don't want another release today ... don't update the release pr or make a new pr"): a push to dev would move #5's head and rerun its suite. scripts/probes-due.sh, 177 lines, a nine-case self-test in both gates, watched fail on two deliberate breaks of a scratch copy: with the version-line ignore dropped, cases 3 (the release commit) and 6 fail; taking the first run whether green or not, case 6 fails. Live against the real runs: at 249fdad it prints true and names Cargo.toml (the include line) as the change since the one green run, the dispatch at 4bec2d5; at 4bec2d5 it prints false. So #5 still runs the probes today, by the rule. The gate's first run on GitHub is the next pull request into main: read the changes job's probes step, which says why. A ci commit leaves the count at 0.30.2 (version-from-log.sh at 9b7acc3).
