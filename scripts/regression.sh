#!/usr/bin/env bash
# The push gate (.githooks/pre-push): deterministic correctness only. Strict lint over every feature
# configuration the default toolchain can build, rustdoc with warnings denied, every test including
# the ignored ones, the MSRV build on exactly the default features, and cargo-deny. Benchmarks never
# run here.
#
# `-D warnings` reaches clippy and rustdoc as their own flags, never as RUSTFLAGS: RUSTFLAGS is part
# of every unit's fingerprint, so exporting it would keep a second copy of the graph under target/.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

MSRV=1.74.0

./scripts/check-file-length.sh
./scripts/check-test-topics.sh
./scripts/check-actions-pinned.sh
./scripts/check-commit-message.sh --self-test

if ! command -v cargo >/dev/null 2>&1; then
  echo "regression: FAILED (no cargo on PATH)" >&2
  exit 1
fi

# The release scripts' fixtures: the count, the entry, the pull request that cuts the release and
# the gate on its probes.
./scripts/version-from-log.sh --self-test
./scripts/changelog-entry.sh --self-test
./scripts/release-pr.sh --self-test
./scripts/probes-due.sh --self-test

cargo fmt --all -- --check
cargo fmt --manifest-path fuzz/Cargo.toml -- --check

# The lib alone, on each feature set a consumer can pick without the test features.
cargo clippy --quiet --lib --no-default-features -- -D warnings
cargo clippy --quiet --lib -- -D warnings
# Every target of every member: the self dev-dependency turns the test features on here.
cargo clippy --quiet --workspace --all-targets -- -D warnings
# The fuzz package stands outside the workspace (fuzz/Cargo.toml), so the lint above never sees
# it. Stable builds everything in it but the sanitizer, so its targets are linted and its oracle's
# tests run here; the targets themselves run in ci.yml's fuzz jobs, ten minutes each. `--locked`,
# so a lockfile a release left stale fails loudly instead of being rewritten under the gate.
cargo clippy --quiet --locked --manifest-path fuzz/Cargo.toml --all-targets -- -D warnings
cargo test --quiet --locked --manifest-path fuzz/Cargo.toml >/dev/null
# The short table's package stands outside the workspace too (examples/short-table/Cargo.toml):
# its lint and tests, then the program run `--locked` and diffed against
# examples/short-table/OUTPUT.md, so the committed table is always what the code prints. CI's gate
# job runs this on x86_64 Linux against the copy blessed on aarch64 macOS.
./scripts/check-short-table.sh --lint

RUSTDOCFLAGS="-D warnings" cargo doc --quiet --no-deps

cargo test --quiet -- --include-ignored >/dev/null
# tests/ui's .stderr files match one compiler, stable 1.99 with rust-src (BLESSED_RUSTC in
# tests/ui.rs); on any other the harness runs its pass cases alone and says so on stderr, which
# the run above swallows, so the topic runs once more with its output kept and the line surfaces
# here. CI asserts the wording on the blessed compiler (ci.yml's rustc 1.99.0 job).
ui_log=$(mktemp)
if ! cargo test --quiet --test main ui_ -- --nocapture >"$ui_log" 2>&1; then
  cat "$ui_log" >&2
  rm -f "$ui_log"
  exit 1
fi
sed -n 's/^ui: skipping/regression: SKIPPED/p' "$ui_log" | sort -u >&2
rm -f "$ui_log"
# The headroom and counter overflow panics are asserted in every build, and only a release run can
# show that: a debug run would pass with a `debug_assert!` too.
cargo test --quiet --release --test main every_build >/dev/null
# The no-feature build, which only a separate member reaches; see tests/noalloc/Cargo.toml.
cargo test --quiet -p float-abel-noalloc >/dev/null
# The architecture claim on this machine, tests/golden/; ci.yml's golden jobs assert it on the other
# six targets.
cargo test --quiet -p float-abel-golden >/dev/null

if rustup run "$MSRV" cargo --version >/dev/null 2>&1; then
  cargo "+$MSRV" build --quiet --locked
  echo "regression: MSRV $MSRV builds the default features"
else
  echo "regression: SKIPPED the MSRV build (rustup toolchain $MSRV is not installed; CI runs it)" >&2
fi

# The dbsp feature on its own toolchain (CLAUDE.md § Commands): the clippy above never sees it,
# because it is out of the self dev-dependency's list, and `--test main` never builds its example.
DBSP_TOOLCHAIN=1.99.0
if rustup run "$DBSP_TOOLCHAIN" cargo --version >/dev/null 2>&1; then
  cargo "+$DBSP_TOOLCHAIN" test --quiet --features dbsp --test main dbsp_ >/dev/null
  cargo "+$DBSP_TOOLCHAIN" build --quiet --features dbsp --example dbsp_uda
  echo "regression: dbsp feature tests and example on rustc $DBSP_TOOLCHAIN"
else
  echo "regression: SKIPPED the dbsp feature (rustup toolchain $DBSP_TOOLCHAIN is not installed; CI runs it)" >&2
fi

# deny.toml is the crate's posture over every feature but dbsp; deny-dbsp.toml states what dbsp's
# own closure carries. Both must hold, and an ignored advisory that no longer matches any crate in
# the graph is an error, not a warning, so an ignore cannot outlive the crate that earned it.
if command -v cargo-deny >/dev/null 2>&1; then
  cargo deny --log-level warn check -D advisory-not-detected
  cargo deny --config deny-dbsp.toml --log-level warn check -D advisory-not-detected
else
  echo "regression: SKIPPED cargo-deny (not installed; CI runs it)" >&2
fi

echo "regression: OK"
