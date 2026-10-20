---
id: fl-ss8hygv
title: Free the runner's disk in CI's dbsp job and drop its stale-cache fallback
status: doing
category: ci/dbsp
relates: [fl-v6f5d70, fl-jphvxrs]
verify: contains .github/workflows/ci.yml /df -h/
docs:
  - docs/DESIGN.md#hardening-sp-hardening
seq: 55
created: 2026-10-20T15:45Z
claimed-by: claude (30b8d01c-d5ec-456d-a066-16edcae1c161)
---

The `ci` workflow's dbsp job failed at 79cf25b (run 39449087008, job 118718750574, 7m24s): clippy
over the dbsp feature passed and the `cargo test --locked --features dbsp --test main dbsp_` step
ended with no conclusion, the runner itself reporting `System.IO.IOException: No space left on
device : '/home/runner/actions-runner/.../_diag/Worker_….log'`. The same job passed in 17m45s on
2026-10-16 at dbsp 0.364 with no cache to restore. Since db4e5d9 the closure is 513 crates (zip 9,
zstd 0.14, tracing-subscriber, sysinfo and the rest of dbsp 0.367's additions), and the job's
`actions/cache` step, keyed on the lockfile's hash, missed and fell back through
`restore-keys: dbsp-${{ runner.os }}-` to the 0.364 target directory, so the runner's disk held
the old closure's debug artifacts and built the new ones beside them. The miri job restores the
same way (its own key); the gate job passed on the same push with its own fallback.

Fix, in the dbsp job: free the runner's disk before the toolchain step (the usual removal of the
preinstalled .NET, Android and Haskell trees under `/usr/share/dotnet`, `/usr/local/lib/android`
and `/opt/ghc`, with `df -h` printed before and after so the headroom is a number in the log), and
drop the job's `restore-keys` so a lockfile change builds from a clean target rather than on top
of the previous closure's. Then watch the job finish green on the pushed commit and record the
run id and the two `df` readings in the close comment. If the job still runs out, the next cut is
splitting clippy and the tests into two jobs so no single runner holds both artifact sets.

## log
- 2026-10-20T15:45Z created
- 2026-10-20T17:01Z open→doing — claimed by claude (30b8d01c-d5ec-456d-a066-16edcae1c161)
