---
id: fl-ezhk4cz
title: "Harmonize the workflows into three: ci.yml, the one suite, nightly on dev only when it moved and on the PR into main; release.yml, the release; publish.yml, the crates.io publish"
status: done
category: crate/release
labels: [reproducible-floats]
parent: fl-b9qtzty
relates: [fl-c8jpmpm, fl-fs9vhnd, fl-m3cymxv]
verify: "all(exists .github/workflows/ci.yml, exists .github/workflows/release.yml, exists .github/workflows/publish.yml, absent .github/workflows/fuzz.yml, absent .github/workflows/golden.yml, absent .github/workflows/cut-release.yml, contains .github/workflows/ci.yml schedule:, lacks .github/workflows/ci.yml /^\\s+push:/, contains .github/workflows/ci.yml /gh run list/, contains .github/workflows/release.yml /^\\s+push:/)"
docs:
  - docs/DESIGN.md#release-sp-release
seq: 10
created: 2026-11-02T18:24Z
---

Owner, 2026-11-02, in session: "we need a simple, best-practice ci/release process. i've given very
clear guidance and you keep giving me broken stuff. you just overcorrected and gave me a ci process
that has no testing until a PR is cut, after first going too granular and doing it on every push.
what we need is testing to occur nightly, ONLY when there have been code changes since the last run.
a middle path. then, yes, again on the PR. please review what we've got and streamline and
harmonize it. there are six different workflows there now and it's not clear what they all are or
when or how they run, and i am skeptical this isn't just careless mess. this is supposed to be a
project that demonstrates best practices in all respects."

What is there at bfa0173: five workflow files and dependabot.yml. ci.yml (gate, msrv, deny, dbsp,
miri; on push to main, every pull request, dispatch), golden.yml (seven targets; the same three
triggers), fuzz.yml (three targets; those three plus a nightly schedule that fuzzed main for ten
minutes a target every night of October whether or not it moved), cut-release.yml (the release PR
on dispatch, the tag and GitHub Release on a push to main) and release.yml (the crates.io publish on
dispatch). Three files share one trigger set and one purpose, testing the code; two carry names a
reader cannot tell apart; and nothing runs on dev between a push and the release PR.

Build: three workflows, each one purpose, each header saying when it runs.

- `ci.yml`, the one suite: gate, msrv, deny, dbsp, miri, golden's seven targets and fuzz's three,
  ten minutes each. Triggers: `schedule` nightly on dev (the default branch), `pull_request` into
  main, `workflow_dispatch`. No `push`. A first job, `changes`, decides the nightly: the last
  completed scheduled run on dev that ended success or failure names the commit it ran at (`gh run
  list`, GitHub's own tool), and a `git diff --quiet <last> <now> -- . ':!docs/meshwork'` that is
  quiet means no code changed and every other job is skipped; a pull request or a dispatch always
  runs. `permissions: contents: read` at the top, `actions: read` on the one job that lists runs.
- `release.yml`, the release, from cut-release.yml: unchanged in what it does, with the .crate
  packaged and attested before the tag and the GitHub Release are created, so nothing is published
  until the artifact exists.
- `publish.yml`, the crates.io publish, from release.yml: renamed, its header pointing at
  release.yml for the GitHub Release and naming itself for the Trusted Publishing form.

Docs in the same commits: CLAUDE.md's commands and standing decisions, DESIGN.md § Release and
§ Bit-identical across architectures and the Verification table, scripts/regression.sh's and
tests/golden/main.rs's comments, dependabot.yml's comment. The store's tasks that name fuzz.yml or
golden.yml in their verify (fl-c8jpmpm, fl-fs9vhnd) get their verify repointed at ci.yml.

Proof: the push gate green on each commit; `ruby -ryaml` parses every file;
scripts/check-actions-pinned.sh passes; the `changes` job's two commands run locally against two
real commits, one store-only (quiet) and one that reached a workflow (not quiet); one
`workflow_dispatch` of ci.yml on dev watched to a conclusion with `gh run view`, every job
present. The nightly's skip path is proven by the first two nights: the first runs, and the second,
if dev did not move outside the store, skips with the `changes` job's line in its log.

## log
- 2026-11-02T18:24Z created
- 2026-11-02T18:25Z open→doing — claimed by claude (227b40e5-20bd-44b0-9b79-d44c1a3c3587)
- 2026-11-02T19:15Z doing→done — verify exit 0 @ dca52bd+1

## comments
- 2026-11-02T18:36Z [claude (227b40e5-20bd-44b0-9b79-d44c1a3c3587)] 2026-11-02 built at 4bec2d5 on dev, pushed through the regression gate (regression: OK; bfa0173..4bec2d5). The push started no workflow of ours, as designed. Proven locally: ruby -ryaml parses ci.yml, release.yml and publish.yml; check-actions-pinned.sh passes; the changes job's diff is quiet across the store-only 84a4936..1887f18 and not across 1887f18..0e81d36; its gh run list query returns empty for ci.yml's schedule and f24b051 for the old fuzz.yml's, the sha the store recorded for October's idle main. GitHub now lists exactly ci, release and publish (plus Dependabot Updates); fuzz, golden and cut-release are gone from the registry. The proof run is the dispatch 41502146344 on dev at 4bec2d5, https://github.com/jbrjake/float-abel/actions/runs/41502146344; its conclusion goes in the next comment. The nightly's skip path cannot be dispatched (the changes job reads the event name) and is proven by the first two nights: 03:17 UTC 2026-11-03 runs, and 2026-11-04 skips if dev did not move outside docs/meshwork/.
- 2026-11-02T19:15Z [claude (227b40e5-20bd-44b0-9b79-d44c1a3c3587)] 2026-11-02 proof: dispatch 41502146344 on dev at 4bec2d5, conclusion=success on attempt 2. Attempt 1 ended red on one job, golden aarch64 macOS, cancelled after 15 min with GitHub's annotation 'The job was not acquired by Runner of type hosted even after multiple attempts' under a 'capacity constraints ... macOS arm64' notice; the other 16 jobs were green; `gh run rerun --failed` re-ran that job alone and it passed in 21 s. Per job: changes <1, gate 6, msrv <1, deny 1, dbsp 5, miri 34, golden Linux/Windows/macOS/aarch64 ≤2 each, wasm32 2, riscv64gc 1, s390x 2, fuzz 11 ×3; the commits job skipped (not a pull request). About 100 billed minutes with warm caches (Linux ×1, Windows ×2, macOS ×10). The changes job's log line: 'ci: a workflow_dispatch runs the suite'. The schedule path's first firing is 03:17 UTC 2026-11-03.
