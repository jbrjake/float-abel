---
id: fl-5mywkqp
title: "Make the dbsp job's cache save: no dbsp- key exists in the store, so every run compiles the five-hundred-crate closure cold"
category: crate/hardening
labels: [reproducible-floats]
seq: 40
parent: fl-b9qtzty
docs:
  - docs/DESIGN.md#hardening-sp-hardening
verify: contains .github/workflows/ci.yml actions/cache/save
status: open
created: 2026-11-02T15:14Z
---
Owner, 2026-11-02: "ARE YOU NOT CACHING ARTIFACTS?" The repo's cache store on 2026-11-02 holds ten
entries, 683 MB: gate 409 MB, miri 54, fuzz 55, golden's seven legs 10 to 33 each. There is no
`dbsp-Linux-<lockhash>` entry at all, though ci.yml's dbsp job declares one (exact lockfile key, no
restore-keys). So the job that frees 22 GB of disk to build dbsp's closure builds it from nothing
every run: 15 to 17 minutes, 85 ci jobs in October.

First read the dbsp job's post-cache step in a recent log (run 41005514396, job 123899404793) for
`Cache saved` or the failure line. The likely cause is size: a single cache entry is capped at
10 GB and `target` after `cargo clippy --all-targets`, `cargo test` and the UDA example under the
dbsp feature can pass it. Then fix in this order of preference: cache `~/.cargo/registry` and
`target` separately so the registry at least hits; trim what is cached (`target/debug/deps` and
`target/debug/build` carry the closure, `target/debug/incremental` and the test binaries do not);
`CARGO_INCREMENTAL=0` in the job, which halves a debug target. The done line is a run whose dbsp
job logs `Cache restored from key: dbsp-` and finishes in minutes, written dated in this task.

## log
- 2026-11-02T15:14Z created

## comments
- 2026-11-02T15:15Z [claude (c90f1eae-8c6b-41ec-aa59-9d0990f41bc2)] 2026-11-02 evidence from PR #4's run 41005514396 (the lockfile is main's, hash 7087d73f…): the dbsp job logged `Cache not found for input keys: dbsp-Linux-7087d73f…` and no `Cache saved` line; the miri and gate jobs logged `Cache not found` for their keys AND their restore-key prefixes, then `Cache saved`. So on the 30th every job was cold, not only dbsp: the caches saved on 2026-10-20 (the last main push) had passed GitHub's seven-day idle eviction. miri and gate re-saved because they ended green; dbsp did not because its deny step failed and `actions/cache`'s post step saves only on a green job. The size hypothesis in the body is unconfirmed and secondary.
  
  The fix that follows: split the dbsp job's cache into `actions/cache/restore` before the compile steps and `actions/cache/save` with `if: always()` right after `cargo test`/`cargo run` and before the deny step, so a red advisory or a red test still banks the compiled closure for the next run; the same split is worth applying to the gate and miri jobs, whose test failures today also throw the build away. The seven-day eviction is GitHub's and stays: an idle main is cold on its next push whatever we do, which the fuzz marker task also lives with. Verify is now `contains .github/workflows/ci.yml actions/cache/save`.
