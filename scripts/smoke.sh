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

rust=0 store=0 fuzz=0 actions=0 scripts=0
for f in "${files[@]}"; do
  case "$f" in
    fuzz/*) rust=1 fuzz=1 ;;
    scripts/*) rust=1 scripts=1 ;;
    *.rs|Cargo.toml|Cargo.lock|.githooks/*|.cargo/*|deny.toml) rust=1 ;;
    docs/meshwork/*) store=1 ;;
    .github/workflows/*) actions=1 ;;
  esac
done

./scripts/check-file-length.sh "${files[@]}"
# The commit message itself is checked by .githooks/commit-msg, which runs after this gate; the
# fixtures run here so a change to the checker is caught before the hook relies on it.
./scripts/check-commit-message.sh --self-test

if [ "$actions" -eq 1 ]; then
  ./scripts/check-actions-pinned.sh "${files[@]}"
fi

# The store lint is the one leg CI never runs: the shim execs a binary only a session's machine
# installs, and CI carries no meshwork dependency at all (owner: "not running meshwork calls in ci
# to avoid the dependency altogether"). The store is linted where it is edited, at commit time.
if [ "$store" -eq 1 ]; then
  if [ -n "${CI:-}" ]; then
    echo "smoke: SKIPPED meshwork lint (CI never installs meshwork; the store is linted at commit time)" >&2
  elif [ -x docs/meshwork/meshwork ]; then
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
# The release scripts' fixtures, when a script is in the diff: the version the release carries, the
# entry it ships and the pull request that cuts it. release-pr.sh's builds a git repository and runs
# cargo, so the push gate runs all three on every push and this gate only when they could have moved.
if [ "$scripts" -eq 1 ]; then
  ./scripts/version-from-log.sh --self-test
  ./scripts/changelog-entry.sh --self-test
  ./scripts/release-pr.sh --self-test
fi
cargo fmt --all -- --check

# The fuzz package is outside the workspace, so the root test below never reaches its oracle.
if [ "$fuzz" -eq 1 ]; then
  cargo fmt --manifest-path fuzz/Cargo.toml -- --check
  if ! cargo test --quiet --manifest-path fuzz/Cargo.toml >/dev/null; then
    echo "smoke: FAILED (the fuzz oracle's tests)" >&2
    exit 1
  fi
fi

if out=$(cargo test --quiet 2>&1); then
  passed=$(printf '%s\n' "$out" | { grep -Eo '[0-9]+ passed' || true; } | awk '{s += $1} END {print s + 0}')
  echo "smoke: OK ($scope diff; $passed tests passed)"
else
  echo "smoke: FAILED" >&2
  printf '%s\n' "$out" | grep -E 'FAILED|failures:|panicked at|^error(\[E[0-9]+\])?' >&2 \
    || printf '%s\n' "$out" | tail -n 40 >&2
  exit 1
fi
