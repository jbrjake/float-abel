#!/usr/bin/env bash
# The commit gate (.githooks/pre-commit): under 60 s, scoped to the diff.
#
# The diff is the staged set when something is staged, else every tracked and untracked file, so a
# hand run checks the whole tree. A diff that reaches no Rust, script, manifest or hook skips the
# cargo legs and says so. The push gate, scripts/regression.sh, is the thorough one.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

# bash 3.2 on macOS: no mapfile, and an empty array is unbound under `set -u`.
files=()
while IFS= read -r f; do files+=("$f"); done < <(git diff --cached --name-only --diff-filter=ACMR)
if [ "${#files[@]}" -gt 0 ]; then
  scope="staged"
else
  while IFS= read -r f; do files+=("$f"); done < <(git ls-files --cached --others --exclude-standard)
  scope="tree"
fi

rust=0 store=0
for f in "${files[@]}"; do
  case "$f" in
    *.rs|Cargo.toml|Cargo.lock|scripts/*|.githooks/*|.cargo/*|deny.toml) rust=1 ;;
    docs/meshwork/*) store=1 ;;
  esac
done

./scripts/check-file-length.sh "${files[@]}"

if [ "$store" -eq 1 ]; then
  if [ -x docs/meshwork/meshwork ]; then
    docs/meshwork/meshwork lint
  else
    echo "smoke: SKIPPED meshwork lint (docs/meshwork/meshwork is not executable here)" >&2
  fi
fi

if [ "$rust" -eq 0 ]; then
  echo "smoke: OK ($scope diff reaches no Rust; cargo legs skipped)"
  exit 0
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "smoke: FAILED (no cargo on PATH, and the diff reaches Rust)" >&2
  exit 1
fi

./scripts/check-test-topics.sh
cargo fmt --all -- --check

if out=$(cargo test --quiet 2>&1); then
  passed=$(printf '%s\n' "$out" | { grep -Eo '[0-9]+ passed' || true; } | awk '{s += $1} END {print s + 0}')
  echo "smoke: OK ($scope diff; $passed tests passed)"
else
  echo "smoke: FAILED" >&2
  printf '%s\n' "$out" | grep -E 'FAILED|failures:|panicked at|^error(\[E[0-9]+\])?' >&2 \
    || printf '%s\n' "$out" | tail -n 40 >&2
  exit 1
fi
