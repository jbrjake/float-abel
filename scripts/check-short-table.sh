#!/usr/bin/env bash
# The short table's gate legs, examples/short-table/ (DESIGN.md § Verification). The package stands
# outside the workspace, so the root legs never see it: its formatting, its tests, then the program
# itself, run `--locked` and diffed against examples/short-table/OUTPUT.md, so the file in git is
# always what the code prints, and a lockfile a release left stale fails loudly instead of being
# rewritten. `--lint` adds clippy on every target with warnings denied, for the push gate.
#
#   check-short-table.sh          the commit gate's legs
#   check-short-table.sh --lint   the push gate's: the same, plus clippy
#
# Every cargo leg shares one target directory under the root's, which git ignores and CI's gate job
# caches.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

manifest=examples/short-table/Cargo.toml
committed=examples/short-table/OUTPUT.md
target="${CARGO_TARGET_DIR:-target}/short-table"

cargo fmt --manifest-path "$manifest" -- --check
if [ "${1:-}" = --lint ]; then
  cargo clippy --quiet --manifest-path "$manifest" --target-dir "$target" --all-targets -- -D warnings
fi
cargo test --quiet --manifest-path "$manifest" --target-dir "$target" >/dev/null

printed=$(mktemp)
trap 'rm -f "$printed"' EXIT
cargo run --quiet --locked --manifest-path "$manifest" --target-dir "$target" >"$printed"
if ! diff -u "$committed" "$printed"; then
  echo "check-short-table: FAILED ($committed is not what the program prints; regenerate it:" >&2
  echo "  cargo run --manifest-path $manifest > $committed)" >&2
  exit 1
fi
echo "check-short-table: OK ($committed is what the program prints)"
