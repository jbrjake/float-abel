---
id: fl-x2k7fbb
title: "Verify the dispatched tag's signature in release.yml against the owner's committed key"
category: crate/release
labels: [reproducible-floats]
seq: 170
discovered-from: fl-k3g1511
needs: [fl-5qbt792]
docs:
  - docs/DESIGN.md#release-sp-release
verify: contains .github/workflows/release.yml verify-tag
status: open
created: 2026-10-20T15:04Z
---
DESIGN.md § Release: "CI builds it with `--locked` from a signed tag". The owner's checkout signs
nothing today: `gpg.format`, `user.signingkey`, `tag.gpgsign` and `commit.gpgsign` are unset and
HEAD is unsigned, and the repo holds no public key. The step cannot be built blind: which key,
and in which format, is the owner's.

The cheapest shape is SSH signing: `git config gpg.format ssh`, `user.signingkey` the public half
of a key the owner already holds, `tag.gpgsign true`; the public key committed as
`.github/allowed_signers` (`<email> namespaces="git" <key>`); and the workflow running
`git -c gpg.ssh.allowedSignersFile=.github/allowed_signers verify-tag "$TAG"` before anything
else. A GPG key works the same with the armoured public key imported in the job. The owner names
the key; the session builds the step and watches a dispatch of a signed tag pass it and an
unsigned one fail it.

## log
- 2026-10-20T15:04Z created
