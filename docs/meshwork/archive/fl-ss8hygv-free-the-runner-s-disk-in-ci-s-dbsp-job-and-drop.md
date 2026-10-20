---
id: fl-ss8hygv
title: Free the runner's disk in CI's dbsp job and drop its stale-cache fallback
status: done
category: ci/dbsp
relates: [fl-v6f5d70, fl-jphvxrs]
verify: contains .github/workflows/ci.yml /df -h/
docs:
  - docs/DESIGN.md#hardening-sp-hardening
seq: 55
created: 2026-10-20T15:45Z
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
- 2026-10-20T17:24Z doing→done — verify exit 0 @ cd82359+1

## comments
- 2026-10-20T17:24Z [claude (30b8d01c-d5ec-456d-a066-16edcae1c161)] Green on the pushed commit: run 39463529492 on 4a83a00 (19cecd2 is the ci change), dbsp job 118768785008, 14m31s end to end against the 17m45s of the 0.364 cold run and the two deaths at 0.367. The disk step's two readings, on ubuntu-24.04 image 20261018.339, /dev/root 72G:
  
  - before: 61G used, 12G available (84%)
  - after: 39G used, 34G available (54%)
  
  so the four trees held 22G, removed in 65 s (the Android SDK is the slow one). The paths are the ones the image's own build scripts name: `install-android-sdk.sh` ANDROID_ROOT=/usr/local/lib/android, `install-dotnetcore-sdk.sh` --install-dir /usr/share/dotnet, `install-haskell.sh` GHCUP_INSTALL_BASE_PREFIX=/usr/local (so `/usr/local/.ghcup`, not the `/opt/ghc` the body guessed), `install-codeql-bundle.sh` under $AGENT_TOOLSDIRECTORY/CodeQL, reached in the job as `$RUNNER_TOOL_CACHE/CodeQL`. The Ubuntu 26.04 readme lists the same four (ANDROID_HOME at the same path), and the label's migration notice now says November 2026, so the step carries over.
  
  The cache: "Cache not found for input keys: dbsp-Linux-dc0215dc…" (no fallback, as intended), the closure built cold, clippy 2m02s, the dbsp_ tests 7m58s (22 passed, 346 s in the harness), the UDA example 5 s, ui_ 1m16s (2 passed), cargo-deny 32 s, and "Cache saved with key: dbsp-Linux-dc0215dc…" in 51 s. The next run with this lockfile restores that exact key. The 0.364 caches (dbsp-Linux-f7e7afa1…, 1.8 GiB each on main and the three dependabot refs) age out on their own; nothing restores them now.
  
  Left as it was: the miri job's and the gate job's `restore-keys`. The gate passed on its fallback every run, and the miri job has never saved a cache (no miri-* key exists), so neither has shown the failure; if either does, this step and the key change are the shape.
