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

if ! command -v cargo >/dev/null 2>&1; then
  echo "regression: FAILED (no cargo on PATH)" >&2
  exit 1
fi

cargo fmt --all -- --check

# The lib alone, on each feature set a consumer can pick without the test features.
cargo clippy --quiet --lib --no-default-features -- -D warnings
cargo clippy --quiet --lib -- -D warnings
# Every target of every member: the self dev-dependency turns the test features on here.
cargo clippy --quiet --workspace --all-targets -- -D warnings

RUSTDOCFLAGS="-D warnings" cargo doc --quiet --no-deps

cargo test --quiet -- --include-ignored >/dev/null
# The headroom and counter overflow panics are asserted in every build, and only a release run can
# show that: a debug run would pass with a `debug_assert!` too.
cargo test --quiet --release --test main every_build >/dev/null
# The no-feature build, which only a separate member reaches; see tests/noalloc/Cargo.toml.
cargo test --quiet -p float-abel-noalloc >/dev/null
# The architecture claim on this machine, tests/golden/; .github/workflows/golden.yml asserts it on
# the other six targets.
cargo test --quiet -p float-abel-golden >/dev/null

if rustup run "$MSRV" cargo --version >/dev/null 2>&1; then
  cargo "+$MSRV" build --quiet --locked
  echo "regression: MSRV $MSRV builds the default features"
else
  echo "regression: SKIPPED the MSRV build (rustup toolchain $MSRV is not installed; CI runs it)" >&2
fi

if command -v cargo-deny >/dev/null 2>&1; then
  cargo deny --log-level warn check
else
  echo "regression: SKIPPED cargo-deny (not installed; CI runs it)" >&2
fi

echo "regression: OK"
