#!/usr/bin/env bash
# tests/main.rs is the one integration-test target, and `autotests = false` keeps cargo from turning
# any other tests/*.rs into a second binary. The price is that a topic file nobody names in
# tests/main.rs never runs, silently. This check fails on such a file.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

status=0
for f in tests/*.rs; do
  [ -f "$f" ] || continue
  stem=$(basename "$f" .rs)
  [ "$stem" = main ] && continue
  if ! grep -Eq "^(pub )?mod ${stem};" tests/main.rs; then
    echo "check-test-topics: FAIL $f is not a \`mod ${stem};\` of tests/main.rs, so it never runs" >&2
    status=1
  fi
done
exit "$status"
