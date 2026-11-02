---
id: fl-cqmcs7c
title: "Run ci's miri, dbsp and golden jobs only when the diff reaches Rust, the lockfile or a workflow"
category: crate/hardening
labels: [reproducible-floats]
seq: 30
parent: fl-b9qtzty
docs:
  - docs/DESIGN.md#hardening-sp-hardening
verify: "all(contains .github/workflows/ci.yml needs.changes.outputs.rust, contains .github/workflows/golden.yml needs.changes.outputs.rust)"
status: open
created: 2026-11-02T15:14Z
---
Owner, 2026-11-02: "WHY IS CI TAKING SO LONG". The miri job is the long pole at 41 to 57 minutes
of interpretation (not compilation: a cache cannot shorten it), the dbsp job 15 to 17 minutes, and
golden's seven targets run on every event. Dependabot's three actions bumps each ran the whole set
on a checkout hash change, 33 to 57 minutes of miri apiece, and a README or BENCH.md commit does the
same. smoke.sh already classifies a diff ("staged diff reaches no Rust; cargo legs skipped") and the
workflows do not.

Build: a first `changes` job in ci.yml and golden.yml that runs `git diff --name-only` over the
push range (`github.event.before..github.sha`) or the PR's merge base and sets outputs `rust`
(any of `**/*.rs`, `Cargo.toml`, `Cargo.lock`, `tests/data/**`, `scripts/**`, `.cargo/**`,
`deny*.toml`) and `workflows` (`.github/**`). miri, dbsp and golden's matrices carry `if:
needs.changes.outputs.rust == 'true'`; the gate job (smoke + regression, nine minutes) and the
pinning check run on `workflows` too, so a dependabot PR still proves its hash; msrv and deny are
cheap and run always. A `workflow_dispatch` runs everything. The push gate's definition does not
move: regression.sh is unchanged, only when CI spends an hour on it.

The fail-closed shape matters: a skipped required check must not read as green to a ruleset, so
the `changes` job itself is what the ruleset requires, and the heavy jobs stay out of the required
list until the flip's ruleset exists (fl-4fcbtkx's checklist, item 5).

## log
- 2026-11-02T15:14Z created
