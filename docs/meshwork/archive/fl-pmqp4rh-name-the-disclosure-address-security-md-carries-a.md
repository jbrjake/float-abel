---
id: fl-pmqp4rh
title: "Name the disclosure address SECURITY.md carries — a private email, GitHub's private vulnerability reporting on the repository, or both"
status: done
category: rulings
labels: [reproducible-floats]
discovered-from: fl-c3ahdwc
docs:
  - docs/DESIGN.md#documentation-sp-docs
seq: 159
created: 2026-10-13T15:11Z
verify: contains docs/meshwork/fl-pmqp4rh-name-the-disclosure-address-security-md-carries-a.md /2026-[0-9][0-9]-[0-9][0-9] owner ruled/
---

Owner's call, surfaced from "Write the outward docs" (fl-c3ahdwc). DESIGN.md § Documentation says `SECURITY.md` names the disclosure address, and no address is in the tree: the crate's `Cargo.toml` carries no authors field (CLAUDE.md § Hygiene), and the git author email is not a published contact. Everything else in that task can be written without it; the file itself waits here.

Options:
(a) A private email address the owner names, in the file as the only channel, with a stated response window (seven days is the common one).
(b) GitHub private vulnerability reporting on the repository (Security → Advisories → "Report a vulnerability"), which needs the feature switched on in the repository settings, with SECURITY.md pointing at it and no email at all.
(c) Both: the advisory form first, the address as the fallback.

Recommendation: (c), because the advisory form keeps the report off email and the address covers a reporter without a GitHub account; the owner names the address and switches the form on, neither of which a session can do.

The owner writes the line this task's verify looks for, dated, with the address or the choice.

## log
- 2026-10-13T15:11Z created
- 2026-10-13T15:14Z open→done — verify exit 0 @ 656c340+5

## comments
- 2026-10-13T15:14Z [claude (ace5f87f-8122-4167-a02a-0a2238009e67)] 2026-10-13 owner ruled, in session: "GitHub reporting only", which is option (b): SECURITY.md points at GitHub's private vulnerability reporting on the repository and carries no email; switching the form on in the repository settings is the owner's act.
