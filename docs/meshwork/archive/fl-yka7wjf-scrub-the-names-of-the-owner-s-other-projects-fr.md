---
id: fl-yka7wjf
title: "Scrub the names of the owner's other projects from the tree and its history before the repo goes public — the task files, the archive and two attachments in a session, the history on the owner's ruling"
status: dropped
category: crate/release
relates: [fl-4fcbtkx]
verify: contains docs/meshwork/fl-yka7wjf-scrub-the-names-of-the-owner-s-other-projects-fr.md /2026-[0-9][0-9]-[0-9][0-9] owner read the scrubbed tree/
docs:
  - CLAUDE.md#boundaries
  - docs/DESIGN.md#release-sp-release
seq: 55
created: 2026-10-20T14:30Z
---

Owner, 2026-10-20, offered the choice before step 2 of fl-4fcbtkx makes the repo readable:
"Scrub before visibility". It carries out the owner's 2026-10-11 word: "this needs to be a
public project that isn't tied to my internal dev stuff". CLAUDE.md § Boundaries already holds
the code to that, and the code is clean. The task store and the history are not.

**What goes.** The names of the owner's other repositories (the projects that consume this crate
and the repository that planned it), the planning document's name, section numbers and draft
handles, cross-store task ids (`<repo>#<id>`) and the edges that carry them (`answers:`,
`relates:`, `needs:`), CI credential names that belong to other repositories, and absolute local
paths. Public upstreams and prior art stay: differential-dataflow, timely, dbsp, bitrep.

**Where it is, at 328d420.** Counted with `git grep -i -E` over the name of every directory beside
this one under the code root (`ls ..`), `target/` excluded. Two of those names hit only as
substrings ("concatenated", `ExactResolver`) and are not counted.
- `src/`, `tests/`, `benches/`, `scripts/`, CI, CLAUDE.md and the docs outside the store: clean.
- The live task files: 28 lines over 7 files. fl-4fcbtkx has 15 (its title, its `answers:` edge
  and its blocked-reason among them), fl-qv0vh39 5, fl-k7gec8v 4, and fl-fm78m8d, fl-jjv29pf,
  fl-k3g1511 and fl-mj3tcta 1 each.
- The archive: 69 lines over 58 files. fl-4fcbtkx's 2026-10-20 comment counts 30 lines over 9
  files. That count leaves the archive out.
- Two attachments, `docs/meshwork/attachments/fl-nhcdk92/probe-Cargo.toml` and
  `docs/meshwork/attachments/fl-zxpq1wz/probe-Cargo.toml`, path-depend on this crate by an
  absolute path under the owner's home directory.
- The history: 6 of 173 commit messages, the blobs of every store edit, and 2 lines once added
  outside the store and since removed.

**The two halves.**
1. The tree, in a session: rewrite those lines so they say what this repo needs without the
   outside names. A consumer becomes "a downstream consumer", and a pin becomes "a downstream
   pin". fl-k7gec8v, the ask to amend the shared plan, leaves this store. It is dropped here with
   a comment, and the plan's owner refiles it where the plan lives. fl-4fcbtkx's `answers:` edge
   points outside the store and goes too, with its title and body rewritten to match. The
   attachments take a relative path or a placeholder.
2. The history, on the owner's ruling: a tree scrub alone leaves every removed line readable in
   `git log -p` once the repo is public. There are two ways out. One is a rewrite of blobs and
   messages (`git filter-repo`) and a force-push over origin's b14d830. The other is one fresh
   root commit at publication. Either way, every commit hash changes, and the pins downstream
   consumers hold by hash move with it. Which way, or neither, is the owner's call.

Out of this task's scope until the owner says otherwise: the task tracker's own footprint (the
store under `docs/meshwork/`, its shim, `.meshwork-version`, the plugin line in
`.claude/settings.json`, and the store commands in CLAUDE.md and `scripts/smoke.sh`).

**Done:** the tree half committed, the history half done as ruled, and a line in this task's
comments, in the owner's hand: "<date> owner read the scrubbed tree".

## log
- 2026-10-20T14:30Z created
- 2026-10-20T14:41Z open→dropped — owner, 2026-10-20: "drop the scrub task, we don't need it" — the disclosure gate passes the tree and all 173 commit messages, and a content read against the protected concepts found names, pins and published technique only
